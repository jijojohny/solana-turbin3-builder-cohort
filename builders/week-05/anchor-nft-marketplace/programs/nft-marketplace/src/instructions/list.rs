use crate::constants::{LISTING_SEED, MARKETPLACE_SEED, NATIVE_MINT};
use crate::state::{Listing, Marketplace};
use crate::utils::{
    add_transfer_delegate, approve_listing_delegate, load_asset, validate_asset_in_collection,
};
use crate::MarketplaceError;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenInterface;
use mpl_core::ID as CORE_PROGRAM_ID;

#[derive(Accounts)]
pub struct List<'info> {
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
        init,
        payer = maker,
        space = 8 + Listing::INIT_SPACE,
        seeds = [LISTING_SEED, asset.key().as_ref()],
        bump,
    )]
    pub listing: Account<'info, Listing>,

    /// CHECK: SPL mint when listing is token-denominated
    pub payment_mint: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: mpl-core program
    pub core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<List>, price: u64, payment_mint: Pubkey) -> Result<()> {
    require!(price > 0, MarketplaceError::MathOverflow);

    let asset = load_asset(&ctx.accounts.asset.to_account_info())?;
    validate_asset_in_collection(
        &asset,
        ctx.accounts.collection.key(),
        ctx.accounts.maker.key(),
    )?;

    if payment_mint != NATIVE_MINT {
        require_keys_eq!(ctx.accounts.payment_mint.key(), payment_mint);
    }

    let asset_key = ctx.accounts.asset.key();
    ctx.accounts.listing.set_inner(Listing {
        maker: ctx.accounts.maker.key(),
        asset: asset_key,
        collection: ctx.accounts.collection.key(),
        price,
        payment_mint,
        bump: ctx.bumps.listing,
    });

    add_transfer_delegate(
        &ctx.accounts.asset.to_account_info(),
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.core_program.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
    )?;

    approve_listing_delegate(
        &ctx.accounts.asset.to_account_info(),
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.listing.to_account_info(),
        &ctx.accounts.core_program.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
    )?;

    Ok(())
}
