use anchor_lang::prelude::*;

#[account]
pub struct TierInfo {
    pub minimum_token_amount: u64,
    pub total_locked: u64,
    pub apy: u64,
    pub minimum_lock_period: u64,
    pub is_active: bool
}

impl Space for TierInfo {
    const INIT_SPACE: usize = (8 * 3) + 1;
}