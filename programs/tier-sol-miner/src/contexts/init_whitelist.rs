use anchor_lang::prelude::*;
use solana_program::system_instruction;
use crate::states::mine::{MineInfo, MineVault};
use crate::states::tier::TierInfo;
use crate::states::user::UserInfo;
use crate::states::whitelist::WhitelistInfo;
use crate::errors::MinerError;
use crate::math::{calculate_fee, to_u128, calculate_interest};

/// Instruction to call for whitelisted users that have not started staking SOL
/// in a TIER. Initializes new user info account and accepts lamports to
/// deposit into the vault. Whitelisted users does not need to hold tokens
#[derive(Accounts)]
#[instruction(_tier_nonce: u8)]
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
        seeds = [b"mine"],
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
        seeds = [b"whitelist", signer.key().as_ref()],
        bump = whitelist_info.bump,
        close = signer
    )]
    pub whitelist_info: Account<'info, WhitelistInfo>,
    #[account(
        mut,
        seeds = [b"tier", &[_tier_nonce]],
        bump = tier_info.bump,
        constraint = tier_info.is_active && whitelist_info.tier == tier_info.key() && _tier_nonce == tier_info.nonce @ MinerError::InvalidTier
    )]
    pub tier_info: Account<'info, TierInfo>,
    #[account(
        mut,
        constraint = mine_info.fee_collector == fee_collector.key()
    )]
    pub fee_collector: SystemAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> InitWhiteList<'info> {
    pub fn consume_whitelist(
        &mut self,
        deposit_amount: u64,
        bump: u8,
        _tier_nonce: u8
    ) -> Result<()> {
        if deposit_amount <= 0 {
            return err!(MinerError::InvalidDepositAmount);
        }
        if (Clock::get()?.unix_timestamp as u64) > self.whitelist_info.expiry {
            return err!(MinerError::ExpiredWhiteList);
        }

        // Calculate fees and transfer lamports to vault and fee collector
        let dev_fee = match calculate_fee(
            to_u128(deposit_amount)?, 
            to_u128(self.mine_info.dev_fee)?
        ) {
            Ok(fee) => fee,
            Err(_) => return err!(MinerError::MathsError)
        };
        let actual_amount = deposit_amount.saturating_sub(dev_fee);
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
        let interest_accrued = match calculate_interest(
            to_u128(actual_amount)?, 
            to_u128(self.tier_info.apy)?, 
            to_u128(self.tier_info.lock_duration)?
        ) {
            Ok(interest) => interest,
            Err(_) => return err!(MinerError::MathsError)
        };
        self.user_info.set_inner(UserInfo {
            bump,
            owner: self.signer.key(),
            total_locked: actual_amount,
            accrued_interest: interest_accrued,
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