use anchor_lang::prelude::*;
use crate::states::user::UserInfo;
use crate::states::mine::{MineInfo, MineVault};
use crate::states::referral::ReferralInfo;
use crate::states::tier::TierInfo;
use crate::states::whitelist::WhitelistInfo;
use solana_program::system_instruction;
use anchor_spl::token::TokenAccount;
use crate::math::{calculate_fee, calculate_interest};
use crate::errors::MinerError;

/// Instruction to call for new users that have not started staking SOL
/// in a TIER. Initializes new user info account and accepts lamports to
/// deposit into the vault.
#[derive(Accounts)]
#[instruction(_tier_name: &[u8])]
pub struct InitStaking<'info> {
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
        mut,
        seeds: [tier_name.as_ref(), mine_info.admin.as_ref()],
        bump = tier.bump,
        constraint = tier.is_active @ MineError::InactiveTier
    )]
    pub tier: Account<'info, TierInfo>,
    #[account(
        constraint = signer.key() != referrer_user_info.owner @ MineError::InvalidReferrer
    )]
    pub referrer_user_info: Option<Account<'info, UserInfo>>,
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + ReferralInfo::INIT_SPACE,
        seeds = [b"referral", referrer_user_info.key().as_ref()],
        bump
    )]
    pub referrer_info: Option<Account<'info, ReferralInfo>>,
    #[account(
        mut,
        constraint = mine_info.fee_collector == fee_collector.key() @ MineError::InvalidFeeCollector
    )]
    pub fee_collector: SystemAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> InitStaking<'info> {
    pub fn initialize(
        &mut self,
        deposit_amount: u64,
        bump: u8,
        _tier_name: &[u8]
    ) -> Result<()> {
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
            accrued_interest: calculate_interest(actual_amount, self.tier.apy, self.tier.lock_duration),
            lock_ts: Clock::get()?.unix_timestamp as u64,
            tier: self.tier.key(),
            is_whitelist: false
        });

        // Update Tier total locked
        let mut tier_info = self.tier.clone().into_inner();
        tier_info.total_locked = tier_info.total_locked.saturating_add(actual_amount);
        self.tier.set_inner(tier_info);

        // Handle referral
        match &mut self.referrer_info {
            Some(referrer) => {
                match &mut self.referrer_user_info {
                    Some(ref_user_info) => {
                        let bonus = calculate_fee(actual_amount, self.mine_info.referral_reward);
                        let mut referral_info = referrer.clone().into_inner();
                        if referral_info.earnings == 0 {
                            referral_info.earnings = bonus;
                            referral_info.count = 1;
                            referral_info.owner = ref_user_info.owner;
                            referral_info.user_info = ref_user_info.key();
                        } else {
                            referral_info.count += 1;
                            referral_info.earnings = referral_info.earnings.saturating_add(bonus);
                        }
                        referrer.set_inner(referral_info);
                    },
                    None => {}
                }
            },
            None => {}
        }
        Ok(())
    }
}
