use anchor_lang::prelude::*;
use crate::states::tier::TierInfo;
use crate::states::mine::MineInfo;
use crate::errors::MinerError;

#[derive(Accounts)]
pub struct UpdateTier<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
    )]
    pub tier_info: Account<'info, TierInfo>,
    #[account(
        seeds = [b"mine".as_ref()],
        bump = mine_info.bump,
        constraint = mine_info.admin == admin.key() @ MinerError::InvalidMine
    )]
    pub mine_info: Account<'info, MineInfo>,
    //pub system_program: Program<'info, System>
}

impl<'info> UpdateTier<'info> {
    pub fn update_tier(
        &mut self,
        minimum_token_amount: Option<u64>,
        apy: Option<u64>,
        lock_duration: Option<u64>,
        is_active: Option<bool>
    ) -> Result<()> {
        let mut tier_info = self.tier_info.clone().into_inner();
        match minimum_token_amount {
            Some(val) => {tier_info.minimum_token_amount = val},
            None => {}
        };
        match apy {
            Some(val) => {tier_info.apy = val},
            None => {}
        };
        match lock_duration {
            Some(val) => {tier_info.lock_duration = val},
            None => {}
        };
        match is_active {
            Some(val) => {tier_info.is_active = val},
            None => {}
        }
        self.tier_info.set_inner(tier_info);
        Ok(())
    }
}