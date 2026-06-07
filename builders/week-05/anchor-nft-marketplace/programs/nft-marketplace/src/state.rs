use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Marketplace {
    pub admin: Pubkey,
    pub treasury: Pubkey,
    pub fee_bps: u16,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub maker: Pubkey,
    pub asset: Pubkey,
    pub collection: Pubkey,
    pub price: u64,
    pub payment_mint: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Offer {
    pub buyer: Pubkey,
    pub asset: Pubkey,
    pub amount: u64,
    pub bump: u8,
}
