use crate::constants::{OFFER_SEED, OFFER_VAULT_SEED};
use crate::state::Offer;
use anchor_lang::prelude::*;

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
    if vault_lamports > 0 {
        ctx.accounts.offer_vault.sub_lamports(vault_lamports)?;
        ctx.accounts.buyer.add_lamports(vault_lamports)?;
    }
    Ok(())
}
