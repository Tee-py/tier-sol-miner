use anchor_lang::prelude::*;

#[account]
pub struct ReferralInfo {
    pub user_info: Pubkey,
    pub owner: Pubkey,
    pub earnings: u64,
    pub count: u64
}

impl Space for ReferralInfo {
    const INIT_SPACE: usize = (32 * 2) + (8 * 2);
}