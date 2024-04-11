use anchor_lang::prelude::*;

#[account]
pub struct MineInfo {
    pub admin: Pubkey,
    pub token_mint: Pubkey,
    pub fee_collector: Pubkey,
    pub dev_fee: u64,
    pub early_withdrawal_fee: u64,
    pub referral_reward: u64,
    pub bump: u8,
    pub is_active: bool
}

impl Space for MineInfo {
    const INIT_SPACE: usize = (32 * 2) + (8 * 3) + 2;
}

#[account]
pub struct MineVault {
    pub bump: u8
}

impl Space for MineVault {
    const INIT_SPACE: usize = 1;
}