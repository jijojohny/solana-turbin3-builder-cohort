mod common;

use common::*;
use solana_keypair::Keypair;
use solana_signer::Signer;

#[test]
fn test_initialize() {
    let mut svm = setup_svm();
    let setup = setup_initialized_staking(&mut svm, REWARDS_BPS, FREEZE_PERIOD);

    let config = load_config(&svm, setup.config);
    assert_eq!(config.admin, setup.admin.pubkey());
    assert_eq!(config.collection, setup.collection.pubkey());
    assert_eq!(config.rewards_mint, setup.rewards_mint);
    assert_eq!(config.rewards_bps, REWARDS_BPS);
    assert_eq!(config.freeze_period, FREEZE_PERIOD);

    assert_eq!(
        collection_update_authority(&svm, setup.collection.pubkey()),
        setup.authority
    );
    assert_eq!(collection_staked_count(&svm, setup.collection.pubkey()), 0);
}

#[test]
fn test_stake() {
    let mut svm = setup_svm();
    let setup = setup_initialized_staking(&mut svm, REWARDS_BPS, FREEZE_PERIOD);

    stake_nft(&mut svm, &setup);

    assert!(asset_is_staked(&svm, setup.asset.pubkey()));
    assert_eq!(collection_staked_count(&svm, setup.collection.pubkey()), 1);
}

#[test]
fn test_claim_rewards_without_unstake() {
    let mut svm = setup_svm();
    let setup = setup_initialized_staking(&mut svm, REWARDS_BPS, FREEZE_PERIOD);
    stake_nft(&mut svm, &setup);

    set_clock_unix(&mut svm, 1_000 + REWARD_ELAPSED);
    send(
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

    let rewards_ata = ata(&setup.user.pubkey(), &setup.rewards_mint);
    assert_eq!(
        token_balance(&svm, rewards_ata),
        expected_rewards(REWARD_ELAPSED, REWARDS_BPS)
    );
    assert!(asset_is_staked(&svm, setup.asset.pubkey()));
}

#[test]
fn test_claim_then_unstake() {
    let mut svm = setup_svm();
    let setup = setup_initialized_staking(&mut svm, REWARDS_BPS, FREEZE_PERIOD);
    stake_nft(&mut svm, &setup);

    set_clock_unix(&mut svm, 1_000 + REWARD_ELAPSED);
    send(
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

    assert!(asset_is_staked(&svm, setup.asset.pubkey()));

    set_clock_unix(&mut svm, 1_000 + FREEZE_PERIOD as i64 + 1);
    send(
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

    assert!(!asset_is_staked(&svm, setup.asset.pubkey()));
    assert_eq!(collection_staked_count(&svm, setup.collection.pubkey()), 0);
}

#[test]
fn test_unstake() {
    let mut svm = setup_svm();
    let setup = setup_initialized_staking(&mut svm, REWARDS_BPS, FREEZE_PERIOD);
    stake_nft(&mut svm, &setup);

    set_clock_unix(&mut svm, 1_000 + FREEZE_PERIOD as i64 + 1);
    send(
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

    assert!(!asset_is_staked(&svm, setup.asset.pubkey()));
    assert_eq!(collection_staked_count(&svm, setup.collection.pubkey()), 0);
}

#[test]
fn test_multiple_stakes_update_count() {
    let mut svm = setup_svm();
    let admin = Keypair::new();
    let user = Keypair::new();
    svm.airdrop(&admin.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&user.pubkey(), LAMPORTS).unwrap();

    let (collection, asset_one) = create_collection_and_asset(&mut svm, &admin, &user);
    let collection_pk = collection.pubkey();
    let asset_two = Keypair::new();
    send(
        &mut svm,
        &admin,
        &[&admin, &asset_two],
        create_asset_ix(
            &admin,
            &asset_two,
            &collection_pk,
            &user.pubkey(),
        ),
    );

    send(
        &mut svm,
        &admin,
        &[&admin],
        initialize_ix(
            admin.pubkey(),
            admin.pubkey(),
            collection_pk,
            REWARDS_BPS,
            FREEZE_PERIOD,
        ),
    );

    let setup = StakingSetup {
        admin,
        user,
        collection,
        asset: asset_one,
        config: config_pda(&collection_pk),
        rewards_mint: rewards_mint_pda(&collection_pk),
        authority: authority_pda(&collection_pk),
    };

    stake_nft(&mut svm, &setup);
    send(
        &mut svm,
        &setup.user,
        &[&setup.user],
        stake_ix(
            setup.user.pubkey(),
            setup.user.pubkey(),
            asset_two.pubkey(),
            setup.collection.pubkey(),
        ),
    );

    assert_eq!(collection_staked_count(&svm, setup.collection.pubkey()), 2);
}
