use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;
use solana_program::system_instruction;
use crate::states::mine::{MineInfo, MineVault};
use crate::states::tier::TierInfo;
use crate::states::user::UserInfo;
use crate::errors::MinerError;
use crate::math::{calculate_fee, calculate_interest, to_u128};

/// Instruction to call for users to increase their locked SOL
#[derive(Accounts)]
pub struct IncreaseStake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", signer.key().as_ref()],
        bump = user_info.bump,
        constraint = user_info.is_whitelist == false @ MinerError::OperationNotAllowed
    )]
    pub user_info: Account<'info, UserInfo>,
    #[account(
        constraint = (
            token_account.mint == mine_info.token_mint &&
            token_account.amount >= tier_info.minimum_token_amount &&
            token_account.owner == signer.key()
        ) @ MinerError::InvalidTokenAccount
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"mine".as_ref()],
        bump = mine_info.bump,
        constraint = mine_info.is_active @ MinerError::InvalidMine
    )]
    pub mine_info: Account<'info, MineInfo>,
    #[account(
        mut,
        seeds = [b"mine-vault"],
        bump = mine_vault.bump
    )]
    pub mine_vault: Account<'info, MineVault>,
    #[account(
        mut,
        constraint = (tier_info.key() == user_info.tier && tier_info.is_active) @ MinerError::InvalidTier
    )]
    pub tier_info: Account<'info, TierInfo>,
    #[account(
        mut,
        constraint = mine_info.fee_collector == fee_collector.key()
    )]
    pub fee_collector: SystemAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> IncreaseStake<'info> {
    pub fn increase_stake(
        &mut self,
        amount: u64
    ) -> Result<()> {
        if amount <= 0 {
            return err!(MinerError::InvalidDepositAmount);
        }
        // Calculate fees and transfer lamports to vault and fee collector
        let dev_fee = match calculate_fee(
            to_u128(amount)?, 
            to_u128(self.mine_info.dev_fee)?
        ) {
            Ok(fee) => fee,
            Err(_) => return err!(MinerError::MathsError)
        };
        let actual_amount = amount.saturating_sub(dev_fee);
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
        let current_interest = match calculate_interest(
            to_u128(user_info.total_locked)?, 
            to_u128(self.tier_info.apy)?, 
            to_u128(current_lock_duration)?
        ) {
            Ok(val) => val,
            Err(_) => return err!(MinerError::MathsError)
        };
        let new_total_locked = user_info.total_locked.saturating_add(actual_amount);
        let new_interest = match calculate_interest(
            to_u128(new_total_locked)?, 
            to_u128(self.tier_info.apy)?, 
            to_u128(self.tier_info.lock_duration)?
        ) {
            Ok(val) => val,
            Err(_) => return err!(MinerError::MathsError)
        };
        msg!("Interval: {}, New Accrued Interest: {}", current_lock_duration, new_interest.saturating_add(current_interest));
        user_info.accrued_interest = new_interest.saturating_add(current_interest);
        user_info.lock_ts = Clock::get()?.unix_timestamp as u64;
        user_info.total_locked = new_total_locked;
        self.user_info.set_inner(user_info);

        // Update Tier total locked
        let mut tier_info = self.tier_info.clone().into_inner();
        tier_info.total_locked = tier_info.total_locked.saturating_add(actual_amount);
        self.tier_info.set_inner(tier_info);

        Ok(())
    }
}