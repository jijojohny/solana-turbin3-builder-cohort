use crate::constants::{MARKETPLACE_SEED, OFFER_SEED, OFFER_VAULT_SEED};
use crate::state::{Marketplace, Offer};
use crate::utils::load_asset;
use crate::MarketplaceError;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{create_account, transfer, CreateAccount, Transfer};

#[derive(Accounts)]
pub struct MakeOffer<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        seeds = [MARKETPLACE_SEED],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

    /// CHECK: mpl-core asset being offered on
    pub asset: UncheckedAccount<'info>,

    #[account(
        init,
        payer = buyer,
        space = 8 + Offer::INIT_SPACE,
        seeds = [OFFER_SEED, asset.key().as_ref(), buyer.key().as_ref()],
        bump,
    )]
    pub offer: Account<'info, Offer>,

    /// CHECK: zero-data PDA holding escrowed SOL
    #[account(
        mut,
        seeds = [OFFER_VAULT_SEED, asset.key().as_ref(), buyer.key().as_ref()],
        bump,
    )]
    pub offer_vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<MakeOffer>, amount: u64) -> Result<()> {
    require!(amount > 0, MarketplaceError::MathOverflow);

    let asset = load_asset(&ctx.accounts.asset.to_account_info())?;
    require!(
        asset.owner != ctx.accounts.buyer.key(),
        MarketplaceError::CannotOfferOnOwnAsset
    );

    ctx.accounts.offer.set_inner(Offer {
        buyer: ctx.accounts.buyer.key(),
        asset: ctx.accounts.asset.key(),
        amount,
        bump: ctx.bumps.offer,
    });

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

    if ctx.accounts.offer_vault.lamports() == 0 {
        create_account(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.key(),
                CreateAccount {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.offer_vault.to_account_info(),
                },
                signer,
            ),
            Rent::get()?.minimum_balance(0),
            0,
            &anchor_lang::system_program::ID,
        )?;
    }

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.offer_vault.to_account_info(),
            },
        ),
        amount,
    )?;

    Ok(())
}
