use crate::constants::{CONFIG_SEED, UPDATE_AUTHORITY_SEED};
use crate::state::StakeConfig;
use crate::utils::{
    claim_updated_attributes, load_asset, load_collection, require_staked, reward_amount,
};
use crate::StakingError;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount},
};
use mpl_core::{
    accounts::BaseAssetV1,
    fetch_plugin,
    instructions::UpdatePluginV1CpiBuilder,
    types::{Plugin, PluginType, UpdateAuthority},
    ID as CORE_PROGRAM_ID,
};

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: mpl-core asset, validated in handler
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK: mpl-core collection, validated in handler
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,

    #[account(
        seeds = [CONFIG_SEED, collection.key().as_ref()],
        bump = config.bump,
        has_one = collection,
        has_one = rewards_mint,
    )]
    pub config: Account<'info, StakeConfig>,

    #[account(mut, address = config.rewards_mint)]
    pub rewards_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = rewards_mint,
        associated_token::authority = owner,
    )]
    pub owner_rewards_ata: Account<'info, TokenAccount>,

    /// CHECK: update authority PDA
    #[account(
        seeds = [UPDATE_AUTHORITY_SEED, collection.key().as_ref()],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: mpl-core program
    pub core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<ClaimRewards>) -> Result<()> {
    let asset = load_asset(&ctx.accounts.asset.to_account_info())?;
    let collection = load_collection(&ctx.accounts.collection.to_account_info())?;

    require!(asset.owner == ctx.accounts.owner.key(), StakingError::InvalidUpdateAuthority);
    require!(
        asset.update_authority == UpdateAuthority::Collection(ctx.accounts.collection.key()),
        StakingError::InvalidUpdateAuthority
    );
    require!(
        collection.update_authority == ctx.accounts.authority.key(),
        StakingError::InvalidUpdateAuthority
    );

    let now = Clock::get()?.unix_timestamp;

    let (_, asset_attrs, _) = fetch_plugin::<BaseAssetV1, mpl_core::types::Attributes>(
        &ctx.accounts.asset.to_account_info(),
        PluginType::Attributes,
    )
    .map_err(|_| StakingError::AttributesNotInitialized)?;

    let (_staked_at, last_claim_at) = require_staked(&asset_attrs)?;
    let elapsed = now
        .checked_sub(last_claim_at)
        .ok_or(StakingError::InvalidTimestamp)?;
    let amount = reward_amount(elapsed, ctx.accounts.config.rewards_bps)?;

    let config = &ctx.accounts.config;
    let config_seeds = &[
        CONFIG_SEED,
        config.collection.as_ref(),
        &[config.bump],
    ];
    let config_signer = &[&config_seeds[..]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            MintTo {
                mint: ctx.accounts.rewards_mint.to_account_info(),
                to: ctx.accounts.owner_rewards_ata.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            config_signer,
        ),
        amount,
    )?;

    let collection_key = ctx.accounts.collection.key();
    let authority_seeds = &[
        UPDATE_AUTHORITY_SEED,
        collection_key.as_ref(),
        &[ctx.bumps.authority],
    ];
    let authority_signer = &[&authority_seeds[..]];

    UpdatePluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::Attributes(claim_updated_attributes(
            &asset_attrs,
            now,
        )?))
        .invoke_signed(authority_signer)?;

    Ok(())
}
