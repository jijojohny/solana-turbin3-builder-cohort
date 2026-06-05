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

declare_id!("9FAFuiroTiq89GaxGpYTVDQq1AbD3whjLpvWB43uhGGZ");

#[program]
pub mod nft_staking {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, rewards_bps: u16, freeze_period: u16) -> Result<()> {
        instructions::initialize::handler(ctx, rewards_bps, freeze_period)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        instructions::stake::handler(ctx)
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        instructions::claim_rewards::handler(ctx)
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        instructions::unstake::handler(ctx)
    }
}
