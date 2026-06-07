pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use instructions::*;
pub use state::*;

declare_id!("9XKHUKki3mBjhkFrwvFqwdenTCykw8FH3YYdqT7f2Uru");

#[program]
pub mod nft_marketplace {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, fee_bps: u16) -> Result<()> {
        instructions::initialize::handler(ctx, fee_bps)
    }

    pub fn list(ctx: Context<List>, price: u64, payment_mint: Pubkey) -> Result<()> {
        instructions::list::handler(ctx, price, payment_mint)
    }

    pub fn delist(ctx: Context<Delist>) -> Result<()> {
        instructions::delist::handler(ctx)
    }

    pub fn buy(ctx: Context<Buy>) -> Result<()> {
        instructions::buy::handler(ctx)
    }

    pub fn buy_with_token(ctx: Context<BuyWithToken>) -> Result<()> {
        instructions::buy_with_token::handler(ctx)
    }

    pub fn make_offer(ctx: Context<MakeOffer>, amount: u64) -> Result<()> {
        instructions::make_offer::handler(ctx, amount)
    }

    pub fn accept_offer(ctx: Context<AcceptOffer>) -> Result<()> {
        instructions::accept_offer::handler(ctx)
    }

    pub fn cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
        instructions::cancel_offer::handler(ctx)
    }
}
