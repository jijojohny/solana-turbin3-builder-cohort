use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Maker does not match escrow maker")]
    InvalidMaker,
    #[msg("Mint does not match escrow mint")]
    InvalidMint,
}
