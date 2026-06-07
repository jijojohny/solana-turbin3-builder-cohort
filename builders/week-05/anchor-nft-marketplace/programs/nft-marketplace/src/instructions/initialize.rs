use crate::constants::MARKETPLACE_SEED;
use crate::state::Marketplace;
use crate::MarketplaceError;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + Marketplace::INIT_SPACE,
        seeds = [MARKETPLACE_SEED],
        bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

    /// CHECK: treasury receives marketplace fees (SOL or ATA owner)
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, fee_bps: u16) -> Result<()> {
    require!(fee_bps <= 10_000, MarketplaceError::InvalidFee);

    ctx.accounts.marketplace.set_inner(Marketplace {
        admin: ctx.accounts.admin.key(),
        treasury: ctx.accounts.treasury.key(),
        fee_bps,
        bump: ctx.bumps.marketplace,
    });

    Ok(())
}
