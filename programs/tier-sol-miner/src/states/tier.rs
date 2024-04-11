use anchor_lang::prelude::*;

#[account]
pub struct TierInfo {
    pub minimum_token_amount: u64,
    pub total_locked: u64,
    pub apy: u64,
    pub lock_duration: u64,
    pub is_active: bool,
    pub bump: u8
}

impl Space for TierInfo {
    const INIT_SPACE: usize = (8 * 3) + 2;
}