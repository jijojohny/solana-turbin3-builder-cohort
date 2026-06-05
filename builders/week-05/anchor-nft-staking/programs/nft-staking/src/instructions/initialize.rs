use crate::constants::{
    CONFIG_SEED, REWARDS_DECIMALS, REWARDS_MINT_SEED, UPDATE_AUTHORITY_SEED,
};
use crate::state::StakeConfig;
use crate::utils::{initial_collection_attributes, load_collection};
use crate::StakingError;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use mpl_core::{
    instructions::{AddCollectionPluginV1CpiBuilder, UpdateCollectionV1CpiBuilder},
    types::{Plugin, PluginAuthority},
    ID as CORE_PROGRAM_ID,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: mpl-core collection, validated in handler
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + StakeConfig::INIT_SPACE,
        seeds = [CONFIG_SEED, collection.key().as_ref()],
        bump,
    )]
    pub config: Account<'info, StakeConfig>,

    #[account(
        init,
        payer = payer,
        seeds = [REWARDS_MINT_SEED, collection.key().as_ref()],
        bump,
        mint::decimals = REWARDS_DECIMALS,
        mint::authority = config,
    )]
    pub rewards_mint: Account<'info, Mint>,

    /// CHECK: update authority PDA
    #[account(
        seeds = [UPDATE_AUTHORITY_SEED, collection.key().as_ref()],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: mpl-core program
    pub core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<Initialize>, rewards_bps: u16, freeze_period: u16) -> Result<()> {
    let collection = load_collection(&ctx.accounts.collection.to_account_info())?;
    require!(
        collection.update_authority == ctx.accounts.admin.key(),
        StakingError::InvalidUpdateAuthority
    );

    let collection_key = ctx.accounts.collection.key();

    ctx.accounts.config.set_inner(StakeConfig {
        admin: ctx.accounts.admin.key(),
        collection: collection_key,
        rewards_mint: ctx.accounts.rewards_mint.key(),
        rewards_bps,
        freeze_period,
        bump: ctx.bumps.config,
        rewards_bump: ctx.bumps.rewards_mint,
    });

    AddCollectionPluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .collection(&ctx.accounts.collection.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.admin.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::Attributes(initial_collection_attributes()))
        .init_authority(PluginAuthority::UpdateAuthority)
        .invoke()?;

    UpdateCollectionV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .collection(&ctx.accounts.collection.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.admin.to_account_info()))
        .new_update_authority(Some(&ctx.accounts.authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .invoke()?;

    Ok(())
}
