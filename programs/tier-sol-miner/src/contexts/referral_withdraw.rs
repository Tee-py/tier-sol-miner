use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;
use crate::states::mine::{MineInfo, MineVault};
use crate::states::tier::TierInfo;
use crate::states::user::UserInfo;
use crate::states::referral::ReferralInfo;
use crate::errors::MinerError;
use crate::math::{calculate_fee, to_u128};

/// Instruction to call for users to increase their locked SOL
#[derive(Accounts)]
pub struct WithdrawReward<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", signer.key().as_ref()],
        bump = user_info.bump
    )]
    pub user_info: Account<'info, UserInfo>,
    #[account(
        mut,
        seeds = [b"referral", user_info.key().as_ref()],
        bump = referrer_info.bump
    )]
    pub referrer_info: Account<'info, ReferralInfo>,
    #[account(
        constraint = (
            token_account.mint == mine_info.token_mint &&
            token_account.amount >= tier_info.minimum_token_amount &&
            token_account.owner == signer.key()
        ) @ MinerError::InvalidTokenAccount
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        constraint = (tier_info.key() == user_info.tier) @ MinerError::InvalidTier
    )]
    pub tier_info: Account<'info, TierInfo>,
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
        constraint = mine_info.fee_collector == fee_collector.key()
    )]
    pub fee_collector: SystemAccount<'info>
}

impl<'info> WithdrawReward<'info> {
    pub fn withdraw(
        &mut self
    ) -> Result<()> {
        // Calculate fees and transfer lamports to vault and fee collector
        let dev_fee = match calculate_fee(
            to_u128(self.referrer_info.earnings)?, 
            to_u128(self.mine_info.dev_fee)?
        ) {
            Ok(fee) => fee,
            Err(_) => return err!(MinerError::MathsError)
        };
        let actual_amount = self.referrer_info.earnings.saturating_sub(dev_fee);
        self.mine_vault.sub_lamports(dev_fee)?;
        self.mine_vault.sub_lamports(actual_amount)?;
        self.signer.add_lamports(actual_amount)?;
        self.fee_collector.add_lamports(dev_fee)?;

        // Update Referral info
        let mut ref_info = self.referrer_info.clone().into_inner();
        ref_info.earnings = 0;
        self.referrer_info.set_inner(ref_info);
        msg!("New Earnings {}", self.referrer_info.earnings);
        Ok(())
    }
}