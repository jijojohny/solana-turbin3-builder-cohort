mod common;

use common::*;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

#[test]
fn test_take_insufficient_mint_b_fails() {
    let mut svm = setup_svm();
    let maker = Keypair::new();
    let taker = Keypair::new();
    svm.airdrop(&maker.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&taker.pubkey(), LAMPORTS).unwrap();

    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    write_mint(&mut svm, mint_a, maker.pubkey(), 6);
    write_mint(&mut svm, mint_b, maker.pubkey(), 6);
    write_token_account(
        &mut svm,
        ata(&maker.pubkey(), &mint_a),
        mint_a,
        maker.pubkey(),
        DEPOSIT * 2,
    );
    write_token_account(
        &mut svm,
        ata(&taker.pubkey(), &mint_b),
        mint_b,
        taker.pubkey(),
        RECEIVE - 1,
    );

    let escrow = escrow_pda(&maker.pubkey(), SEED);
    let vault = ata(&escrow, &mint_a);
    send(
        &mut svm,
        &maker,
        &[&maker],
        make_ix(
            maker.pubkey(),
            mint_a,
            mint_b,
            ata(&maker.pubkey(), &mint_a),
            escrow,
            vault,
            SEED,
            DEPOSIT,
            RECEIVE,
        ),
    );

    send_expect_err(
        &mut svm,
        &taker,
        &[&taker],
        take_ix(
            taker.pubkey(),
            maker.pubkey(),
            mint_a,
            mint_b,
            escrow,
            vault,
        ),
    );

    assert!(svm.get_account(&escrow).is_some());
}

#[test]
fn test_refund_by_non_maker_fails() {
    let mut svm = setup_svm();
    let maker = Keypair::new();
    let taker = Keypair::new();
    svm.airdrop(&maker.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&taker.pubkey(), LAMPORTS).unwrap();

    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    let (escrow, vault) = setup_open_escrow(&mut svm, &maker, &taker, mint_a, mint_b, SEED);

    send_expect_err(
        &mut svm,
        &taker,
        &[&taker],
        refund_ix(taker.pubkey(), mint_a, ata(&taker.pubkey(), &mint_a), escrow, vault),
    );

    assert!(svm.get_account(&escrow).is_some());
}

#[test]
fn test_take_wrong_maker_fails() {
    let mut svm = setup_svm();
    let maker = Keypair::new();
    let taker = Keypair::new();
    let impostor = Keypair::new();
    svm.airdrop(&maker.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&taker.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&impostor.pubkey(), LAMPORTS).unwrap();

    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    let (escrow, vault) = setup_open_escrow(&mut svm, &maker, &taker, mint_a, mint_b, SEED);

    send_expect_err(
        &mut svm,
        &taker,
        &[&taker],
        take_ix(
            taker.pubkey(),
            impostor.pubkey(),
            mint_a,
            mint_b,
            escrow,
            vault,
        ),
    );
}

#[test]
fn test_take_wrong_mint_fails() {
    let mut svm = setup_svm();
    let maker = Keypair::new();
    let taker = Keypair::new();
    svm.airdrop(&maker.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&taker.pubkey(), LAMPORTS).unwrap();

    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    let wrong_mint = Pubkey::new_unique();
    write_mint(&mut svm, wrong_mint, maker.pubkey(), 6);

    let (escrow, vault) = setup_open_escrow(&mut svm, &maker, &taker, mint_a, mint_b, SEED);

    send_expect_err(
        &mut svm,
        &taker,
        &[&taker],
        take_ix(
            taker.pubkey(),
            maker.pubkey(),
            wrong_mint,
            mint_b,
            escrow,
            vault,
        ),
    );
}

#[test]
fn test_double_take_fails() {
    let mut svm = setup_svm();
    let maker = Keypair::new();
    let taker = Keypair::new();
    svm.airdrop(&maker.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&taker.pubkey(), LAMPORTS).unwrap();

    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    let (escrow, vault) = setup_open_escrow(&mut svm, &maker, &taker, mint_a, mint_b, SEED);

    send(
        &mut svm,
        &taker,
        &[&taker],
        take_ix(
            taker.pubkey(),
            maker.pubkey(),
            mint_a,
            mint_b,
            escrow,
            vault,
        ),
    );

    send_expect_err(
        &mut svm,
        &taker,
        &[&taker],
        take_ix(
            taker.pubkey(),
            maker.pubkey(),
            mint_a,
            mint_b,
            escrow,
            vault,
        ),
    );
}

#[test]
fn test_take_after_refund_fails() {
    let mut svm = setup_svm();
    let maker = Keypair::new();
    let taker = Keypair::new();
    svm.airdrop(&maker.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&taker.pubkey(), LAMPORTS).unwrap();

    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    let (escrow, vault) = setup_open_escrow(&mut svm, &maker, &taker, mint_a, mint_b, SEED);

    send(
        &mut svm,
        &maker,
        &[&maker],
        refund_ix(
            maker.pubkey(),
            mint_a,
            ata(&maker.pubkey(), &mint_a),
            escrow,
            vault,
        ),
    );

    send_expect_err(
        &mut svm,
        &taker,
        &[&taker],
        take_ix(
            taker.pubkey(),
            maker.pubkey(),
            mint_a,
            mint_b,
            escrow,
            vault,
        ),
    );
}

#[test]
fn test_make_insufficient_deposit_fails() {
    let mut svm = setup_svm();
    let maker = Keypair::new();
    svm.airdrop(&maker.pubkey(), LAMPORTS).unwrap();

    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    write_mint(&mut svm, mint_a, maker.pubkey(), 6);
    write_mint(&mut svm, mint_b, maker.pubkey(), 6);
    write_token_account(
        &mut svm,
        ata(&maker.pubkey(), &mint_a),
        mint_a,
        maker.pubkey(),
        DEPOSIT - 1,
    );

    let escrow = escrow_pda(&maker.pubkey(), SEED);
    let vault = ata(&escrow, &mint_a);

    send_expect_err(
        &mut svm,
        &maker,
        &[&maker],
        make_ix(
            maker.pubkey(),
            mint_a,
            mint_b,
            ata(&maker.pubkey(), &mint_a),
            escrow,
            vault,
            SEED,
            DEPOSIT,
            RECEIVE,
        ),
    );
}

#[test]
fn test_duplicate_make_same_seed_fails() {
    let mut svm = setup_svm();
    let maker = Keypair::new();
    let taker = Keypair::new();
    svm.airdrop(&maker.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&taker.pubkey(), LAMPORTS).unwrap();

    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    let (escrow, vault) = setup_open_escrow(&mut svm, &maker, &taker, mint_a, mint_b, SEED);

    send_expect_err(
        &mut svm,
        &maker,
        &[&maker],
        make_ix(
            maker.pubkey(),
            mint_a,
            mint_b,
            ata(&maker.pubkey(), &mint_a),
            escrow,
            vault,
            SEED,
            DEPOSIT,
            RECEIVE,
        ),
    );
}

#[test]
fn test_refund_after_take_fails() {
    let mut svm = setup_svm();
    let maker = Keypair::new();
    let taker = Keypair::new();
    svm.airdrop(&maker.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&taker.pubkey(), LAMPORTS).unwrap();

    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    let (escrow, vault) = setup_open_escrow(&mut svm, &maker, &taker, mint_a, mint_b, SEED);

    send(
        &mut svm,
        &taker,
        &[&taker],
        take_ix(
            taker.pubkey(),
            maker.pubkey(),
            mint_a,
            mint_b,
            escrow,
            vault,
        ),
    );

    send_expect_err(
        &mut svm,
        &maker,
        &[&maker],
        refund_ix(
            maker.pubkey(),
            mint_a,
            ata(&maker.pubkey(), &mint_a),
            escrow,
            vault,
        ),
    );
}
