use anchor_lang::prelude::*;
use crate::states::tier::TierInfo;
use crate::states::whitelist::WhitelistInfo;
use crate::errors::MinerError;

#[derive(Accounts)]
#[instruction(_tier_nonce: u8)]
pub struct WhiteList<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub beneficiary: SystemAccount<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + WhitelistInfo::INIT_SPACE,
        seeds = [b"whitelist", beneficiary.key().as_ref()],
        bump
    )]
    pub whitelist_info: Account<'info, WhitelistInfo>,
    #[account(
        mut,
        seeds = [b"tier".as_ref(), &[_tier_nonce]],
        bump = tier_info.bump,
        constraint = tier_info.is_active && _tier_nonce == tier_info.nonce @ MinerError::InvalidTier
    )]
    pub tier_info: Account<'info, TierInfo>,
    pub system_program: Program<'info, System>
}

impl<'info> WhiteList<'info> {
    pub fn whitelist_account(
        &mut self, 
        expiry: u64, 
        bump: u8, 
        _tier_nonce: u8
    ) -> Result<()> {
        self.whitelist_info.set_inner(WhitelistInfo {
            beneficiary: self.beneficiary.key(),
            bump,
            expiry,
            tier: self.tier_info.key()
        });
        Ok(())
    }
}