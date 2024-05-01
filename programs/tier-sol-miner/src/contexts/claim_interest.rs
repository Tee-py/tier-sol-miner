use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;
use crate::states::mine::{MineInfo, MineVault};
use crate::states::tier::TierInfo;
use crate::states::user::UserInfo;
use crate::errors::MinerError;
use crate::math::{calculate_fee, calculate_interest, to_u128};

/// Instruction to call for normal users to claim their interests
#[derive(Accounts)]
pub struct ClaimInterest<'info> {
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
        constraint = (tier_info.key() == user_info.tier && tier_info.is_active) @ MinerError::InvalidTier
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
}

impl<'info> ClaimInterest<'info> {
    pub fn claim_interest(
        &mut self
    ) -> Result<()> {
        // Update User info
        let mut user_info = self.user_info.clone().into_inner();
        let current_interest = user_info.accrued_interest;
        let current_interval = (Clock::get()?.unix_timestamp as u64).saturating_sub(user_info.lock_ts);
        let dev_fee = calculate_fee(
            to_u128(current_interest)?, 
            to_u128(self.mine_info.dev_fee)?
        )?;

        // Calculate amount out based on early claim penalty and dev fee
        let (amount_out, penalty) = if current_interval >= self.tier_info.lock_duration {
            // update with the new interest and the lock timestamp
            let new_interest = match calculate_interest(
                to_u128(user_info.total_locked)?, 
                to_u128(self.tier_info.apy)?, 
                to_u128(self.tier_info.lock_duration)?
            ) {
                Ok(val) => val,
                Err(_) => return err!(MinerError::MathsError)
            };
            user_info.accrued_interest = new_interest;
            user_info.lock_ts = Clock::get()?.unix_timestamp as u64;
            (current_interest.saturating_sub(dev_fee), 0_u64)
        } else {
            let penalty = calculate_fee(
                to_u128(current_interest)?, 
                to_u128(self.mine_info.early_withdrawal_fee)?
            )?;
            user_info.accrued_interest = 0;
            user_info.lock_ts = Clock::get()?.unix_timestamp as u64;
            msg!("Penalty: {}", penalty);
            (current_interest.saturating_sub(dev_fee).saturating_sub(penalty), penalty)
        };
        let vault_bal = self.mine_vault.get_lamports();
        msg!("Vault Balance: {}", vault_bal);
        msg!(
            "Amount Out {}, Dev Fee: {}, Interest: {}, Duration: {}", 
            amount_out, dev_fee, current_interest, current_interval
        );
        // Send amount out and dev fee
        self.fee_collector.add_lamports(dev_fee)?;
        self.signer.add_lamports(amount_out)?;
        self.penalty_collector.add_lamports(penalty)?;
        self.mine_vault.sub_lamports(dev_fee.saturating_add(amount_out).saturating_add(penalty))?;
        self.user_info.set_inner(user_info);

        Ok(())
    }
}