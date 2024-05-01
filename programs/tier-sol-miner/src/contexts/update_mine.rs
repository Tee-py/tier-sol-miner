use anchor_lang::prelude::*;
use crate::states::mine::MineInfo;
use crate::errors::MinerError;

#[derive(Accounts)]
pub struct UpdateMine<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"mine".as_ref()],
        bump = mine_info.bump,
        constraint = mine_info.admin == admin.key() @ MinerError::InvalidMine
    )]
    pub mine_info: Account<'info, MineInfo>
}

impl<'info> UpdateMine<'info> {
    pub fn update_mine(
        &mut self,
        fee_collector: Option<Pubkey>,
        penalty_fee_collector: Option<Pubkey>,
        dev_fee: Option<u64>,
        early_withdrawal_fee: Option<u64>,
        referral_reward: Option<u64>,
        is_active: Option<bool>
    ) -> Result<()> {
        let mut mine_info = self.mine_info.clone().into_inner();
        match fee_collector {
            Some(val) => {mine_info.fee_collector = val},
            None => {}
        };
        match penalty_fee_collector {
            Some(val) => {mine_info.penalty_fee_collector = val},
            None => {}
        };
        match dev_fee {
            Some(val) => {mine_info.dev_fee = val},
            None => {}
        };
        match early_withdrawal_fee {
            Some(val) => {mine_info.early_withdrawal_fee = val},
            None => {}
        };
        match referral_reward {
            Some(val) => {mine_info.referral_reward = val},
            None => {}
        };
        match is_active {
            Some(val) => {mine_info.is_active = val},
            None => {}
        }
        self.mine_info.set_inner(mine_info);
        Ok(())
    }
}