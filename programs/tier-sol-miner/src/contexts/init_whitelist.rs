use anchor_lang::prelude::*;
use solana_program::system_instruction;
use crate::states::mine::{MineInfo, MineVault};
use crate::states::tier::TierInfo;
use crate::states::user::UserInfo;
use crate::states::whitelist::WhitelistInfo;
use crate::errors::MinerError;
use crate::math::{calculate_fee, calculate_interest};

/// Instruction to call for whitelisted users that have not started staking SOL
/// in a TIER. Initializes new user info account and accepts lamports to
/// deposit into the vault. Whitelisted users does not need to hold tokens
#[derive(Accounts)]
#[instruction(_tier_name: &[u8])]
pub struct InitWhiteList<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + UserInfo::INIT_SPACE,
        seeds = [b"user", signer.key().as_ref()],
        bump
    )]
    pub user_info: Account<'info, UserInfo>,
    #[account(
        seeds = [b"mine".as_ref(), mine_info.admin.as_ref()],
        bump = mine_info.bump,
        constraint = mine_info.is_active @ MineError::InactiveMine
    )]
    pub mine_info: Account<'info, MineInfo>,
    #[account(
        mut,
        seeds = [b"mine-vault", mine_info.admin.as_ref()],
        bump = mine_vault.bump
    )]
    pub mine_vault: Account<'info, MineVault>,
    #[account(
        seeds = [b"whitelist", signer.key().as_ref(), mine_info.admin.as_ref()],
        bump = whitelist_info.bump,
        close = signer
    )]
    pub whitelist_info: Account<'info, WhitelistInfo>,
    #[account(
        mut,
        seeds: [tier_name.as_ref(), mine_info.admin.as_ref()],
        bump = tier.bump,
        constraint = tier.is_active @ MineError::InactiveTier
    )]
    pub tier_info: Account<'info, TierInfo>,
    pub fee_collector: SystemAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> InitWhiteList<'info> {
    pub fn consume_whitelist(
        &mut self,
        deposit_amount: u64,
        bump: u8,
        _tier_name: &[u8]
    ) -> Result<()> {
        if deposit_amount <= 0 {
            err!(MinerError::InvalidDepositAmount);
        }
        if (Clock::get()?.unix_timestamp as u64) > self.whitelist_info.expiry {
            err!(MinerError::ExpiredWhiteList);
        }

        // Calculate fees and transfer lamports to vault and fee collector
        let dev_fee = calculate_fee(deposit_amount, self.mine_info.dev_fee);
        let actual_amount = deposit_amount - dev_fee;
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

        // Initialize User info
        self.user_info.set_inner(UserInfo {
            bump,
            owner: self.signer.key(),
            total_locked: actual_amount,
            accrued_interest: calculate_interest(actual_amount, self.tier_info.apy, self.tier_info.lock_duration),
            lock_ts: Clock::get()?.unix_timestamp as u64,
            tier: self.tier_info.key(),
            is_whitelist: true
        });

        // Update Tier total locked
        let mut tier_info = self.tier_info.clone().into_inner();
        tier_info.total_locked = tier_info.total_locked.saturating_add(actual_amount);
        self.tier_info.set_inner(tier_info);
        Ok(())
    }
}