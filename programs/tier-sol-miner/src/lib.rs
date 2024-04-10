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
}
