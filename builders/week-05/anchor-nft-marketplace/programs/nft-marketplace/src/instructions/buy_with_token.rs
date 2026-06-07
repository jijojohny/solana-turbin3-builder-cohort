use crate::constants::{LISTING_SEED, MARKETPLACE_SEED, NATIVE_MINT};
use crate::state::{Listing, Marketplace};
use crate::utils::{pay_tokens, transfer_asset_to_buyer};
use crate::MarketplaceError;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use mpl_core::ID as CORE_PROGRAM_ID;

#[derive(Accounts)]
pub struct BuyWithToken<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: receives sale proceeds
    #[account(mut, address = listing.maker)]
    pub maker: UncheckedAccount<'info>,

    #[account(
        seeds = [MARKETPLACE_SEED],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

    /// CHECK: treasury owner for ATA derivation
    #[account(address = marketplace.treasury)]
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

    #[account(
        address = listing.payment_mint,
        mint::token_program = token_program,
    )]
    pub payment_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_program,
    )]
    pub buyer_payment_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = payment_mint,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_payment_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = payment_mint,
        associated_token::authority = treasury,
        associated_token::token_program = token_program,
    )]
    pub treasury_payment_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: mpl-core program
    pub core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<BuyWithToken>) -> Result<()> {
    let listing = &ctx.accounts.listing;
    require!(
        listing.payment_mint != NATIVE_MINT,
        MarketplaceError::InvalidPaymentMint
    );
    require!(
        ctx.accounts.buyer.key() != listing.maker,
        MarketplaceError::CannotBuyOwnListing
    );

    pay_tokens(
        &ctx.accounts.buyer_payment_ata,
        &ctx.accounts.maker_payment_ata,
        &ctx.accounts.treasury_payment_ata,
        &ctx.accounts.payment_mint,
        &ctx.accounts.buyer,
        &ctx.accounts.token_program,
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
