mod common;

use common::*;
use solana_signer::Signer;

#[test]
fn test_buy_own_listing_fails() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);
    list_sol(&mut svm, &setup, LIST_PRICE_SOL);

    send_expect_err(
        &mut svm,
        &setup.maker,
        &[&setup.maker],
        buy_ix_with_treasury(
            setup.maker.pubkey(),
            setup.maker.pubkey(),
            setup.treasury.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
        ),
    );
}

#[test]
fn test_delist_unauthorized_fails() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);
    list_sol(&mut svm, &setup, LIST_PRICE_SOL);

    send_expect_err(
        &mut svm,
        &setup.buyer,
        &[&setup.buyer],
        delist_ix(
            setup.buyer.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
        ),
    );
}

#[test]
fn test_buy_with_token_on_sol_listing_fails() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);
    list_sol(&mut svm, &setup, LIST_PRICE_SOL);

    send_expect_err(
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
}

#[test]
fn test_buy_with_sol_on_token_listing_fails() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);
    list_token(&mut svm, &setup, LIST_PRICE_TOKEN);

    send_expect_err(
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
}

#[test]
fn test_make_offer_on_own_asset_fails() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);

    send_expect_err(
        &mut svm,
        &setup.maker,
        &[&setup.maker],
        make_offer_ix(setup.maker.pubkey(), setup.asset.pubkey(), OFFER_AMOUNT),
    );
}

#[test]
fn test_cancel_offer_unauthorized_fails() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);

    send(
        &mut svm,
        &setup.buyer,
        &[&setup.buyer],
        make_offer_ix(setup.buyer.pubkey(), setup.asset.pubkey(), OFFER_AMOUNT),
    );

    send_expect_err(
        &mut svm,
        &setup.maker,
        &[&setup.maker],
        cancel_offer_ix(setup.maker.pubkey(), setup.asset.pubkey()),
    );
}

#[test]
fn test_double_list_fails() {
    let mut svm = setup_svm();
    let setup = setup_marketplace(&mut svm);
    list_sol(&mut svm, &setup, LIST_PRICE_SOL);

    send_expect_err(
        &mut svm,
        &setup.maker,
        &[&setup.maker],
        list_ix(
            setup.maker.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
            LIST_PRICE_SOL,
            NATIVE_MINT,
        ),
    );
}
