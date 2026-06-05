//! Shared LiteSVM helpers for NFT staking integration tests.
#![allow(dead_code)]

use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use borsh::BorshDeserialize;
use litesvm::LiteSVM;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1, PluginHeaderV1},
    instructions::{CreateCollectionV1Builder, CreateV2Builder},
    types::{Attributes, DataState, PluginType},
    DataBlob, PluginRegistryV1Safe, ID as CORE_PROGRAM_ID,
};
use nft_staking::{
    constants::{CONFIG_SEED, REWARDS_MINT_SEED, STAKED_COUNT_KEY, UPDATE_AUTHORITY_SEED},
    state::StakeConfig,
};
use num_traits::FromPrimitive;
use solana_clock::Clock;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use solana_transaction::versioned::VersionedTransaction;
use spl_associated_token_account_interface::{
    address::get_associated_token_address, program::ID as ASSOCIATED_TOKEN_PROGRAM_ID,
};
use spl_token_interface::state::Account as TokenAccount;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

pub const LAMPORTS: u64 = 10_000_000_000;
pub const REWARDS_BPS: u16 = 10_000;
pub const FREEZE_PERIOD: u16 = 100;
pub const REWARD_ELAPSED: i64 = 200;

pub const MPL_CORE: Pubkey = Pubkey::new_from_array(CORE_PROGRAM_ID.to_bytes());

pub struct StakingSetup {
    pub admin: Keypair,
    pub user: Keypair,
    pub collection: Keypair,
    pub asset: Keypair,
    pub config: Pubkey,
    pub rewards_mint: Pubkey,
    pub authority: Pubkey,
}

pub fn setup_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    svm.add_program(
        nft_staking::ID,
        include_bytes!("../../../../target/deploy/nft_staking.so"),
    )
    .unwrap();
    svm.add_program(
        MPL_CORE,
        include_bytes!("../fixtures/mpl_core.so"),
    )
    .unwrap();
    svm
}

fn mpl_pk(pk: Pubkey) -> solana_program::pubkey::Pubkey {
    solana_program::pubkey::Pubkey::new_from_array(pk.to_bytes())
}

fn sdk_pk(pk: solana_program::pubkey::Pubkey) -> Pubkey {
    Pubkey::new_from_array(pk.to_bytes())
}

fn mpl_to_sdk_ix(ix: solana_program::instruction::Instruction) -> Instruction {
    Instruction {
        program_id: sdk_pk(ix.program_id),
        accounts: ix
            .accounts
            .into_iter()
            .map(|meta| AccountMeta {
                pubkey: sdk_pk(meta.pubkey),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data: ix.data,
    }
}

pub fn send(svm: &mut LiteSVM, payer: &Keypair, signers: &[&Keypair], ix: Instruction) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "tx failed: {:?}", result.err());
}

pub fn send_expect_err(svm: &mut LiteSVM, payer: &Keypair, signers: &[&Keypair], ix: Instruction) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "expected tx to fail but it succeeded");
}

pub fn set_clock_unix(svm: &mut LiteSVM, unix_timestamp: i64) {
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = unix_timestamp;
    svm.set_sysvar(&clock);
}

pub fn authority_pda(collection: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[UPDATE_AUTHORITY_SEED, collection.as_ref()],
        &nft_staking::ID,
    )
    .0
}

pub fn config_pda(collection: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[CONFIG_SEED, collection.as_ref()], &nft_staking::ID).0
}

pub fn rewards_mint_pda(collection: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[REWARDS_MINT_SEED, collection.as_ref()], &nft_staking::ID).0
}

pub fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address(owner, mint)
}

pub fn token_balance(svm: &LiteSVM, address: Pubkey) -> u64 {
    let account = svm.get_account(&address).expect("token account missing");
    TokenAccount::unpack(&account.data)
        .expect("invalid token account")
        .amount
}

pub fn create_collection_ix(admin: &Keypair, collection: &Keypair) -> Instruction {
    mpl_to_sdk_ix(
        CreateCollectionV1Builder::new()
            .collection(mpl_pk(collection.pubkey()))
            .payer(mpl_pk(admin.pubkey()))
            .update_authority(Some(mpl_pk(admin.pubkey())))
            .name("Test Collection".to_string())
            .uri("https://example.com/collection.json".to_string())
            .instruction(),
    )
}

pub fn create_asset_ix(
    admin: &Keypair,
    asset: &Keypair,
    collection: &Pubkey,
    owner: &Pubkey,
) -> Instruction {
    mpl_to_sdk_ix(
        CreateV2Builder::new()
            .asset(mpl_pk(asset.pubkey()))
            .collection(Some(mpl_pk(*collection)))
            .authority(Some(mpl_pk(admin.pubkey())))
            .payer(mpl_pk(admin.pubkey()))
            .owner(Some(mpl_pk(*owner)))
            .data_state(DataState::AccountState)
            .name("Test NFT".to_string())
            .uri("https://example.com/nft.json".to_string())
            .instruction(),
    )
}

pub fn initialize_ix(
    admin: Pubkey,
    payer: Pubkey,
    collection: Pubkey,
    rewards_bps: u16,
    freeze_period: u16,
) -> Instruction {
    let accounts = nft_staking::accounts::Initialize {
        admin,
        payer,
        collection,
        config: config_pda(&collection),
        rewards_mint: rewards_mint_pda(&collection),
        authority: authority_pda(&collection),
        token_program: TOKEN_PROGRAM_ID,
        system_program: SYSTEM_PROGRAM_ID,
        core_program: MPL_CORE,
    };
    Instruction {
        program_id: nft_staking::ID,
        accounts: accounts.to_account_metas(None),
        data: nft_staking::instruction::Initialize {
            rewards_bps,
            freeze_period,
        }
        .data(),
    }
}

pub fn stake_ix(owner: Pubkey, payer: Pubkey, asset: Pubkey, collection: Pubkey) -> Instruction {
    let accounts = nft_staking::accounts::Stake {
        owner,
        payer,
        asset,
        collection,
        config: config_pda(&collection),
        authority: authority_pda(&collection),
        system_program: SYSTEM_PROGRAM_ID,
        core_program: MPL_CORE,
    };
    Instruction {
        program_id: nft_staking::ID,
        accounts: accounts.to_account_metas(None),
        data: nft_staking::instruction::Stake {}.data(),
    }
}

pub fn claim_rewards_ix(
    owner: Pubkey,
    payer: Pubkey,
    asset: Pubkey,
    collection: Pubkey,
) -> Instruction {
    let rewards_mint = rewards_mint_pda(&collection);
    let accounts = nft_staking::accounts::ClaimRewards {
        owner,
        payer,
        asset,
        collection,
        config: config_pda(&collection),
        rewards_mint,
        owner_rewards_ata: ata(&owner, &rewards_mint),
        authority: authority_pda(&collection),
        token_program: TOKEN_PROGRAM_ID,
        associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
        system_program: SYSTEM_PROGRAM_ID,
        core_program: MPL_CORE,
    };
    Instruction {
        program_id: nft_staking::ID,
        accounts: accounts.to_account_metas(None),
        data: nft_staking::instruction::ClaimRewards {}.data(),
    }
}

pub fn unstake_ix(owner: Pubkey, payer: Pubkey, asset: Pubkey, collection: Pubkey) -> Instruction {
    let accounts = nft_staking::accounts::Unstake {
        owner,
        payer,
        asset,
        collection,
        config: config_pda(&collection),
        authority: authority_pda(&collection),
        system_program: SYSTEM_PROGRAM_ID,
        core_program: MPL_CORE,
    };
    Instruction {
        program_id: nft_staking::ID,
        accounts: accounts.to_account_metas(None),
        data: nft_staking::instruction::Unstake {}.data(),
    }
}

pub fn create_collection_and_asset(
    svm: &mut LiteSVM,
    admin: &Keypair,
    user: &Keypair,
) -> (Keypair, Keypair) {
    let collection = Keypair::new();
    let asset = Keypair::new();
    send(
        svm,
        admin,
        &[admin, &collection],
        create_collection_ix(admin, &collection),
    );
    send(
        svm,
        admin,
        &[admin, &asset],
        create_asset_ix(admin, &asset, &collection.pubkey(), &user.pubkey()),
    );
    (collection, asset)
}

pub fn setup_initialized_staking(
    svm: &mut LiteSVM,
    rewards_bps: u16,
    freeze_period: u16,
) -> StakingSetup {
    let admin = Keypair::new();
    let user = Keypair::new();
    svm.airdrop(&admin.pubkey(), LAMPORTS).unwrap();
    svm.airdrop(&user.pubkey(), LAMPORTS).unwrap();

    let (collection, asset) = create_collection_and_asset(svm, &admin, &user);
    let collection_pk = collection.pubkey();

    send(
        svm,
        &admin,
        &[&admin],
        initialize_ix(
            admin.pubkey(),
            admin.pubkey(),
            collection_pk,
            rewards_bps,
            freeze_period,
        ),
    );

    StakingSetup {
        admin,
        user,
        collection,
        asset,
        config: config_pda(&collection_pk),
        rewards_mint: rewards_mint_pda(&collection_pk),
        authority: authority_pda(&collection_pk),
    }
}

pub fn stake_nft(svm: &mut LiteSVM, setup: &StakingSetup) {
    set_clock_unix(svm, 1_000);
    send(
        svm,
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

fn attributes_after_base(base_len: usize, data: &[u8]) -> Option<Attributes> {
    let header = PluginHeaderV1::from_bytes(&data[base_len..]).ok()?;
    let registry =
        PluginRegistryV1Safe::from_bytes(&data[header.plugin_registry_offset as usize..]).ok()?;
    let record = registry.registry.iter().find(|r| {
        PluginType::from_u8(r.plugin_type) == Some(PluginType::Attributes)
    })?;
    Attributes::deserialize(&mut &data[record.offset as usize + 1..]).ok()
}

pub fn collection_update_authority(svm: &LiteSVM, collection: Pubkey) -> Pubkey {
    let account = svm.get_account(&collection).expect("collection missing");
    let base = BaseCollectionV1::from_bytes(&account.data).expect("invalid collection");
    sdk_pk(base.update_authority)
}

pub fn collection_staked_count(svm: &LiteSVM, collection: Pubkey) -> u64 {
    let account = svm.get_account(&collection).expect("collection missing");
    let base = BaseCollectionV1::from_bytes(&account.data).expect("invalid collection");
    let attrs = attributes_after_base(base.len(), &account.data)
        .expect("collection attributes missing");
    attrs
        .attribute_list
        .iter()
        .find(|a| a.key == STAKED_COUNT_KEY)
        .map(|a| a.value.parse().unwrap_or(0))
        .unwrap_or(0)
}

pub fn asset_is_staked(svm: &LiteSVM, asset: Pubkey) -> bool {
    let account = svm.get_account(&asset).expect("asset missing");
    let base = BaseAssetV1::from_bytes(&account.data).expect("invalid asset");
    let attrs =
        attributes_after_base(base.len(), &account.data).expect("asset attributes missing");
    attrs
        .attribute_list
        .iter()
        .any(|a| a.key == "staked" && a.value == "true")
}

pub fn load_config(svm: &LiteSVM, config: Pubkey) -> StakeConfig {
    let account = svm.get_account(&config).expect("config missing");
    StakeConfig::try_deserialize(&mut account.data.as_ref()).unwrap()
}

pub fn expected_rewards(elapsed: i64, rewards_bps: u16) -> u64 {
    (elapsed as u64)
        .checked_mul(rewards_bps as u64)
        .unwrap()
        .checked_div(10_000)
        .unwrap()
}
