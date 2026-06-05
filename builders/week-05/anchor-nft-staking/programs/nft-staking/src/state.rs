use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeConfig {
    pub admin: Pubkey,
    pub collection: Pubkey,
    pub rewards_mint: Pubkey,
    pub rewards_bps: u16,
    pub freeze_period: u16,
    pub bump: u8,
    pub rewards_bump: u8,
}
