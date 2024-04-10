use anchor_lang::prelude::*;

#[account]
pub struct UserInfo {
    pub bump: u8,
    pub owner: Pubkey,
    pub total_locked: u64,
    pub accrued_interest: u64,
    pub lock_ts: u64,
    pub tier: Pubkey,
    pub is_whitelist: bool
}

impl Space for UserInfo {
    const INIT_SPACE: usize = (32 * 2) + (3 * 8) + 2;
}