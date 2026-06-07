use crate::constants::{MARKETPLACE_SEED, OFFER_SEED, OFFER_VAULT_SEED};
use crate::state::{Marketplace, Offer};
use crate::utils::{load_asset, pay_sol_from_pda, transfer_asset_to_buyer};
use crate::MarketplaceError;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use mpl_core::ID as CORE_PROGRAM_ID;

#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mut, address = offer.buyer)]
    pub buyer: Signer<'info>,

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

    #[account(
        mut,
        seeds = [OFFER_VAULT_SEED, asset.key().as_ref(), buyer.key().as_ref()],
        bump,
    )]
    /// CHECK: SOL escrow vault
    pub offer_vault: UncheckedAccount<'info>,

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
    let buyer_key = ctx.accounts.buyer.key();
    let vault_bump = ctx.bumps.offer_vault;
    let vault_seeds = &[
        OFFER_VAULT_SEED,
        asset_key.as_ref(),
        buyer_key.as_ref(),
        &[vault_bump],
    ];
    let signer = &[&vault_seeds[..]];

    pay_sol_from_pda(
        &ctx.accounts.offer_vault.to_account_info(),
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.treasury.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        offer.amount,
        ctx.accounts.marketplace.fee_bps,
        signer,
    )?;

    let vault_rent = ctx.accounts.offer_vault.lamports();
    if vault_rent > 0 {
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info().key(),
                Transfer {
                    from: ctx.accounts.offer_vault.to_account_info(),
                    to: ctx.accounts.buyer.to_account_info(),
                },
                signer,
            ),
            vault_rent,
        )?;
    }

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
