mod common;

use common::*;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

#[test]
fn test_initialize_invalid_mint_order() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS).unwrap();

    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    let (mint_lo, mint_hi) = sorted_mints(mint_a, mint_b);

    write_mint(&mut svm, mint_lo, payer.pubkey(), DECIMALS);
    write_mint(&mut svm, mint_hi, payer.pubkey(), DECIMALS);

    send_expect_err(
        &mut svm,
        &payer,
        &[&payer],
        initialize_ix(payer.pubkey(), mint_hi, mint_lo, FEE_BPS),
    );
}

#[test]
fn test_deposit_zero_amount() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS).unwrap();

    let setup = setup_initialized_pool(&mut svm, &payer, &payer.pubkey());
    setup_user_tokens(
        &mut svm,
        &payer.pubkey(),
        setup.mint_a,
        setup.mint_b,
        INITIAL_A,
        INITIAL_B,
    );

    send_expect_err(
        &mut svm,
        &payer,
        &[&payer],
        deposit_ix(
            payer.pubkey(),
            setup.mint_a,
            setup.mint_b,
            0,
            INITIAL_B,
            0,
        ),
    );
}

#[test]
fn test_swap_slippage_exceeded() {
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

    send_expect_err(
        &mut svm,
        &trader,
        &[&trader],
        swap_a_for_b_ix(
            trader.pubkey(),
            setup.mint_a,
            setup.mint_b,
            SWAP_IN,
            u64::MAX,
        ),
    );
}

#[test]
fn test_withdraw_payout_without_burn() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS).unwrap();

    let setup = setup_pool_with_liquidity(&mut svm, &payer, &payer);

    send_expect_err(
        &mut svm,
        &payer,
        &[&payer],
        withdraw_payout_ix(payer.pubkey(), setup.mint_a, setup.mint_b, 1, 1),
    );
}

#[test]
fn test_withdraw_payout_wrong_instruction_order() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS).unwrap();

    let setup = setup_pool_with_liquidity(&mut svm, &payer, &payer);
    let lp_balance = token_balance(&svm, ata(&payer.pubkey(), &setup.lp_mint));

    let ixs = [
        withdraw_payout_ix(payer.pubkey(), setup.mint_a, setup.mint_b, 1, 1),
        burn_lp_ix(
            payer.pubkey(),
            setup.mint_a,
            setup.mint_b,
            lp_balance / 2,
        ),
    ];
    send_multi_expect_err(&mut svm, &payer, &[&payer], &ixs);
}

#[test]
fn test_withdraw_payout_slippage_exceeded() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS).unwrap();

    let setup = setup_pool_with_liquidity(&mut svm, &payer, &payer);
    let lp_balance = token_balance(&svm, ata(&payer.pubkey(), &setup.lp_mint));

    let ixs = burn_and_withdraw_ixs(
        payer.pubkey(),
        setup.mint_a,
        setup.mint_b,
        lp_balance,
        u64::MAX,
        u64::MAX,
    );
    send_multi_expect_err(&mut svm, &payer, &[&payer], &ixs);
}
