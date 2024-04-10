use anchor_lang::prelude::*;
use crate::states::tier::TierInfo;
use crate::states::mine::MineInfo;

#[derive(Accounts)]
#[instruction(tier_name: &[u8])]
pub struct AddTier<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer: admin,
        space: 8 + TierInfo::INIT_SPACE,
        seeds: [tier_name.as_ref(), admin.key().as_ref()],
        bump
    )]
    pub tier: Account<'info, TierInfo>,
    #[account(
        seeds = [b"mine".as_ref(), admin.key().as_ref()],
        bump = mine_account.bump,
        constraint = mine_account.admin == admin.key()
    )]
    pub mine_account: Account<'info, MineInfo>,
    pub system_program: Program<'info, System>
}

impl<'info> AddTier<'info> {
    pub fn add_tier(
        &mut self,
        minimum_token_amount: u64,
        apy: u64,
        minimum_lock_duration: u64,
        bump: u8
    ) -> Result<()> {
        self.tier.set_inner(TierInfo {
            minimum_lock_duration,
            minimum_token_amount,
            apy,
            is_active: true,
            total_locked: 0,
            bump
        });
        Ok(())
    }
}