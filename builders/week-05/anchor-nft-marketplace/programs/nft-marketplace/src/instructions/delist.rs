use crate::constants::{LISTING_SEED, MARKETPLACE_SEED};
use crate::state::{Listing, Marketplace};
use crate::utils::{remove_transfer_delegate, revoke_listing_delegate};
use anchor_lang::prelude::*;
use mpl_core::ID as CORE_PROGRAM_ID;

#[derive(Accounts)]
pub struct Delist<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        seeds = [MARKETPLACE_SEED],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

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
        has_one = maker,
        has_one = asset,
    )]
    pub listing: Account<'info, Listing>,

    pub system_program: Program<'info, System>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: mpl-core program
    pub core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<Delist>) -> Result<()> {
    let listing = &ctx.accounts.listing;
    let asset_key = ctx.accounts.asset.key();
    let listing_seeds = &[
        LISTING_SEED,
        asset_key.as_ref(),
        &[listing.bump],
    ];
    let signer = &[&listing_seeds[..]];

    revoke_listing_delegate(
        &ctx.accounts.asset.to_account_info(),
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.listing.to_account_info(),
        &ctx.accounts.core_program.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        signer,
    )?;

    remove_transfer_delegate(
        &ctx.accounts.asset.to_account_info(),
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.core_program.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
    )?;

    Ok(())
}
