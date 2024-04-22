use anchor_lang::prelude::*;
use crate::states::tier::TierInfo;
use crate::states::mine::MineInfo;

#[derive(Accounts)]
pub struct AddTier<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + TierInfo::INIT_SPACE,
        seeds = [&[mine_info.current_tier_nonce], admin.key().as_ref()],
        bump
    )]
    pub tier_info: Account<'info, TierInfo>,
    #[account(
        mut,
        seeds = [b"mine".as_ref(), admin.key().as_ref()],
        bump = mine_info.bump,
        constraint = mine_info.admin == admin.key()
    )]
    pub mine_info: Account<'info, MineInfo>,
    pub system_program: Program<'info, System>
}

impl<'info> AddTier<'info> {
    pub fn add_tier(
        &mut self,
        minimum_token_amount: u64,
        apy: u64,
        lock_duration: u64,
        bump: u8
    ) -> Result<()> {
        self.tier_info.set_inner(TierInfo {
            lock_duration,
            minimum_token_amount,
            apy,
            is_active: true,
            total_locked: 0,
            bump,
            nonce: self.mine_info.current_tier_nonce
        });
        // Increase mine info tier nonce
        let mut mine_info = self.mine_info.clone().into_inner();
        mine_info.current_tier_nonce += 1;
        self.mine_info.set_inner(mine_info);
        Ok(())
    }
}