use anchorvault_litesvm_tests::*;
use solana_keypair::Keypair;
use solana_signer::Signer;

#[test]
fn test_deposit_without_initialize_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), LAMPORTS).unwrap();

    let vault_state = vault_state_pda(&user.pubkey());
    let vault = vault_pda(&vault_state);

    send_expect_err(
        &mut svm,
        &user,
        &[&user],
        deposit_ix(user.pubkey(), vault, vault_state, DEPOSIT_AMOUNT),
    );
}

#[test]
fn test_initialize_twice_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    let (vault_state, vault) = setup_initialized_vault(&mut svm, &user);

    send_expect_err(
        &mut svm,
        &user,
        &[&user],
        initialize_ix(user.pubkey(), vault_state, vault),
    );
}

#[test]
fn test_withdraw_more_than_vault_lamports_after_deposit_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    let (vault_state, vault) = setup_initialized_vault(&mut svm, &user);

    send(
        &mut svm,
        &user,
        &[&user],
        deposit_ix(user.pubkey(), vault, vault_state, DEPOSIT_AMOUNT),
    );

    let vault_balance = svm.get_balance(&vault).unwrap();
    send_expect_err(
        &mut svm,
        &user,
        &[&user],
        withdraw_ix(user.pubkey(), vault, vault_state, vault_balance + 1),
    );
}

#[test]
fn test_withdraw_more_than_vault_lamports_on_init_only_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    let (vault_state, vault) = setup_initialized_vault(&mut svm, &user);

    let vault_balance = svm.get_balance(&vault).unwrap();
    send_expect_err(
        &mut svm,
        &user,
        &[&user],
        withdraw_ix(user.pubkey(), vault, vault_state, vault_balance + 1),
    );
}

#[test]
fn test_deposit_with_wrong_vault_state_fails() {
    let mut svm = setup_svm();
    let alice = Keypair::new();
    let bob = Keypair::new();
    let (alice_state, alice_vault) = setup_initialized_vault(&mut svm, &alice);
    svm.airdrop(&bob.pubkey(), LAMPORTS).unwrap();

    // Bob tries to deposit into Alice's vault but passes his own vault_state PDA.
    let bob_state = vault_state_pda(&bob.pubkey());
    send_expect_err(
        &mut svm,
        &bob,
        &[&bob],
        deposit_ix(bob.pubkey(), alice_vault, bob_state, DEPOSIT_AMOUNT),
    );

    let _ = (alice_state, bob_state);
}

#[test]
fn test_close_wrong_owner_fails() {
    let mut svm = setup_svm();
    let alice = Keypair::new();
    let bob = Keypair::new();
    let (alice_state, alice_vault) = setup_initialized_vault(&mut svm, &alice);
    svm.airdrop(&bob.pubkey(), LAMPORTS).unwrap();

    send(
        &mut svm,
        &alice,
        &[&alice],
        deposit_ix(alice.pubkey(), alice_vault, alice_state, DEPOSIT_AMOUNT),
    );

    // Bob cannot close Alice's vault (vault_state seeds are tied to Alice).
    let bob_state = vault_state_pda(&bob.pubkey());
    send_expect_err(
        &mut svm,
        &bob,
        &[&bob],
        close_ix(bob.pubkey(), alice_vault, bob_state),
    );
}

#[test]
fn test_double_close_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    let (vault_state, vault) = setup_initialized_vault(&mut svm, &user);

    send(
        &mut svm,
        &user,
        &[&user],
        deposit_ix(user.pubkey(), vault, vault_state, DEPOSIT_AMOUNT),
    );
    send(
        &mut svm,
        &user,
        &[&user],
        close_ix(user.pubkey(), vault, vault_state),
    );

    send_expect_err(
        &mut svm,
        &user,
        &[&user],
        close_ix(user.pubkey(), vault, vault_state),
    );
}
