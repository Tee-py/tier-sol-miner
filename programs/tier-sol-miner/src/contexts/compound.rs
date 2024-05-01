use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;
use crate::states::mine::MineInfo;
use crate::states::tier::TierInfo;
use crate::states::user::UserInfo;
use crate::errors::MinerError;
use crate::math::{calculate_interest, to_u128};

/// Instruction to call for users to restake their interests
#[derive(Accounts)]
pub struct Compound<'info> {
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
        constraint = (tier_info.key() == user_info.tier && tier_info.is_active) @ MinerError::InvalidTier
    )]
    pub tier_info: Account<'info, TierInfo>,
}

impl<'info> Compound<'info> {
    pub fn compound_interest(
        &mut self
    ) -> Result<()> {
        // Update User info
        let current_interval = (Clock::get()?.unix_timestamp as u64) - self.user_info.lock_ts;
        if current_interval < self.tier_info.lock_duration {
            return err!(MinerError::OperationNotAllowed)
        }
        let mut user_info = self.user_info.clone().into_inner();
        let current_interest = user_info.accrued_interest;
        let new_total_locked = user_info.total_locked.saturating_add(current_interest);
        let new_interest = match calculate_interest(
            to_u128(new_total_locked)?, 
            to_u128(self.tier_info.apy)?, 
            to_u128(self.tier_info.lock_duration)?
        ) {
            Ok(val) => val,
            Err(_) => return err!(MinerError::MathsError)
        };
        user_info.accrued_interest = new_interest;
        user_info.lock_ts = Clock::get()?.unix_timestamp as u64;
        user_info.total_locked = new_total_locked;
        self.user_info.set_inner(user_info);

        // Update Tier total locked
        let mut tier_info = self.tier_info.clone().into_inner();
        tier_info.total_locked = tier_info.total_locked.saturating_add(current_interest);
        self.tier_info.set_inner(tier_info);

        Ok(())
    }
}