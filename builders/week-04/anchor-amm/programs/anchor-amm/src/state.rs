use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub admin: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub lp_mint: Pubkey,
    pub fee_bps: u16,
    pub bump: u8,
    pub lp_bump: u8,
}
