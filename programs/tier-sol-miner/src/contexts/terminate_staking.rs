use anchor_lang::prelude::*;
use crate::states::mine::{MineInfo, MineVault};
use crate::states::tier::TierInfo;
use crate::states::user::UserInfo;
use crate::states::referral::ReferralInfo;
use crate::errors::MinerError;
use crate::math::{to_u128, calculate_fee};

/// Instruction to call for users to restake their interests
#[derive(Accounts)]
pub struct TerminateStaking<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        constraint = user_info.is_whitelist == false @ MinerError::OperationNotAllowed,
        close = admin
    )]
    pub user_info: Account<'info, UserInfo>,
    #[account(
        mut,
        seeds = [b"referral", user_info.key().as_ref()],
        bump = referrer_info.bump,
        close = admin
    )]
    pub referrer_info: Option<Account<'info, ReferralInfo>>,
    #[account(
        seeds = [b"mine"],
        bump = mine_info.bump,
        constraint = mine_info.admin == admin.key() @ MinerError::OperationNotAllowed
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
        constraint = tier_info.key() == user_info.tier @ MinerError::InvalidTier
    )]
    pub tier_info: Account<'info, TierInfo>,
    #[account(
        mut,
        constraint = user_info.owner == user_account.key()
    )]
    pub user_account: SystemAccount<'info>,
    #[account(
        mut,
        constraint = mine_info.fee_collector == fee_collector.key()
    )]
    pub fee_collector: SystemAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> TerminateStaking<'info> {
    pub fn terminate(
        &mut self
    ) -> Result<()> {
        // Calculate fees and transfer lamports to vault and fee collector
        let dev_fee = match calculate_fee(
            to_u128(self.user_info.total_locked)?, 
            to_u128(self.mine_info.dev_fee)?
        ) {
            Ok(fee) => fee,
            Err(_) => return err!(MinerError::MathsError)
        };
        let amount_out = self.user_info.total_locked.saturating_sub(dev_fee);
        self.mine_vault.sub_lamports(dev_fee)?;
        self.fee_collector.add_lamports(dev_fee)?;
        self.mine_vault.sub_lamports(amount_out)?;
        self.user_account.add_lamports(amount_out)?;

        // Update Tier total locked
        let mut tier_info = self.tier_info.clone().into_inner();
        tier_info.total_locked = tier_info.total_locked.saturating_sub(self.user_info.total_locked);
        self.tier_info.set_inner(tier_info);

        Ok(())
    }
}