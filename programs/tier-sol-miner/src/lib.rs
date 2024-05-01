mod states;
mod contexts;
mod math;
mod errors;

use contexts::*;
use anchor_lang::prelude::*;

declare_id!("K35hGi544FaiNx7s1MJuLuBxhr993Bq59CJR9mBaUna");

#[program]
pub mod tier_sol_miner {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        fee_collector: Pubkey,
        penalty_fee_collector: Pubkey,
        token_mint: Pubkey,
        dev_fee: u64,
        early_withdrawal_fee: u64,
        referral_reward: u64
    ) -> Result<()> {
        ctx.accounts.initialize_mine(
            ctx.bumps.mine_info,
            ctx.bumps.mine_vault,
            fee_collector,
            penalty_fee_collector,
            token_mint,
            dev_fee,
            early_withdrawal_fee,
            referral_reward,
        )?;
        Ok(())
    }

    pub fn add_tier(
        ctx: Context<AddTier>,
        apy: u64, minimum_token_amount: u64,
        lock_duration: u64
    ) -> Result<()> {
        ctx.accounts.add_tier(
            minimum_token_amount,
            apy, lock_duration,
            ctx.bumps.tier_info
        )?;
        Ok(())
    }

    pub fn whitelist_account(
        ctx: Context<WhiteList>,
        tier_nonce: u8,
        expiry: u64
    ) -> Result<()> {
        ctx.accounts.whitelist_account(
            expiry,
            ctx.bumps.whitelist_info,
            tier_nonce
        )?;
        Ok(())
    }

    pub fn initialize_staking(
        ctx: Context<InitStaking>,
        tier_nonce: u8,
        deposit_amount: u64
    ) -> Result<()> {
        ctx.accounts.initialize(
            deposit_amount,
            ctx.bumps.user_info,
            tier_nonce
        )?;
        Ok(())
    }

    pub fn initialize_staking_with_referrer(
        ctx: Context<InitStakingWithReferrer>,
        tier_nonce: u8,
        deposit_amount: u64
    ) -> Result<()> {
        ctx.accounts.initialize(
            deposit_amount,
            ctx.bumps.user_info,
            ctx.bumps.referrer_info,
            tier_nonce
        )?;
        Ok(())
    }

    pub fn initialize_whitelist(
        ctx: Context<InitWhiteList>,
        tier_nonce: u8,
        deposit_amount: u64
    ) -> Result<()> {
        ctx.accounts.consume_whitelist(
            deposit_amount,
            ctx.bumps.user_info,
            tier_nonce
        )?;
        Ok(())
    }

    pub fn increase_stake(
        ctx: Context<IncreaseStake>,
        amount: u64
    ) -> Result<()> {
        ctx.accounts.increase_stake(
            amount
        )?;
        Ok(())
    }

    pub fn compound(
        ctx: Context<Compound>,
    ) -> Result<()> {
        ctx.accounts.compound_interest()?;
        Ok(())
    }

    pub fn terminate_staking(
        ctx: Context<TerminateStaking>,
    ) -> Result<()> {
        ctx.accounts.terminate()?;
        Ok(())
    }

    pub fn claim_interest(
        ctx: Context<ClaimInterest>
    ) -> Result<()> {
        ctx.accounts.claim_interest()?;
        Ok(())
    }

    pub fn withdraw(
        ctx: Context<WithdrawStake>
    ) -> Result<()> {
        ctx.accounts.withdraw()?;
        Ok(())
    }

    pub fn whitelist_withdraw(
        ctx: Context<WithdrawWhitelistStake>
    ) -> Result<()> {
        ctx.accounts.withdraw()?;
        Ok(())
    }

    pub fn withdraw_referral_rewards(
        ctx: Context<WithdrawReward>
    ) -> Result<()> {
        ctx.accounts.withdraw()?;
        Ok(())
    }

    pub fn update_tier(
        ctx: Context<UpdateTier>,
        minimum_token_amount: Option<u64>,
        apy: Option<u64>,
        lock_duration: Option<u64>,
        is_active: Option<bool>
    ) -> Result<()> {
        ctx.accounts.update_tier(
            minimum_token_amount, 
            apy, 
            lock_duration, 
            is_active
        )?;
        Ok(())
    }

    pub fn update_mine(
        ctx: Context<UpdateMine>,
        fee_collector: Option<Pubkey>,
        penalty_fee_collector: Option<Pubkey>,
        dev_fee: Option<u64>,
        early_withdrawal_fee: Option<u64>,
        referral_reward: Option<u64>,
        is_active: Option<bool>
    ) -> Result<()> {
        ctx.accounts.update_mine(
            fee_collector, 
            penalty_fee_collector, 
            dev_fee, 
            early_withdrawal_fee, 
            referral_reward, 
            is_active
        )?;
        Ok(())
    }
}
