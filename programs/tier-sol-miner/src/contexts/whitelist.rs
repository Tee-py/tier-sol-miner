use anchor_lang::prelude::*;
use crate::states::tier::TierInfo;
use crate::states::whitelist::WhitelistInfo;

#[derive(Accounts)]
#[instruction(_tier_name: &[u8])]
pub struct WhiteList<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub beneficiary: AccountInfo<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + WhiteListInfo::INIT_SPACE,
        seeds = [b"whitelist", beneficiary.key().as_ref(), admin.key().as_ref()],
        bump
    )]
    pub whitelist_info: Account<'info, WhitelistInfo>,
    #[account(
        seeds = [_tier_name, admin.key().as_ref()],
        bump = tier_info.bump,
        constraint = tier_info.is_active
    )]
    pub tier_info: Account<'info, TierInfo>,
    pub system_program: Program<'info, System>
}

impl<'info> WhiteList<'info> {
    pub fn whitelist_account(&mut self, expiry: u64, bump: u8, _tier_name: &[u8]) -> Result<()> {
        self.whitelist_info.set_inner(WhitelistInfo {
            beneficiary,
            bump,
            expiry,
            tier: *self.tier_info.key()
        })
    }
}