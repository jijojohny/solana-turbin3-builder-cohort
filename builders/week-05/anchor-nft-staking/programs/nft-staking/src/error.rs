use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("invalid attribute timestamp")]
    InvalidTimestamp,
    #[msg("asset is already staked")]
    AlreadyStaked,
    #[msg("asset is not staked")]
    NotStaked,
    #[msg("staking attributes not initialized")]
    AttributesNotInitialized,
    #[msg("freeze period has not elapsed")]
    FreezePeriodNotElapsed,
    #[msg("no rewards to claim")]
    NoRewards,
    #[msg("collection update authority mismatch")]
    InvalidUpdateAuthority,
    #[msg("arithmetic overflow")]
    MathOverflow,
    #[msg("staked count underflow")]
    StakedCountUnderflow,
}
