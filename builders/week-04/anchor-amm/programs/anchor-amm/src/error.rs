use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("mint_a must be less than mint_b")]
    InvalidMintOrder,
    #[msg("fee_bps exceeds maximum allowed fee")]
    FeeTooHigh,
    #[msg("mints must be different")]
    SameMint,
    #[msg("amount must be greater than zero")]
    ZeroAmount,
    #[msg("slippage limit exceeded")]
    SlippageExceeded,
    #[msg("pool has insufficient liquidity")]
    InsufficientLiquidity,
    #[msg("overflow in arithmetic")]
    MathOverflow,
}
