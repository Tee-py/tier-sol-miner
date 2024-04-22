use anchor_lang::prelude::*;
use crate::states::user::UserInfo;
use crate::states::mine::{MineInfo, MineVault};
use crate::states::referral::ReferralInfo;
use crate::states::tier::TierInfo;
use solana_program::system_instruction;
use anchor_spl::token_interface::TokenAccount;
use crate::math::{to_u128, calculate_fee, calculate_interest};
use crate::errors::MinerError;

/// Instruction to call for new users that have not started staking SOL
/// in a TIER. Initializes new user info account and accepts lamports to
/// deposit into the vault.
#[derive(Accounts)]
#[instruction(_tier_nonce: u8)]
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
            token_account.amount >= tier_info.minimum_token_amount &&
            token_account.owner == signer.key()
        ) @ MinerError::InvalidTokenAccount
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"mine".as_ref(), mine_info.admin.as_ref()],
        bump = mine_info.bump,
        constraint = mine_info.is_active @ MinerError::InactiveMine
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
        seeds = [&[_tier_nonce], mine_info.admin.as_ref()],
        bump = tier_info.bump,
        constraint = tier_info.is_active && _tier_nonce == tier_info.nonce @ MinerError::InvalidTier
    )]
    pub tier_info: Account<'info, TierInfo>,
    #[account(
        mut,
        constraint = mine_info.fee_collector == fee_collector.key()
    )]
    pub fee_collector: SystemAccount<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(_tier_nonce: u8)]
pub struct InitStakingWithReferrer<'info> {
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
            token_account.amount >= tier_info.minimum_token_amount &&
            token_account.owner == signer.key()
        ) @ MinerError::InvalidTokenAccount
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"mine".as_ref(), mine_info.admin.as_ref()],
        bump = mine_info.bump,
        constraint = mine_info.is_active @ MinerError::InactiveMine
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
        seeds = [&[_tier_nonce], mine_info.admin.as_ref()],
        bump = tier_info.bump,
        constraint = tier_info.is_active && _tier_nonce == tier_info.nonce @ MinerError::InvalidTier
    )]
    pub tier_info: Account<'info, TierInfo>,
    #[account(
        constraint = signer.key() != referrer_user_info.owner @ MinerError::InvalidReferrer
    )]
    pub referrer_user_info: Account<'info, UserInfo>,
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + ReferralInfo::INIT_SPACE,
        seeds = [b"referral", referrer_user_info.key().as_ref()],
        bump
    )]
    pub referrer_info: Account<'info, ReferralInfo>,
    #[account(
        mut,
        constraint = mine_info.fee_collector == fee_collector.key() @ MinerError::InvalidFeeCollector
    )]
    pub fee_collector: SystemAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> InitStaking<'info> {
    pub fn initialize(
        &mut self,
        deposit_amount: u64,
        bump: u8,
        _tier_nonce: u8
    ) -> Result<()> {
        if deposit_amount <= 0 {
            return err!(MinerError::InvalidDepositAmount);
        }
        // Calculate fees and transfer lamports to vault and fee collector
        let dev_fee = match calculate_fee(to_u128(deposit_amount)?, to_u128(self.mine_info.dev_fee)?) {
            Ok(fee) => fee,
            Err(_e) => return err!(MinerError::MathsError)
        };
        let actual_amount = match deposit_amount.checked_sub(dev_fee) {
            Some(val) => val,
            None => return err!(MinerError::MathsError)
        };
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
        msg!(
            "Interest: {}, APY: {}, Duration: {}, Dev Fee: {}, Total Locked: {}", 
            interest_accrued, self.tier_info.apy, self.tier_info.lock_duration, dev_fee, actual_amount
        );
        self.user_info.set_inner(UserInfo {
            bump,
            owner: self.signer.key(),
            total_locked: actual_amount,
            accrued_interest: interest_accrued,
            lock_ts: Clock::get()?.unix_timestamp as u64,
            tier: self.tier_info.key(),
            is_whitelist: false
        });

        // Update Tier total locked
        let mut tier_info = self.tier_info.clone().into_inner();
        tier_info.total_locked = tier_info.total_locked.saturating_add(actual_amount);
        self.tier_info.set_inner(tier_info);
        Ok(())
    }
}

// impl<'info> InitStakingWithReferrer<'info> {
//     pub fn initialize(
//         &mut self,
//         deposit_amount: u64,
//         bump: u8,
//         _tier_nonce: u8
//     ) -> Result<()> {
//         if deposit_amount <= 0 {
//             return err!(MinerError::InvalidDepositAmount);
//         }
//         // Calculate fees and transfer lamports to vault and fee collector
//         let dev_fee = calculate_fee(deposit_amount, self.mine_info.dev_fee);
//         let actual_amount = deposit_amount - dev_fee;
//         let fee_transfer_ix = system_instruction::transfer(
//             self.signer.key,
//             &self.mine_info.fee_collector,
//             dev_fee
//         );
//         let actual_transfer_ix = system_instruction::transfer(
//             self.signer.key,
//             &self.mine_vault.key(),
//             actual_amount
//         );
//         solana_program::program::invoke_signed(
//             &fee_transfer_ix,
//             &[
//                 self.signer.to_account_info(),
//                 self.fee_collector.to_account_info(),
//                 self.system_program.to_account_info(),
//             ],
//             &[],
//         )?;
//         solana_program::program::invoke_signed(
//             &actual_transfer_ix,
//             &[
//                 self.signer.to_account_info(),
//                 self.mine_vault.to_account_info(),
//                 self.system_program.to_account_info(),
//             ],
//             &[],
//         )?;

//         // Initialize User info
//         self.user_info.set_inner(UserInfo {
//             bump,
//             owner: self.signer.key(),
//             total_locked: actual_amount,
//             accrued_interest: calculate_interest(actual_amount, self.tier_info.apy, self.tier_info.lock_duration),
//             lock_ts: Clock::get()?.unix_timestamp as u64,
//             tier: self.tier_info.key(),
//             is_whitelist: false
//         });

//         // Update Tier total locked
//         let mut tier_info = self.tier_info.clone().into_inner();
//         tier_info.total_locked = tier_info.total_locked.saturating_add(actual_amount);
//         self.tier_info.set_inner(tier_info);

//         // Handle referral
//         let bonus = calculate_fee(actual_amount, self.mine_info.referral_reward);
//         let mut referral_info = self.referrer_info.clone().into_inner();
//         if referral_info.earnings == 0 {
//             referral_info.earnings = bonus;
//             referral_info.count = 1;
//             referral_info.owner = self.referrer_user_info.owner;
//             referral_info.user_info = self.referrer_user_info.key();
//         } else {
//             referral_info.count += 1;
//             referral_info.earnings = referral_info.earnings.saturating_add(bonus);
//         }
//         self.referrer_info.set_inner(referral_info);
//         Ok(())
//     }
// }