use crate::constants::POOL_SEED;
use crate::math::withdraw_amounts;
use crate::state::Pool;
use crate::utils::parse_previous_burn_lp;
use crate::AmmError;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use solana_instructions_sysvar::ID as INSTRUCTIONS_SYSVAR_ID;

#[derive(Accounts)]
pub struct WithdrawPayout<'info> {
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
        associated_token::authority = depositor,
    )]
    pub depositor_ata_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = depositor,
    )]
    pub depositor_ata_b: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = depositor,
    )]
    pub depositor_lp_ata: Box<Account<'info, TokenAccount>>,

    /// CHECK: instructions sysvar for burn_lp introspection
    #[account(address = INSTRUCTIONS_SYSVAR_ID @ AmmError::InvalidBurnInstruction)]
    pub instruction_sysvar: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<WithdrawPayout>,
    min_amount_a: u64,
    min_amount_b: u64,
) -> Result<()> {
    let burned_lp = parse_previous_burn_lp(
        &ctx.accounts.instruction_sysvar.to_account_info(),
        ctx.accounts.depositor.key(),
        ctx.accounts.pool.key(),
        ctx.accounts.lp_mint.key(),
        ctx.accounts.depositor_lp_ata.key(),
    )?;

    let reserve_a = ctx.accounts.vault_a.amount;
    let reserve_b = ctx.accounts.vault_b.amount;
    let lp_supply_after = ctx.accounts.lp_mint.supply;
    let lp_supply_before = lp_supply_after
        .checked_add(burned_lp)
        .ok_or(AmmError::MathOverflow)?;

    require!(lp_supply_before > 0, AmmError::InsufficientLiquidity);

    let (amount_a, amount_b) =
        withdraw_amounts(burned_lp, reserve_a, reserve_b, lp_supply_before)?;

    require!(amount_a >= min_amount_a, AmmError::SlippageExceeded);
    require!(amount_b >= min_amount_b, AmmError::SlippageExceeded);
    require!(amount_a > 0 && amount_b > 0, AmmError::ZeroAmount);

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
        CpiContext::new_with_signer(
            token_program,
            Transfer {
                from: ctx.accounts.vault_a.to_account_info(),
                to: ctx.accounts.depositor_ata_a.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer,
        ),
        amount_a,
    )?;

    token::transfer(
        CpiContext::new_with_signer(
            token_program,
            Transfer {
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.depositor_ata_b.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer,
        ),
        amount_b,
    )?;

    Ok(())
}
