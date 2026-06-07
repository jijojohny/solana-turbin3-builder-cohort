use crate::constants::{MARKETPLACE_SEED, OFFER_SEED};
use crate::state::{Marketplace, Offer};
use crate::utils::{load_asset, pay_sol_from_pda, transfer_asset_to_buyer};
use crate::MarketplaceError;
use anchor_lang::prelude::*;
use mpl_core::ID as CORE_PROGRAM_ID;

#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    /// CHECK: buyer receives the asset
    #[account(mut, address = offer.buyer)]
    pub buyer: UncheckedAccount<'info>,

    #[account(
        seeds = [MARKETPLACE_SEED],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

    /// CHECK: treasury receives fees
    #[account(mut, address = marketplace.treasury)]
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: mpl-core asset
    #[account(mut, address = offer.asset)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK: mpl-core collection
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,

    #[account(
        mut,
        close = buyer,
        seeds = [OFFER_SEED, asset.key().as_ref(), buyer.key().as_ref()],
        bump = offer.bump,
        has_one = asset,
        has_one = buyer,
    )]
    pub offer: Account<'info, Offer>,

    pub system_program: Program<'info, System>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: mpl-core program
    pub core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<AcceptOffer>) -> Result<()> {
    let asset = load_asset(&ctx.accounts.asset.to_account_info())?;
    require!(
        asset.owner == ctx.accounts.maker.key(),
        MarketplaceError::UnauthorizedMaker
    );

    let offer = &ctx.accounts.offer;
    let asset_key = ctx.accounts.asset.key();
    let offer_seeds = &[
        OFFER_SEED,
        asset_key.as_ref(),
        offer.buyer.as_ref(),
        &[offer.bump],
    ];
    let signer = &[&offer_seeds[..]];

    pay_sol_from_pda(
        &ctx.accounts.offer.to_account_info(),
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.treasury.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        offer.amount,
        ctx.accounts.marketplace.fee_bps,
        signer,
    )?;

    transfer_asset_to_buyer(
        &ctx.accounts.asset.to_account_info(),
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.buyer.to_account_info(),
        &ctx.accounts.core_program.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        None,
    )?;

    Ok(())
}
