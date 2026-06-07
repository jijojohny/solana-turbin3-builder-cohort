use anchor_lang::prelude::*;
use mpl_core::accounts::{BaseAssetV1, BaseCollectionV1};

pub fn load_asset(info: &AccountInfo) -> Result<BaseAssetV1> {
    BaseAssetV1::from_bytes(&info.data.borrow()).map_err(Into::into)
}

pub fn load_collection(info: &AccountInfo) -> Result<BaseCollectionV1> {
    BaseCollectionV1::from_bytes(&info.data.borrow()).map_err(Into::into)
}
