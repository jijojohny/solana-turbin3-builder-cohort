use crate::constants::POOL_SEED;
use crate::math::swap_out;
use crate::state::Pool;
use crate::AmmError;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub trader: Signer<'info>,

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
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = pool,
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = pool,
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = trader,
    )]
    pub trader_ata_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = trader,
    )]
    pub trader_ata_b: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn handler_swap_a_for_b(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
    require!(amount_in > 0, AmmError::ZeroAmount);

    let amount_out = swap_out(
        amount_in,
        ctx.accounts.vault_a.amount,
        ctx.accounts.vault_b.amount,
        ctx.accounts.pool.fee_bps,
    )?;

    require!(amount_out >= min_amount_out, AmmError::SlippageExceeded);

    let pool = &ctx.accounts.pool;
    let seeds = &[
        POOL_SEED,
        pool.mint_a.as_ref(),
        pool.mint_b.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&seeds[..]];
    let token_program = ctx.accounts.token_program.key();

    token::transfer(
        CpiContext::new(
            token_program,
            Transfer {
                from: ctx.accounts.trader_ata_a.to_account_info(),
                to: ctx.accounts.vault_a.to_account_info(),
                authority: ctx.accounts.trader.to_account_info(),
            },
        ),
        amount_in,
    )?;

    token::transfer(
        CpiContext::new_with_signer(
            token_program,
            Transfer {
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.trader_ata_b.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer,
        ),
        amount_out,
    )?;

    Ok(())
}

pub fn handler_swap_b_for_a(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
    require!(amount_in > 0, AmmError::ZeroAmount);

    let amount_out = swap_out(
        amount_in,
        ctx.accounts.vault_b.amount,
        ctx.accounts.vault_a.amount,
        ctx.accounts.pool.fee_bps,
    )?;

    require!(amount_out >= min_amount_out, AmmError::SlippageExceeded);

    let pool = &ctx.accounts.pool;
    let seeds = &[
        POOL_SEED,
        pool.mint_a.as_ref(),
        pool.mint_b.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&seeds[..]];
    let token_program = ctx.accounts.token_program.key();

    token::transfer(
        CpiContext::new(
            token_program,
            Transfer {
                from: ctx.accounts.trader_ata_b.to_account_info(),
                to: ctx.accounts.vault_b.to_account_info(),
                authority: ctx.accounts.trader.to_account_info(),
            },
        ),
        amount_in,
    )?;

    token::transfer(
        CpiContext::new_with_signer(
            token_program,
            Transfer {
                from: ctx.accounts.vault_a.to_account_info(),
                to: ctx.accounts.trader_ata_a.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer,
        ),
        amount_out,
    )?;

    Ok(())
}
