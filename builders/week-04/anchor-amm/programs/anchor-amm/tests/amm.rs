mod common;

use anchor_amm::state::Pool;
use anchor_lang::AccountDeserialize;
use common::*;
use solana_keypair::Keypair;
use solana_signer::Signer;

#[test]
fn test_initialize() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS).unwrap();

    let setup = setup_initialized_pool(&mut svm, &payer, &payer.pubkey());

    let pool_account = svm.get_account(&setup.pool).expect("pool missing");
    let pool = Pool::try_deserialize(&mut pool_account.data.as_ref()).unwrap();
    assert_eq!(pool.admin, payer.pubkey());
    assert_eq!(pool.mint_a, setup.mint_a);
    assert_eq!(pool.mint_b, setup.mint_b);
    assert_eq!(pool.lp_mint, setup.lp_mint);
    assert_eq!(pool.fee_bps, FEE_BPS);
    assert_eq!(token_balance(&svm, setup.vault_a), 0);
    assert_eq!(token_balance(&svm, setup.vault_b), 0);
}

#[test]
fn test_deposit() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let depositor = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&depositor.pubkey(), LAMPORTS).unwrap();

    let setup = setup_initialized_pool(&mut svm, &payer, &payer.pubkey());
    setup_user_tokens(
        &mut svm,
        &depositor.pubkey(),
        setup.mint_a,
        setup.mint_b,
        INITIAL_A,
        INITIAL_B,
    );

    send(
        &mut svm,
        &depositor,
        &[&depositor],
        deposit_ix(
            depositor.pubkey(),
            setup.mint_a,
            setup.mint_b,
            INITIAL_A,
            INITIAL_B,
            1,
        ),
    );

    assert_eq!(token_balance(&svm, setup.vault_a), INITIAL_A);
    assert_eq!(token_balance(&svm, setup.vault_b), INITIAL_B);
    assert!(mint_supply(&svm, setup.lp_mint) > 0);
    assert!(token_balance(&svm, ata(&depositor.pubkey(), &setup.lp_mint)) > 0);
}

#[test]
fn test_swap_a_for_b() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let trader = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&trader.pubkey(), LAMPORTS).unwrap();

    let setup = setup_pool_with_liquidity(&mut svm, &payer, &payer);
    setup_user_tokens(
        &mut svm,
        &trader.pubkey(),
        setup.mint_a,
        setup.mint_b,
        SWAP_IN,
        0,
    );

    let b_before = token_balance(&svm, ata(&trader.pubkey(), &setup.mint_b));

    send(
        &mut svm,
        &trader,
        &[&trader],
        swap_a_for_b_ix(
            trader.pubkey(),
            setup.mint_a,
            setup.mint_b,
            SWAP_IN,
            1,
        ),
    );

    assert_eq!(
        token_balance(&svm, setup.vault_a),
        INITIAL_A + SWAP_IN
    );
    assert!(token_balance(&svm, ata(&trader.pubkey(), &setup.mint_b)) > b_before);
}

#[test]
fn test_swap_b_for_a() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let trader = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&trader.pubkey(), LAMPORTS).unwrap();

    let setup = setup_pool_with_liquidity(&mut svm, &payer, &payer);
    setup_user_tokens(
        &mut svm,
        &trader.pubkey(),
        setup.mint_a,
        setup.mint_b,
        0,
        SWAP_IN,
    );

    let a_before = token_balance(&svm, ata(&trader.pubkey(), &setup.mint_a));

    send(
        &mut svm,
        &trader,
        &[&trader],
        swap_b_for_a_ix(
            trader.pubkey(),
            setup.mint_a,
            setup.mint_b,
            SWAP_IN,
            1,
        ),
    );

    assert_eq!(
        token_balance(&svm, setup.vault_b),
        INITIAL_B + SWAP_IN
    );
    assert!(token_balance(&svm, ata(&trader.pubkey(), &setup.mint_a)) > a_before);
}

#[test]
fn test_withdraw() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS).unwrap();

    let setup = setup_pool_with_liquidity(&mut svm, &payer, &payer);
    let lp_balance = token_balance(&svm, ata(&payer.pubkey(), &setup.lp_mint));
    let half_lp = lp_balance / 2;

    send(
        &mut svm,
        &payer,
        &[&payer],
        withdraw_ix(
            payer.pubkey(),
            setup.mint_a,
            setup.mint_b,
            half_lp,
            1,
            1,
        ),
    );

    assert_eq!(
        token_balance(&svm, ata(&payer.pubkey(), &setup.lp_mint)),
        lp_balance - half_lp
    );
    assert!(token_balance(&svm, setup.vault_a) < INITIAL_A);
    assert!(token_balance(&svm, setup.vault_b) < INITIAL_B);
}
