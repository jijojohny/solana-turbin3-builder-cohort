use crate::constants::{LISTING_SEED, MARKETPLACE_SEED, NATIVE_MINT};
use crate::state::{Listing, Marketplace};
use crate::utils::{pay_sol, transfer_asset_to_buyer};
use crate::MarketplaceError;
use anchor_lang::prelude::*;
use mpl_core::ID as CORE_PROGRAM_ID;

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: receives sale proceeds
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,

    #[account(
        seeds = [MARKETPLACE_SEED],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

    /// CHECK: treasury receives fees
    #[account(mut, address = marketplace.treasury)]
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: mpl-core asset
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK: mpl-core collection
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,

    #[account(
        mut,
        close = maker,
        seeds = [LISTING_SEED, asset.key().as_ref()],
        bump = listing.bump,
        has_one = asset,
        has_one = maker,
    )]
    pub listing: Account<'info, Listing>,

    pub system_program: Program<'info, System>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: mpl-core program
    pub core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<Buy>) -> Result<()> {
    let listing = &ctx.accounts.listing;
    require!(
        listing.payment_mint == NATIVE_MINT,
        MarketplaceError::InvalidPaymentMint
    );
    require!(
        ctx.accounts.buyer.key() != listing.maker,
        MarketplaceError::CannotBuyOwnListing
    );

    pay_sol(
        &ctx.accounts.buyer.to_account_info(),
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.treasury.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        listing.price,
        ctx.accounts.marketplace.fee_bps,
    )?;

    let asset_key = ctx.accounts.asset.key();
    let listing_seeds = &[
        LISTING_SEED,
        asset_key.as_ref(),
        &[listing.bump],
    ];
    let signer = &[&listing_seeds[..]];

    transfer_asset_to_buyer(
        &ctx.accounts.asset.to_account_info(),
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.buyer.to_account_info(),
        &ctx.accounts.listing.to_account_info(),
        &ctx.accounts.buyer.to_account_info(),
        &ctx.accounts.core_program.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        Some(signer),
    )?;

    Ok(())
}
