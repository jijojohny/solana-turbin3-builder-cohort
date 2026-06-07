use anchor_lang::prelude::*;

#[error_code]
pub enum MarketplaceError {
    #[msg("invalid asset owner")]
    InvalidOwner,
    #[msg("listing already exists for this asset")]
    ListingAlreadyExists,
    #[msg("listing not found")]
    ListingNotFound,
    #[msg("unauthorized maker")]
    UnauthorizedMaker,
    #[msg("unauthorized buyer")]
    UnauthorizedBuyer,
    #[msg("invalid payment mint for this instruction")]
    InvalidPaymentMint,
    #[msg("offer already exists")]
    OfferAlreadyExists,
    #[msg("offer not found")]
    OfferNotFound,
    #[msg("cannot buy your own listing")]
    CannotBuyOwnListing,
    #[msg("cannot offer on your own asset")]
    CannotOfferOnOwnAsset,
    #[msg("arithmetic overflow")]
    MathOverflow,
    #[msg("invalid marketplace fee")]
    InvalidFee,
    #[msg("invalid asset for collection")]
    InvalidAsset,
}
