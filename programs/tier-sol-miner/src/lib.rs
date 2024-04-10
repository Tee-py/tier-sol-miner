mod states;
mod contexts;

use contexts::*;

use anchor_lang::prelude::*;

declare_id!("K35hGi544FaiNx7s1MJuLuBxhr993Bq59CJR9mBaUna");

#[program]
pub mod tier_sol_miner {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        fee_collector: Pubkey,
        dev_fee: u64,
        early_withdrawal_fee: u64,
        referral_reward: u64
    ) -> Result<()> {
        let x = b"hello";
        ctx.accounts.initialize_mine(
            ctx.bumps.mine_account,
            ctx.bumps.mine_vault,
            fee_collector,
            dev_fee,
            early_withdrawal_fee,
            referral_reward
        )?;
        Ok(())
    }

    pub fn add_tier(
        ctx: Context<AddTier>,
        apy: u64, minimum_token_amount: u64,
        minimum_lock_duration: u64
    ) -> Result<()> {
        ctx.accounts.add_tier(
            minimum_token_amount,
            apy,
            minimum_lock_duration,
            ctx.bumps.tier
        )?;
        Ok(())
    }
}
