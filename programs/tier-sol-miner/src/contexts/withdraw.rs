use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;
use crate::states::mine::{MineInfo, MineVault};
use crate::states::referral::ReferralInfo;
use crate::states::tier::TierInfo;
use crate::states::user::UserInfo;
use crate::errors::MinerError;
use crate::math::{calculate_fee, to_u128};

/// Instruction to call for users to increase their locked SOL
#[derive(Accounts)]
pub struct WithdrawStake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", signer.key().as_ref()],
        bump = user_info.bump,
        constraint = user_info.is_whitelist == false @ MinerError::OperationNotAllowed,
        close = signer
    )]
    pub user_info: Account<'info, UserInfo>,
    #[account(
        mut,
        seeds = [b"referral", user_info.key().as_ref()],
        bump = referrer_info.bump,
        close = signer
    )]
    pub referrer_info: Option<Account<'info, ReferralInfo>>,
    #[account(
        constraint = (
            token_account.mint == mine_info.token_mint &&
            token_account.amount >= tier_info.minimum_token_amount &&
            token_account.owner == signer.key()
        ) @ MinerError::InvalidTokenAccount
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
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
        constraint = (tier_info.key() == user_info.tier) @ MinerError::InvalidTier
    )]
    pub tier_info: Account<'info, TierInfo>,
    #[account(
        mut,
        constraint = mine_info.fee_collector == fee_collector.key()
    )]
    pub fee_collector: SystemAccount<'info>,
    #[account(
        mut,
        constraint = mine_info.penalty_fee_collector == penalty_collector.key()
    )]
    pub penalty_collector: SystemAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> WithdrawStake<'info> {
    pub fn withdraw(
        &mut self
    ) -> Result<()> {
        // Calculate fees and transfer lamports to vault and fee collector
        let total_withdrawal = self.user_info.accrued_interest.saturating_add(self.user_info.total_locked);
        let lock_duration = (Clock::get()?.unix_timestamp as u64).saturating_sub(self.user_info.lock_ts);
        let dev_fee = match calculate_fee(
            to_u128(total_withdrawal)?, 
            to_u128(self.mine_info.dev_fee)?
        ) {
            Ok(fee) => fee,
            Err(_) => return err!(MinerError::MathsError)
        };
        let amount_out = if lock_duration >= self.tier_info.lock_duration {
            total_withdrawal.saturating_sub(dev_fee)
        } else {
            let penalty = calculate_fee(
                to_u128(total_withdrawal)?, 
                to_u128(self.mine_info.early_withdrawal_fee)?
            )?;
            self.mine_vault.sub_lamports(penalty)?;
            self.penalty_collector.add_lamports(penalty)?;
            total_withdrawal.saturating_sub(dev_fee).saturating_sub(penalty)
        };

        self.mine_vault.sub_lamports(dev_fee)?;
        self.fee_collector.add_lamports(dev_fee)?;
        self.mine_vault.sub_lamports(amount_out)?;
        self.signer.add_lamports(amount_out)?;

        // Update Tier total locked
        let mut tier_info = self.tier_info.clone().into_inner();
        tier_info.total_locked = tier_info.total_locked.saturating_sub(self.user_info.total_locked);
        self.tier_info.set_inner(tier_info);

        Ok(())
    }
}

/// Instruction to call for whitelist users to increase their locked SOL
#[derive(Accounts)]
pub struct WithdrawWhitelistStake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        constraint = user_info.is_whitelist == true @ MinerError::OperationNotAllowed,
        close = signer
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
        constraint = (tier_info.key() == user_info.tier) @ MinerError::InvalidTier
    )]
    pub tier_info: Account<'info, TierInfo>,
    #[account(
        mut,
        constraint = mine_info.fee_collector == fee_collector.key()
    )]
    pub fee_collector: SystemAccount<'info>,
    #[account(
        mut,
        constraint = mine_info.penalty_fee_collector == penalty_collector.key()
    )]
    pub penalty_collector: SystemAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> WithdrawWhitelistStake<'info> {
    pub fn withdraw(
        &mut self
    ) -> Result<()> {
        // Calculate fees and transfer lamports to vault and fee collector
        let total_withdrawal = self.user_info.accrued_interest.saturating_add(self.user_info.total_locked);
        let lock_duration = (Clock::get()?.unix_timestamp as u64).saturating_sub(self.user_info.lock_ts);
        let dev_fee = match calculate_fee(
            to_u128(total_withdrawal)?, 
            to_u128(self.mine_info.dev_fee)?
        ) {
            Ok(fee) => fee,
            Err(_) => return err!(MinerError::MathsError)
        };
        let amount_out = if lock_duration >= self.tier_info.lock_duration {
            total_withdrawal.saturating_sub(dev_fee)
        } else {
            let penalty = calculate_fee(
                to_u128(total_withdrawal)?, 
                to_u128(self.mine_info.early_withdrawal_fee)?
            )?;
            self.mine_vault.sub_lamports(penalty)?;
            self.penalty_collector.add_lamports(penalty)?;
            total_withdrawal.saturating_sub(dev_fee).saturating_sub(penalty)
        };

        self.mine_vault.sub_lamports(dev_fee)?;
        self.fee_collector.add_lamports(dev_fee)?;
        self.mine_vault.sub_lamports(amount_out)?;
        self.signer.add_lamports(amount_out)?;

        // Update Tier total locked
        let mut tier_info = self.tier_info.clone().into_inner();
        tier_info.total_locked = tier_info.total_locked.saturating_sub(self.user_info.total_locked);
        self.tier_info.set_inner(tier_info);

        Ok(())
    }
}