mod common;

use anchor_escrow::state::Escrow;
use anchor_lang::AccountDeserialize;
use common::*;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

#[test]
fn test_make_take() {
    let mut svm = setup_svm();
    let maker = Keypair::new();
    let taker = Keypair::new();
    svm.airdrop(&maker.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&taker.pubkey(), LAMPORTS).unwrap();

    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    let (escrow, vault) = setup_open_escrow(&mut svm, &maker, &taker, mint_a, mint_b, SEED);
    let maker_ata_a = ata(&maker.pubkey(), &mint_a);

    let escrow_account = svm.get_account(&escrow).expect("escrow missing");
    let escrow_state = Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
    assert_eq!(escrow_state.seed, SEED);
    assert_eq!(escrow_state.maker, maker.pubkey());
    assert_eq!(escrow_state.mint_a, mint_a);
    assert_eq!(escrow_state.mint_b, mint_b);
    assert_eq!(escrow_state.receive, RECEIVE);
    assert_eq!(token_balance(&svm, vault), DEPOSIT);
    assert_eq!(token_balance(&svm, maker_ata_a), DEPOSIT);

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

    assert!(svm.get_account(&escrow).is_none());
    assert!(svm.get_account(&vault).is_none());
    assert_eq!(token_balance(&svm, ata(&taker.pubkey(), &mint_a)), DEPOSIT);
    assert_eq!(token_balance(&svm, ata(&maker.pubkey(), &mint_b)), RECEIVE);
}

#[test]
fn test_make_refund() {
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
        DEPOSIT * 2,
    );

    let escrow = escrow_pda(&maker.pubkey(), SEED);
    let vault = ata(&escrow, &mint_a);
    let maker_ata_a = ata(&maker.pubkey(), &mint_a);

    send(
        &mut svm,
        &maker,
        &[&maker],
        make_ix(
            maker.pubkey(),
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            SEED,
            DEPOSIT,
            RECEIVE,
        ),
    );

    send(
        &mut svm,
        &maker,
        &[&maker],
        refund_ix(maker.pubkey(), mint_a, maker_ata_a, escrow, vault),
    );

    assert!(svm.get_account(&escrow).is_none());
    assert!(svm.get_account(&vault).is_none());
    assert_eq!(token_balance(&svm, maker_ata_a), DEPOSIT * 2);
}
