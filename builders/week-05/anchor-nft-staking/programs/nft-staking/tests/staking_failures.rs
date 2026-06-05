mod common;

use common::*;
use solana_signer::Signer;

#[test]
fn test_stake_before_initialize_fails() {
    let mut svm = setup_svm();
    let admin = solana_keypair::Keypair::new();
    let user = solana_keypair::Keypair::new();
    svm.airdrop(&admin.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&user.pubkey(), LAMPORTS).unwrap();

    let (collection, asset) = create_collection_and_asset(&mut svm, &admin, &user);
    set_clock_unix(&mut svm, 1_000);

    send_expect_err(
        &mut svm,
        &user,
        &[&user],
        stake_ix(
            user.pubkey(),
            user.pubkey(),
            asset.pubkey(),
            collection.pubkey(),
        ),
    );
}

#[test]
fn test_stake_twice_fails() {
    let mut svm = setup_svm();
    let setup = setup_initialized_staking(&mut svm, REWARDS_BPS, FREEZE_PERIOD);
    stake_nft(&mut svm, &setup);

    send_expect_err(
        &mut svm,
        &setup.user,
        &[&setup.user],
        stake_ix(
            setup.user.pubkey(),
            setup.user.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
        ),
    );
}

#[test]
fn test_claim_without_stake_fails() {
    let mut svm = setup_svm();
    let setup = setup_initialized_staking(&mut svm, REWARDS_BPS, FREEZE_PERIOD);

    send_expect_err(
        &mut svm,
        &setup.user,
        &[&setup.user],
        claim_rewards_ix(
            setup.user.pubkey(),
            setup.user.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
        ),
    );
}

#[test]
fn test_claim_with_no_elapsed_rewards_fails() {
    let mut svm = setup_svm();
    let setup = setup_initialized_staking(&mut svm, REWARDS_BPS, FREEZE_PERIOD);
    stake_nft(&mut svm, &setup);

    send_expect_err(
        &mut svm,
        &setup.user,
        &[&setup.user],
        claim_rewards_ix(
            setup.user.pubkey(),
            setup.user.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
        ),
    );
}

#[test]
fn test_unstake_before_freeze_period_fails() {
    let mut svm = setup_svm();
    let setup = setup_initialized_staking(&mut svm, REWARDS_BPS, FREEZE_PERIOD);
    stake_nft(&mut svm, &setup);

    send_expect_err(
        &mut svm,
        &setup.user,
        &[&setup.user],
        unstake_ix(
            setup.user.pubkey(),
            setup.user.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
        ),
    );
}

#[test]
fn test_unstake_without_stake_fails() {
    let mut svm = setup_svm();
    let setup = setup_initialized_staking(&mut svm, REWARDS_BPS, FREEZE_PERIOD);

    send_expect_err(
        &mut svm,
        &setup.user,
        &[&setup.user],
        unstake_ix(
            setup.user.pubkey(),
            setup.user.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
        ),
    );
}
