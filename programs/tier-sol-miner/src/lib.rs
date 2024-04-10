mod states;

use anchor_lang::prelude::*;

declare_id!("K35hGi544FaiNx7s1MJuLuBxhr993Bq59CJR9mBaUna");

#[program]
pub mod tier_sol_miner {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
