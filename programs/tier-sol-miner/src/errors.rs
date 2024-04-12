use anchor_lang::prelude::*;

#[error_code]
pub enum MinerError {
    #[msg("Token account is invalid: Might be due to insufficient balance for tier or lack of ownership of the specified account")]
    InvalidTokenAccount,
    #[msg("Selected TIER is not active")]
    InactiveTier,
    #[msg("Mine is currently inactive")]
    InactiveMine,
    #[msg("Referral account is invalid")]
    InvalidReferrer,
    #[msg("Invalid Fee Collector")]
    InvalidFeeCollector,
    #[msg("Whitelist has expired")]
    ExpiredWhiteList,
    #[msg("Invalid deposit amount")]
    InvalidDepositAmount,
    #[msg("Invalid tier: due to inactive of user_info tier mismatch")]
    InvalidTier
}