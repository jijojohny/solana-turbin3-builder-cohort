pub const MARKETPLACE_SEED: &[u8] = b"marketplace";
pub const LISTING_SEED: &[u8] = b"listing";
pub const OFFER_VAULT_SEED: &[u8] = b"offer_vault";
pub const OFFER_SEED: &[u8] = b"offer";

pub const BPS_DENOMINATOR: u64 = 10_000;

/// System program id marks native SOL listings.
pub const NATIVE_MINT: anchor_lang::prelude::Pubkey =
    anchor_lang::prelude::pubkey!("11111111111111111111111111111111");
