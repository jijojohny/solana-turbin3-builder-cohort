use crate::constants::{
    LAST_CLAIM_KEY, STAKED_AT_KEY, STAKED_COUNT_KEY, STAKED_FALSE, STAKED_KEY, STAKED_TRUE,
};
use crate::StakingError;
use anchor_lang::prelude::*;
use mpl_core::types::{Attribute, Attributes};

pub fn get_attr(attrs: &Attributes, key: &str) -> Option<String> {
    attrs
        .attribute_list
        .iter()
        .find(|a| a.key == key)
        .map(|a| a.value.clone())
}

pub fn upsert_attr(list: &mut Vec<Attribute>, key: &str, value: String) {
    if let Some(attr) = list.iter_mut().find(|a| a.key == key) {
        attr.value = value;
    } else {
        list.push(Attribute {
            key: key.to_string(),
            value,
        });
    }
}

pub fn build_attributes(list: Vec<Attribute>) -> Attributes {
    Attributes {
        attribute_list: list,
    }
}

pub fn parse_i64(value: &str) -> Result<i64> {
    value
        .parse::<i64>()
        .map_err(|_| StakingError::InvalidTimestamp.into())
}

pub fn parse_u64(value: &str) -> Result<u64> {
    value
        .parse::<u64>()
        .map_err(|_| StakingError::InvalidTimestamp.into())
}

pub fn is_staked(attrs: &Attributes) -> Result<bool> {
    match get_attr(attrs, STAKED_KEY) {
        Some(v) => Ok(v == STAKED_TRUE),
        None => Ok(false),
    }
}

pub fn require_not_staked(attrs: &Attributes) -> Result<()> {
    require!(!is_staked(attrs)?, StakingError::AlreadyStaked);
    Ok(())
}

pub fn require_staked(attrs: &Attributes) -> Result<(i64, i64)> {
    require!(is_staked(attrs)?, StakingError::NotStaked);
    let staked_at = parse_i64(&get_attr(attrs, STAKED_AT_KEY).ok_or(StakingError::NotStaked)?)?;
    let last_claim = parse_i64(
        &get_attr(attrs, LAST_CLAIM_KEY).unwrap_or_else(|| staked_at.to_string()),
    )?;
    Ok((staked_at, last_claim))
}

pub fn staking_asset_attributes(now: i64) -> Attributes {
    build_attributes(vec![
        Attribute {
            key: STAKED_KEY.to_string(),
            value: STAKED_TRUE.to_string(),
        },
        Attribute {
            key: STAKED_AT_KEY.to_string(),
            value: now.to_string(),
        },
        Attribute {
            key: LAST_CLAIM_KEY.to_string(),
            value: now.to_string(),
        },
    ])
}

pub fn unstaked_asset_attributes(existing: &Attributes) -> Result<Attributes> {
    let mut list: Vec<Attribute> = existing
        .attribute_list
        .iter()
        .filter(|a| a.key != STAKED_KEY && a.key != STAKED_AT_KEY && a.key != LAST_CLAIM_KEY)
        .cloned()
        .collect();
    upsert_attr(&mut list, STAKED_KEY, STAKED_FALSE.to_string());
    Ok(build_attributes(list))
}

pub fn claim_updated_attributes(existing: &Attributes, now: i64) -> Result<Attributes> {
    let mut list = existing.attribute_list.clone();
    require!(is_staked(&build_attributes(list.clone()))?, StakingError::NotStaked);
    upsert_attr(&mut list, LAST_CLAIM_KEY, now.to_string());
    Ok(build_attributes(list))
}

pub fn collection_staked_count(attrs: &Attributes) -> Result<u64> {
    Ok(parse_u64(
        &get_attr(attrs, STAKED_COUNT_KEY).unwrap_or_else(|| "0".to_string()),
    )?)
}

pub fn increment_staked_count(attrs: &Attributes) -> Result<Attributes> {
    let count = collection_staked_count(attrs)?;
    let mut list = attrs.attribute_list.clone();
    upsert_attr(
        &mut list,
        STAKED_COUNT_KEY,
        count
            .checked_add(1)
            .ok_or(StakingError::MathOverflow)?
            .to_string(),
    );
    Ok(build_attributes(list))
}

pub fn decrement_staked_count(attrs: &Attributes) -> Result<Attributes> {
    let count = collection_staked_count(attrs)?;
    require!(count > 0, StakingError::StakedCountUnderflow);
    let mut list = attrs.attribute_list.clone();
    upsert_attr(
        &mut list,
        STAKED_COUNT_KEY,
        count
            .checked_sub(1)
            .ok_or(StakingError::StakedCountUnderflow)?
            .to_string(),
    );
    Ok(build_attributes(list))
}

pub fn initial_collection_attributes() -> Attributes {
    build_attributes(vec![Attribute {
        key: STAKED_COUNT_KEY.to_string(),
        value: "0".to_string(),
    }])
}
