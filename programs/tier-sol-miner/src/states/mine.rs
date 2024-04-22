use anchor_lang::prelude::*;

#[account]
pub struct MineInfo {
    pub admin: Pubkey,
    pub token_mint: Pubkey,
    pub fee_collector: Pubkey,
    pub penalty_fee_collector: Pubkey,
    pub dev_fee: u64,
    pub early_withdrawal_fee: u64,
    pub referral_reward: u64,
    pub bump: u8,
    pub current_tier_nonce: u8,
    pub is_active: bool
}

impl Space for MineInfo {
    const INIT_SPACE: usize = (32 * 4) + (8 * 3) + 3;
}

#[account]
pub struct MineVault {
    pub bump: u8
}

impl Space for MineVault {
    const INIT_SPACE: usize = 1;
}