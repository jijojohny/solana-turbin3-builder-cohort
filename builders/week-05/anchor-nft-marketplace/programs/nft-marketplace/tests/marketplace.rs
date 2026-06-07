mod common;

use common::*;
use solana_signer::Signer;

#[test]
fn test_initialize() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);

    let marketplace = load_marketplace(&svm);
    assert_eq!(marketplace.admin, setup.admin.pubkey());
    assert_eq!(marketplace.treasury, setup.treasury.pubkey());
    assert_eq!(marketplace.fee_bps, FEE_BPS);
}

#[test]
fn test_list_and_delist() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);

    list_sol(&mut svm, &setup, LIST_PRICE_SOL);
    let listing = load_listing(&svm, setup.asset.pubkey());
    assert_eq!(listing.maker, setup.maker.pubkey());
    assert_eq!(listing.price, LIST_PRICE_SOL);
    assert_eq!(listing.payment_mint, NATIVE_MINT);

    send(
        &mut svm,
        &setup.maker,
        &[&setup.maker],
        delist_ix(
            setup.maker.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
        ),
    );

    assert!(svm.get_account(&listing_pda(&setup.asset.pubkey())).is_none());
    assert_eq!(
        asset_owner(&svm, setup.asset.pubkey()),
        setup.maker.pubkey()
    );
}

#[test]
fn test_buy_with_sol() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);
    list_sol(&mut svm, &setup, LIST_PRICE_SOL);

    let maker_before = lamports(&svm, setup.maker.pubkey());
    let treasury_before = lamports(&svm, setup.treasury.pubkey());
    let listing_rent = lamports(&svm, listing_pda(&setup.asset.pubkey()));
    let (maker_share, fee) = split_payment(LIST_PRICE_SOL, FEE_BPS);

    send(
        &mut svm,
        &setup.buyer,
        &[&setup.buyer],
        buy_ix_with_treasury(
            setup.buyer.pubkey(),
            setup.maker.pubkey(),
            setup.treasury.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
        ),
    );

    assert_eq!(
        asset_owner(&svm, setup.asset.pubkey()),
        setup.buyer.pubkey()
    );
    assert_eq!(
        lamports(&svm, setup.maker.pubkey()),
        maker_before + maker_share + listing_rent
    );
    assert_eq!(
        lamports(&svm, setup.treasury.pubkey()),
        treasury_before + fee
    );
    assert!(svm.get_account(&listing_pda(&setup.asset.pubkey())).is_none());
}

#[test]
fn test_buy_with_token() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);
    list_token(&mut svm, &setup, LIST_PRICE_TOKEN);

    let maker_ata = ata(&setup.maker.pubkey(), &setup.payment_mint.pubkey());
    let treasury_ata = ata(&setup.treasury.pubkey(), &setup.payment_mint.pubkey());
    let buyer_ata = ata(&setup.buyer.pubkey(), &setup.payment_mint.pubkey());

    let maker_before = token_balance_or_zero(&svm, maker_ata);
    let treasury_before = token_balance_or_zero(&svm, treasury_ata);
    let buyer_before = token_balance(&svm, buyer_ata);
    let (maker_share, fee) = split_payment(LIST_PRICE_TOKEN, FEE_BPS);

    send(
        &mut svm,
        &setup.buyer,
        &[&setup.buyer],
        buy_with_token_ix(
            setup.buyer.pubkey(),
            setup.maker.pubkey(),
            setup.treasury.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
            setup.payment_mint.pubkey(),
        ),
    );

    assert_eq!(
        asset_owner(&svm, setup.asset.pubkey()),
        setup.buyer.pubkey()
    );
    assert_eq!(token_balance(&svm, maker_ata), maker_before + maker_share);
    assert_eq!(token_balance(&svm, treasury_ata), treasury_before + fee);
    assert_eq!(
        token_balance(&svm, buyer_ata),
        buyer_before - LIST_PRICE_TOKEN
    );
}

#[test]
fn test_make_accept_and_cancel_offer() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);

    send(
        &mut svm,
        &setup.buyer,
        &[&setup.buyer],
        make_offer_ix(setup.buyer.pubkey(), setup.asset.pubkey(), OFFER_AMOUNT),
    );

    let offer_key = offer_pda(&setup.asset.pubkey(), &setup.buyer.pubkey());
    assert!(svm.get_account(&offer_key).is_some());

    let buyer_before = lamports(&svm, setup.buyer.pubkey());
    send(
        &mut svm,
        &setup.buyer,
        &[&setup.buyer],
        cancel_offer_ix(setup.buyer.pubkey(), setup.asset.pubkey()),
    );
    assert!(svm.get_account(&offer_key).is_none());
    assert!(lamports(&svm, setup.buyer.pubkey()) > buyer_before);
}

#[test]
fn test_accept_offer() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);

    send(
        &mut svm,
        &setup.buyer,
        &[&setup.buyer],
        make_offer_ix(setup.buyer.pubkey(), setup.asset.pubkey(), OFFER_AMOUNT),
    );

    let maker_before = lamports(&svm, setup.maker.pubkey());
    let treasury_before = lamports(&svm, setup.treasury.pubkey());
    let (maker_share, fee) = split_payment(OFFER_AMOUNT, FEE_BPS);

    send(
        &mut svm,
        &setup.buyer,
        &[&setup.maker, &setup.buyer],
        accept_offer_ix(
            setup.maker.pubkey(),
            setup.buyer.pubkey(),
            setup.treasury.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
        ),
    );

    assert_eq!(
        asset_owner(&svm, setup.asset.pubkey()),
        setup.buyer.pubkey()
    );
    assert_eq!(
        lamports(&svm, setup.maker.pubkey()),
        maker_before + maker_share
    );
    assert_eq!(
        lamports(&svm, setup.treasury.pubkey()),
        treasury_before + fee
    );
    assert!(svm
        .get_account(&offer_pda(&setup.asset.pubkey(), &setup.buyer.pubkey()))
        .is_none());
}

#[test]
fn test_accept_offer_instead_of_list_price() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);
    list_sol(&mut svm, &setup, LIST_PRICE_SOL);

    send(
        &mut svm,
        &setup.buyer,
        &[&setup.buyer],
        make_offer_ix(setup.buyer.pubkey(), setup.asset.pubkey(), OFFER_AMOUNT),
    );

    send(
        &mut svm,
        &setup.buyer,
        &[&setup.maker, &setup.buyer],
        accept_offer_ix(
            setup.maker.pubkey(),
            setup.buyer.pubkey(),
            setup.treasury.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
        ),
    );

    assert_eq!(
        asset_owner(&svm, setup.asset.pubkey()),
        setup.buyer.pubkey()
    );
    assert!(svm.get_account(&listing_pda(&setup.asset.pubkey())).is_some());
}
