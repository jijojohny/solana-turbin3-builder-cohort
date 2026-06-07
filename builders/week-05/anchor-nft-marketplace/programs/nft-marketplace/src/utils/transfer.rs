use anchor_lang::prelude::*;
use mpl_core::{
    accounts::BaseAssetV1,
    fetch_plugin,
    instructions::{
        AddPluginV1CpiBuilder, ApprovePluginAuthorityV1CpiBuilder, RemovePluginV1CpiBuilder,
        RevokePluginAuthorityV1CpiBuilder, TransferV1CpiBuilder,
    },
    types::{Plugin, PluginAuthority, PluginType, TransferDelegate, UpdateAuthority},
};

pub fn add_transfer_delegate<'info>(
    asset: &AccountInfo<'info>,
    collection: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    owner: &AccountInfo<'info>,
    core_program: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
) -> Result<()> {
    match fetch_plugin::<BaseAssetV1, TransferDelegate>(asset, PluginType::TransferDelegate) {
        Ok(_) => Ok(()),
        Err(_) => AddPluginV1CpiBuilder::new(core_program)
            .asset(asset)
            .collection(Some(collection))
            .payer(payer)
            .authority(Some(owner))
            .system_program(system_program)
            .plugin(Plugin::TransferDelegate(TransferDelegate {}))
            .init_authority(PluginAuthority::Owner)
            .invoke()
            .map_err(Into::into),
    }
}

pub fn approve_listing_delegate<'info>(
    asset: &AccountInfo<'info>,
    collection: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    owner: &AccountInfo<'info>,
    listing: &AccountInfo<'info>,
    core_program: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
) -> Result<()> {
    ApprovePluginAuthorityV1CpiBuilder::new(core_program)
        .asset(asset)
        .collection(Some(collection))
        .payer(payer)
        .authority(Some(owner))
        .system_program(system_program)
        .plugin_type(PluginType::TransferDelegate)
        .new_authority(PluginAuthority::Address {
            address: listing.key(),
        })
        .invoke()
        .map_err(Into::into)
}

pub fn revoke_listing_delegate<'info>(
    asset: &AccountInfo<'info>,
    collection: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    listing: &AccountInfo<'info>,
    core_program: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    RevokePluginAuthorityV1CpiBuilder::new(core_program)
        .asset(asset)
        .collection(Some(collection))
        .payer(payer)
        .authority(Some(listing))
        .system_program(system_program)
        .plugin_type(PluginType::TransferDelegate)
        .invoke_signed(signer_seeds)
        .map_err(Into::into)
}

pub fn remove_transfer_delegate<'info>(
    asset: &AccountInfo<'info>,
    collection: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    owner: &AccountInfo<'info>,
    core_program: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
) -> Result<()> {
    if fetch_plugin::<BaseAssetV1, TransferDelegate>(asset, PluginType::TransferDelegate).is_ok() {
        return RemovePluginV1CpiBuilder::new(core_program)
            .asset(asset)
            .collection(Some(collection))
            .payer(payer)
            .authority(Some(owner))
            .system_program(system_program)
            .plugin_type(PluginType::TransferDelegate)
            .invoke()
            .map_err(Into::into);
    }
    Ok(())
}

pub fn transfer_asset_to_buyer<'info>(
    asset: &AccountInfo<'info>,
    collection: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    new_owner: &AccountInfo<'info>,
    core_program: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let mut cpi = TransferV1CpiBuilder::new(core_program);
    let builder = cpi
        .asset(asset)
        .collection(Some(collection))
        .payer(payer)
        .authority(Some(authority))
        .new_owner(new_owner)
        .system_program(Some(system_program));

    match signer_seeds {
        Some(seeds) => builder.invoke_signed(seeds).map_err(Into::into),
        None => builder.invoke().map_err(Into::into),
    }
}

pub fn validate_asset_in_collection(
    asset: &BaseAssetV1,
    collection_key: Pubkey,
    owner: Pubkey,
) -> Result<()> {
    require!(asset.owner == owner, crate::MarketplaceError::InvalidOwner);
    require!(
        asset.update_authority == UpdateAuthority::Collection(collection_key),
        crate::MarketplaceError::InvalidAsset
    );
    Ok(())
}
