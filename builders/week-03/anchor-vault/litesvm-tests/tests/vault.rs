use anchorvault_litesvm_tests::*;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

#[test]
fn test_initialize() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), LAMPORTS).unwrap();

    let vault_state = vault_state_pda(&user.pubkey());
    let vault = vault_pda(&vault_state);

    send(
        &mut svm,
        &user,
        &[&user],
        initialize_ix(user.pubkey(), vault_state, vault),
    );

    let (vault_bump, state_bump) = read_vault_state(&svm, vault_state);
    assert_eq!(
        vault_bump,
        Pubkey::find_program_address(&[b"vault", vault_state.as_ref()], &PROGRAM_ID).1
    );
    assert_eq!(
        state_bump,
        Pubkey::find_program_address(&[b"state", user.pubkey().as_ref()], &PROGRAM_ID).1
    );
    assert!(svm.get_account(&vault).is_some());
}

#[test]
fn test_deposit() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), LAMPORTS).unwrap();

    let vault_state = vault_state_pda(&user.pubkey());
    let vault = vault_pda(&vault_state);

    send(
        &mut svm,
        &user,
        &[&user],
        initialize_ix(user.pubkey(), vault_state, vault),
    );

    let balance_before = svm.get_balance(&user.pubkey()).unwrap();
    let vault_before = svm.get_balance(&vault).unwrap();

    send(
        &mut svm,
        &user,
        &[&user],
        deposit_ix(user.pubkey(), vault, vault_state, DEPOSIT_AMOUNT),
    );

    const TX_FEE: u64 = 5_000;

    assert_eq!(
        svm.get_balance(&vault).unwrap(),
        vault_before + DEPOSIT_AMOUNT
    );
    assert_eq!(
        svm.get_balance(&user.pubkey()).unwrap(),
        balance_before - DEPOSIT_AMOUNT - TX_FEE
    );
}

#[test]
fn test_withdraw() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), LAMPORTS).unwrap();

    let vault_state = vault_state_pda(&user.pubkey());
    let vault = vault_pda(&vault_state);

    send(
        &mut svm,
        &user,
        &[&user],
        initialize_ix(user.pubkey(), vault_state, vault),
    );
    send(
        &mut svm,
        &user,
        &[&user],
        deposit_ix(user.pubkey(), vault, vault_state, DEPOSIT_AMOUNT),
    );

    let balance_before = svm.get_balance(&user.pubkey()).unwrap();
    let vault_before_withdraw = svm.get_balance(&vault).unwrap();
    let withdraw_amount = DEPOSIT_AMOUNT / 2;

    send(
        &mut svm,
        &user,
        &[&user],
        withdraw_ix(user.pubkey(), vault, vault_state, withdraw_amount),
    );

    const TX_FEE: u64 = 5_000;

    assert_eq!(
        svm.get_balance(&user.pubkey()).unwrap(),
        balance_before + withdraw_amount - TX_FEE
    );
    assert_eq!(
        svm.get_balance(&vault).unwrap(),
        vault_before_withdraw - withdraw_amount
    );
}

#[test]
fn test_close() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), LAMPORTS).unwrap();

    let vault_state = vault_state_pda(&user.pubkey());
    let vault = vault_pda(&vault_state);

    send(
        &mut svm,
        &user,
        &[&user],
        initialize_ix(user.pubkey(), vault_state, vault),
    );
    send(
        &mut svm,
        &user,
        &[&user],
        deposit_ix(user.pubkey(), vault, vault_state, DEPOSIT_AMOUNT),
    );

    let balance_before = svm.get_balance(&user.pubkey()).unwrap();
    let vault_balance = svm.get_balance(&vault).unwrap();

    send(
        &mut svm,
        &user,
        &[&user],
        close_ix(user.pubkey(), vault, vault_state),
    );

    const TX_FEE: u64 = 5_000;

    assert!(svm.get_account(&vault_state).is_none());
    assert_eq!(svm.get_balance(&vault).unwrap_or(0), 0);
    assert!(
        svm.get_balance(&user.pubkey()).unwrap()
            >= balance_before + vault_balance - TX_FEE
    );
}
