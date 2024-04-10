use anchor_lang::prelude::*;

#[account]
pub struct WhitelistInfo {
    pub beneficiary: Pubkey,
    pub tier: Pubkey,
    pub expiry: u64
}

impl Space for WhitelistInfo {
    const INIT_SPACE: usize = (32 * 2) + 8;
}