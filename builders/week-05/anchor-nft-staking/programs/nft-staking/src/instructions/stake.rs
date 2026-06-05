use crate::constants::{CONFIG_SEED, UPDATE_AUTHORITY_SEED};
use crate::state::StakeConfig;
use crate::utils::{
    increment_staked_count, load_asset, load_collection, require_not_staked,
    staking_asset_attributes,
};
use crate::StakingError;
use anchor_lang::prelude::*;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{AddPluginV1CpiBuilder, UpdateCollectionPluginV1CpiBuilder, UpdatePluginV1CpiBuilder},
    types::{FreezeDelegate, Plugin, PluginAuthority, PluginType, UpdateAuthority},
    ID as CORE_PROGRAM_ID,
};

#[derive(Accounts)]
pub struct Stake<'info> {
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
    )]
    pub config: Account<'info, StakeConfig>,

    /// CHECK: update authority PDA
    #[account(
        seeds = [UPDATE_AUTHORITY_SEED, collection.key().as_ref()],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: mpl-core program
    pub core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<Stake>) -> Result<()> {
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
    let collection_key = ctx.accounts.collection.key();
    let authority_seeds = &[
        UPDATE_AUTHORITY_SEED,
        collection_key.as_ref(),
        &[ctx.bumps.authority],
    ];
    let signer = &[&authority_seeds[..]];

    match fetch_plugin::<BaseAssetV1, mpl_core::types::Attributes>(
        &ctx.accounts.asset.to_account_info(),
        PluginType::Attributes,
    ) {
        Ok((_, attrs, _)) => {
            require_not_staked(&attrs)?;
            UpdatePluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
                .asset(&ctx.accounts.asset.to_account_info())
                .collection(Some(&ctx.accounts.collection.to_account_info()))
                .payer(&ctx.accounts.payer.to_account_info())
                .authority(Some(&ctx.accounts.authority.to_account_info()))
                .system_program(&ctx.accounts.system_program.to_account_info())
                .plugin(Plugin::Attributes(staking_asset_attributes(now)))
                .invoke_signed(signer)?;
        }
        Err(_) => {
            AddPluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
                .asset(&ctx.accounts.asset.to_account_info())
                .collection(Some(&ctx.accounts.collection.to_account_info()))
                .payer(&ctx.accounts.payer.to_account_info())
                .authority(Some(&ctx.accounts.authority.to_account_info()))
                .system_program(&ctx.accounts.system_program.to_account_info())
                .plugin(Plugin::Attributes(staking_asset_attributes(now)))
                .init_authority(PluginAuthority::UpdateAuthority)
                .invoke_signed(signer)?;
        }
    }

    AddPluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.owner.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
        .init_authority(PluginAuthority::UpdateAuthority)
        .invoke()?;

    let (_, collection_attrs, _) = fetch_plugin::<BaseCollectionV1, mpl_core::types::Attributes>(
        &ctx.accounts.collection.to_account_info(),
        PluginType::Attributes,
    )
    .map_err(|_| StakingError::AttributesNotInitialized)?;

    UpdateCollectionPluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .collection(&ctx.accounts.collection.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::Attributes(increment_staked_count(&collection_attrs)?))
        .invoke_signed(signer)?;

    Ok(())
}
