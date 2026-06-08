use crate::constants::POOL_SEED;
use crate::state::Pool;
use crate::AmmError;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct BurnLp<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        constraint = mint_a.key() < mint_b.key() @ AmmError::InvalidMintOrder,
    )]
    pub mint_a: Box<Account<'info, Mint>>,

    pub mint_b: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [POOL_SEED, mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump = pool.bump,
        has_one = mint_a,
        has_one = mint_b,
        has_one = lp_mint,
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut, address = pool.lp_mint)]
    pub lp_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = depositor,
    )]
    pub depositor_lp_ata: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<BurnLp>, lp_amount: u64) -> Result<()> {
    require!(lp_amount > 0, AmmError::ZeroAmount);

    let lp_supply = ctx.accounts.lp_mint.supply;
    require!(lp_supply > 0, AmmError::InsufficientLiquidity);
    require!(lp_amount <= lp_supply, AmmError::InsufficientLiquidity);

    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.depositor_lp_ata.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        lp_amount,
    )?;

    Ok(())
}
