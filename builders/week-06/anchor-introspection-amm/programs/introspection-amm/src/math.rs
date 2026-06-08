use crate::constants::BPS_DENOMINATOR;
use crate::AmmError;
use anchor_lang::prelude::*;

pub fn initial_lp(amount_a: u64, amount_b: u64) -> Result<u64> {
    let product = (amount_a as u128)
        .checked_mul(amount_b as u128)
        .ok_or(AmmError::MathOverflow)?;
    let lp = integer_sqrt(product);
    require!(lp > 0, AmmError::ZeroAmount);
    Ok(lp as u64)
}

pub fn deposit_lp(
    amount_a: u64,
    amount_b: u64,
    reserve_a: u64,
    reserve_b: u64,
    lp_supply: u64,
) -> Result<u64> {
    let lp_a = (amount_a as u128)
        .checked_mul(lp_supply as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(reserve_a as u128)
        .ok_or(AmmError::MathOverflow)?;

    let lp_b = (amount_b as u128)
        .checked_mul(lp_supply as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(reserve_b as u128)
        .ok_or(AmmError::MathOverflow)?;

    Ok(lp_a.min(lp_b) as u64)
}

pub fn deposit_amounts(
    lp_amount: u64,
    reserve_a: u64,
    reserve_b: u64,
    lp_supply: u64,
) -> Result<(u64, u64)> {
    let amount_a = (lp_amount as u128)
        .checked_mul(reserve_a as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(lp_supply as u128)
        .ok_or(AmmError::MathOverflow)? as u64;

    let amount_b = (lp_amount as u128)
        .checked_mul(reserve_b as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(lp_supply as u128)
        .ok_or(AmmError::MathOverflow)? as u64;

    Ok((amount_a, amount_b))
}

pub fn withdraw_amounts(
    lp_amount: u64,
    reserve_a: u64,
    reserve_b: u64,
    lp_supply: u64,
) -> Result<(u64, u64)> {
    deposit_amounts(lp_amount, reserve_a, reserve_b, lp_supply)
}

pub fn swap_out(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_bps: u16,
) -> Result<u64> {
    require!(
        reserve_in > 0 && reserve_out > 0,
        AmmError::InsufficientLiquidity
    );

    let amount_in_eff = (amount_in as u128)
        .checked_mul((BPS_DENOMINATOR - fee_bps as u64) as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR as u128)
        .ok_or(AmmError::MathOverflow)?;

    let numerator = (reserve_out as u128)
        .checked_mul(amount_in_eff)
        .ok_or(AmmError::MathOverflow)?;

    let denominator = (reserve_in as u128)
        .checked_add(amount_in_eff)
        .ok_or(AmmError::MathOverflow)?;

    let amount_out = numerator
        .checked_div(denominator)
        .ok_or(AmmError::MathOverflow)? as u64;

    require!(amount_out > 0, AmmError::ZeroAmount);
    Ok(amount_out)
}

fn integer_sqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
