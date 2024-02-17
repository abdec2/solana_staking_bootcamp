use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Tokens are already staked.")]
    IsStaked,
    #[msg("Tokens not staked.")]
    NotStaked,
    #[msg("No Tokens to stake.")]
    NoTokens,
    #[msg("Staking period not expired")]
    StakingNotExpired,
    #[msg("Invalid staking period")]
    InvalidPeriod,
    #[msg("Not Allowed")]
    NotAllowed,
}
