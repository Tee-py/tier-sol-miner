use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use solana_program::system_instruction;
use crate::states::mine::{MineInfo, MineVault};
use crate::states::tier::TierInfo;
use crate::states::user::UserInfo;
use crate::errors::MinerError;
use crate::math::{calculate_fee, calculate_interest};

/// Instruction to call for users to increase their locked SOL
#[derive(Accounts)]
pub struct IncreaseStake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds = [b"user", signer.key().as_ref()],
        bump = user_info.bump
    )]
    pub user_info: Account<'info, UserInfo>,
    #[account(
        constraint = (
            token_account.mint == mine_info.token_mint &&
            token_account.amount >= tier.minimum_token_amount &&
            token_account.owner == signer
        ) @ MineError::InvalidTokenAccount
    )]
    pub token_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"mine".as_ref(), mine_info.admin.as_ref()],
        bump = mine_info.bump,
        constraint = mine_info.is_active @ MineError::InvalidTier
    )]
    pub mine_info: Account<'info, MineInfo>,
    #[account(
        mut,
        seeds = [b"mine-vault", mine_info.admin.as_ref()],
        bump = mine_vault.bump
    )]
    pub mine_vault: Account<'info, MineVault>,
    #[account(
        mut,
        constraint = (tier_info.key() == user_info.tier && tier_info.is_Active) @ MineError::InactiveTier
    )]
    pub tier_info: Account<'info, TierInfo>,
    pub fee_collector: SystemAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> IncreaseStake<'info> {
    pub fn increase_stake(
        &mut self,
        amount: u64
    ) -> Result<()> {
        if amount <= 0 {
            err!(MinerError::InvalidDepositAmount);
        }
        // Calculate fees and transfer lamports to vault and fee collector
        let dev_fee = calculate_fee(amount, self.mine_info.dev_fee);
        let actual_amount = amount - dev_fee;
        let fee_transfer_ix = system_instruction::transfer(
            self.signer.key,
            &self.mine_info.fee_collector,
            dev_fee
        );
        let actual_transfer_ix = system_instruction::transfer(
            self.signer.key,
            &self.mine_vault.key(),
            actual_amount
        );
        solana_program::program::invoke_signed(
            &fee_transfer_ix,
            &[
                self.signer.to_account_info(),
                self.fee_collector.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &[],
        )?;
        solana_program::program::invoke_signed(
            &actual_transfer_ix,
            &[
                self.signer.to_account_info(),
                self.mine_vault.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &[],
        )?;

        // Update User info
        let mut user_info = self.user_info.clone().into_inner();
        let current_lock_duration = (Clock::get()?.unix_timestamp as u64) - user_info.lock_ts;
        let current_interest = calculate_interest(user_info.total_locked, self.tier_info.apy, current_lock_duration);
        let new_total_locked = user_info.total_locked + actual_amount;
        let new_interest = calculate_interest(new_total_locked, self.tier_info.apy, self.tier_info.lock_duration);
        user_info.total_locked = new_total_locked;
        user_info.accrued_interest = new_interest + current_interest;
        user_info.lock_ts = Clock::get()?.unix_timestamp as u64;
        self.user_info.set_inner(user_info);

        // Update Tier total locked
        let mut tier_info = self.tier_info.clone().into_inner();
        tier_info.total_locked = tier_info.total_locked.saturating_add(actual_amount);
        self.tier_info.set_inner(tier_info);

        Ok(())
    }
}