pub mod constants;
pub mod error;
pub mod instructions;
pub mod math;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use instructions::*;
pub use state::*;

declare_id!("2QMKjmtkFRnDQJihz9DEnNBFGoUqEXqXc1kQBVjrvmyB");

#[program]
pub mod introspection_amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, fee_bps: u16) -> Result<()> {
        instructions::initialize::handler(ctx, fee_bps)
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        amount_a: u64,
        amount_b: u64,
        min_lp_out: u64,
    ) -> Result<()> {
        instructions::deposit::deposit_handler(ctx, amount_a, amount_b, min_lp_out)
    }

    pub fn burn_lp(ctx: Context<BurnLp>, lp_amount: u64) -> Result<()> {
        instructions::burn_lp::handler(ctx, lp_amount)
    }

    pub fn withdraw_payout(
        ctx: Context<WithdrawPayout>,
        min_amount_a: u64,
        min_amount_b: u64,
    ) -> Result<()> {
        instructions::withdraw_payout::handler(ctx, min_amount_a, min_amount_b)
    }

    pub fn swap_a_for_b(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
        instructions::swap::handler_swap_a_for_b(ctx, amount_in, min_amount_out)
    }

    pub fn swap_b_for_a(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
        instructions::swap::handler_swap_b_for_a(ctx, amount_in, min_amount_out)
    }
}
