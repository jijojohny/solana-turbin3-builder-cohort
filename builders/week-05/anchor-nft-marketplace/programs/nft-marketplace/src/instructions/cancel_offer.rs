use crate::constants::{OFFER_SEED, OFFER_VAULT_SEED};
use crate::state::Offer;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: mpl-core asset
    pub asset: UncheckedAccount<'info>,

    #[account(
        mut,
        close = buyer,
        seeds = [OFFER_SEED, asset.key().as_ref(), buyer.key().as_ref()],
        bump = offer.bump,
        has_one = buyer,
        has_one = asset,
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
}

pub fn handler(ctx: Context<CancelOffer>) -> Result<()> {
    let vault_lamports = ctx.accounts.offer_vault.lamports();
    if vault_lamports == 0 {
        return Ok(());
    }

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

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info().key(),
            Transfer {
                from: ctx.accounts.offer_vault.to_account_info(),
                to: ctx.accounts.buyer.to_account_info(),
            },
            signer,
        ),
        vault_lamports,
    )?;
    Ok(())
}
