use crate::constants::POOL_SEED;
use crate::math::{deposit_amounts, deposit_lp, initial_lp};
use crate::state::Pool;
use crate::AmmError;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct Deposit<'info> {
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
        init_if_needed,
        payer = depositor,
        associated_token::mint = lp_mint,
        associated_token::authority = depositor,
    )]
    pub depositor_lp_ata: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Deposit>,
    amount_a: u64,
    amount_b: u64,
    min_lp_out: u64,
) -> Result<()> {
    require!(amount_a > 0 && amount_b > 0, AmmError::ZeroAmount);

    let reserve_a = ctx.accounts.vault_a.amount;
    let reserve_b = ctx.accounts.vault_b.amount;
    let lp_supply = ctx.accounts.lp_mint.supply;

    let lp_amount = if lp_supply == 0 {
        initial_lp(amount_a, amount_b)?
    } else {
        deposit_lp(amount_a, amount_b, reserve_a, reserve_b, lp_supply)?
    };

    require!(lp_amount >= min_lp_out, AmmError::SlippageExceeded);
    require!(lp_amount > 0, AmmError::ZeroAmount);

    let (transfer_a, transfer_b) = if lp_supply == 0 {
        (amount_a, amount_b)
    } else {
        deposit_amounts(lp_amount, reserve_a, reserve_b, lp_supply)?
    };

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
                from: ctx.accounts.depositor_ata_a.to_account_info(),
                to: ctx.accounts.vault_a.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        transfer_a,
    )?;

    token::transfer(
        CpiContext::new(
            token_program,
            Transfer {
                from: ctx.accounts.depositor_ata_b.to_account_info(),
                to: ctx.accounts.vault_b.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        transfer_b,
    )?;

    if transfer_a < amount_a {
        let refund = amount_a - transfer_a;
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
            refund,
        )?;
    }

    if transfer_b < amount_b {
        let refund = amount_b - transfer_b;
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
            refund,
        )?;
    }

    token::mint_to(
        CpiContext::new_with_signer(
            token_program,
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.depositor_lp_ata.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer,
        ),
        lp_amount,
    )?;

    Ok(())
}
