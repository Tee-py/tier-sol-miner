use anchor_lang::prelude::*;
use crate::states::mine::{MineVault, MineInfo};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        init,
        payer = initializer,
        space = 8 + MineInfo::INIT_SPACE,
        seeds = [b"mine".as_ref()],
        bump
    )]
    pub mine_info: Account<'info, MineInfo>,
    #[account(
        init,
        payer = initializer,
        space = 8 + MineVault::INIT_SPACE,
        seeds = [b"mine-vault".as_ref()],
        bump
    )]
    pub mine_vault: Account<'info, MineVault>,
    pub system_program: Program<'info, System>
}

impl<'info> Initialize<'info> {
    pub fn initialize_mine(
        &mut self,
        bump1: u8,
        bump2: u8,
        fee_collector: Pubkey,
        penalty_fee_collector: Pubkey,
        token_mint: Pubkey,
        dev_fee: u64,
        early_withdrawal_fee: u64,
        referral_reward: u64
    ) -> Result<()> {
        self.mine_info.set_inner(MineInfo {
            admin: *self.initializer.key,
            token_mint,
            fee_collector,
            penalty_fee_collector,
            dev_fee,
            early_withdrawal_fee,
            referral_reward,
            is_active: true,
            bump: bump1,
            current_tier_nonce: 0
        });
        self.mine_vault.set_inner(MineVault {
            bump: bump2
        });
        Ok(())
    }
}