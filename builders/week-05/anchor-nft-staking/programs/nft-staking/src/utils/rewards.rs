use crate::constants::BPS_DENOMINATOR;
use crate::StakingError;
use anchor_lang::prelude::*;

pub fn reward_amount(elapsed_seconds: i64, rewards_bps: u16) -> Result<u64> {
    require!(elapsed_seconds > 0, StakingError::NoRewards);
    let elapsed = elapsed_seconds as u64;
    elapsed
        .checked_mul(rewards_bps as u64)
        .ok_or(StakingError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(StakingError::MathOverflow.into())
        .and_then(|amount| {
            require!(amount > 0, StakingError::NoRewards);
            Ok(amount)
        })
}
