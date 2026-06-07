//! Shared LiteSVM helpers for marketplace integration tests.
#![allow(dead_code)]

use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use mpl_core::{
    accounts::BaseAssetV1,
    instructions::{CreateCollectionV1Builder, CreateV2Builder},
    types::DataState,
    ID as CORE_PROGRAM_ID,
};
use nft_marketplace::{
    constants::{LISTING_SEED, MARKETPLACE_SEED, OFFER_SEED, OFFER_VAULT_SEED},
    state::{Listing, Marketplace},
};

pub use nft_marketplace::constants::NATIVE_MINT;
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_program_option::COption;
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use solana_transaction::versioned::VersionedTransaction;
use spl_associated_token_account_interface::{
    address::get_associated_token_address, program::ID as ASSOCIATED_TOKEN_PROGRAM_ID,
};
use spl_token_interface::state::{Account as TokenAccount, AccountState, Mint};
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

pub const LAMPORTS: u64 = 10_000_000_000;
pub const FEE_BPS: u16 = 250;
pub const LIST_PRICE_SOL: u64 = 1_000_000_000;
pub const LIST_PRICE_TOKEN: u64 = 1_000_000;
pub const OFFER_AMOUNT: u64 = 500_000_000;
pub const TOKEN_DECIMALS: u8 = 6;

pub const MPL_CORE: Pubkey = Pubkey::new_from_array(CORE_PROGRAM_ID.to_bytes());

pub struct MarketSetup {
    pub admin: Keypair,
    pub treasury: Keypair,
    pub maker: Keypair,
    pub buyer: Keypair,
    pub collection: Keypair,
    pub asset: Keypair,
    pub payment_mint: Keypair,
    pub marketplace: Pubkey,
}

pub fn setup_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    svm.add_program(
        nft_marketplace::ID,
        include_bytes!("../../../../target/deploy/nft_marketplace.so"),
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

pub fn marketplace_pda() -> Pubkey {
    Pubkey::find_program_address(&[MARKETPLACE_SEED], &nft_marketplace::ID).0
}

pub fn listing_pda(asset: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[LISTING_SEED, asset.as_ref()], &nft_marketplace::ID).0
}

pub fn offer_pda(asset: &Pubkey, buyer: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[OFFER_SEED, asset.as_ref(), buyer.as_ref()],
        &nft_marketplace::ID,
    )
    .0
}

pub fn offer_vault_pda(asset: &Pubkey, buyer: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[OFFER_VAULT_SEED, asset.as_ref(), buyer.as_ref()],
        &nft_marketplace::ID,
    )
    .0
}

pub fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address(owner, mint)
}

pub fn lamports(svm: &LiteSVM, address: Pubkey) -> u64 {
    svm.get_account(&address).expect("account missing").lamports
}

pub fn token_balance(svm: &LiteSVM, address: Pubkey) -> u64 {
    let account = svm.get_account(&address).expect("token account missing");
    TokenAccount::unpack(&account.data)
        .expect("invalid token account")
        .amount
}

pub fn token_balance_or_zero(svm: &LiteSVM, address: Pubkey) -> u64 {
    let Some(account) = svm.get_account(&address) else {
        return 0;
    };
    TokenAccount::unpack(&account.data)
        .expect("invalid token account")
        .amount
}

pub fn write_mint(svm: &mut LiteSVM, mint: Pubkey, authority: Pubkey, decimals: u8) {
    let mut data = vec![0u8; Mint::LEN];
    Mint::pack(
        Mint {
            mint_authority: COption::Some(authority),
            supply: 0,
            decimals,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        &mut data,
    )
    .unwrap();
    svm.set_account(
        mint,
        Account {
            lamports: svm.minimum_balance_for_rent_exemption(Mint::LEN),
            data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

pub fn write_token_account(
    svm: &mut LiteSVM,
    address: Pubkey,
    mint: Pubkey,
    owner: Pubkey,
    amount: u64,
) {
    let mut data = vec![0u8; TokenAccount::LEN];
    TokenAccount::pack(
        TokenAccount {
            mint,
            owner,
            amount,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        &mut data,
    )
    .unwrap();
    svm.set_account(
        address,
        Account {
            lamports: svm.minimum_balance_for_rent_exemption(TokenAccount::LEN),
            data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

pub fn create_collection_ix(admin: &Keypair, collection: &Keypair) -> Instruction {
    mpl_to_sdk_ix(
        CreateCollectionV1Builder::new()
            .collection(mpl_pk(collection.pubkey()))
            .payer(mpl_pk(admin.pubkey()))
            .update_authority(Some(mpl_pk(admin.pubkey())))
            .name("Market Collection".to_string())
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
            .name("Market NFT".to_string())
            .uri("https://example.com/nft.json".to_string())
            .instruction(),
    )
}

pub fn initialize_ix(admin: Pubkey, treasury: Pubkey, fee_bps: u16) -> Instruction {
    let accounts = nft_marketplace::accounts::Initialize {
        admin,
        marketplace: marketplace_pda(),
        treasury,
        system_program: SYSTEM_PROGRAM_ID,
    };
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: accounts.to_account_metas(None),
        data: nft_marketplace::instruction::Initialize { fee_bps }.data(),
    }
}

pub fn list_ix(
    maker: Pubkey,
    asset: Pubkey,
    collection: Pubkey,
    price: u64,
    payment_mint: Pubkey,
) -> Instruction {
    let accounts = nft_marketplace::accounts::List {
        maker,
        marketplace: marketplace_pda(),
        asset,
        collection,
        listing: listing_pda(&asset),
        payment_mint,
        token_program: TOKEN_PROGRAM_ID,
        system_program: SYSTEM_PROGRAM_ID,
        core_program: MPL_CORE,
    };
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: accounts.to_account_metas(None),
        data: nft_marketplace::instruction::List {
            price,
            payment_mint,
        }
        .data(),
    }
}

pub fn delist_ix(maker: Pubkey, asset: Pubkey, collection: Pubkey) -> Instruction {
    let accounts = nft_marketplace::accounts::Delist {
        maker,
        marketplace: marketplace_pda(),
        asset,
        collection,
        listing: listing_pda(&asset),
        system_program: SYSTEM_PROGRAM_ID,
        core_program: MPL_CORE,
    };
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: accounts.to_account_metas(None),
        data: nft_marketplace::instruction::Delist {}.data(),
    }
}

pub fn buy_ix_with_treasury(
    buyer: Pubkey,
    maker: Pubkey,
    treasury: Pubkey,
    asset: Pubkey,
    collection: Pubkey,
) -> Instruction {
    let accounts = nft_marketplace::accounts::Buy {
        buyer,
        maker,
        marketplace: marketplace_pda(),
        treasury,
        asset,
        collection,
        listing: listing_pda(&asset),
        system_program: SYSTEM_PROGRAM_ID,
        core_program: MPL_CORE,
    };
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: accounts.to_account_metas(None),
        data: nft_marketplace::instruction::Buy {}.data(),
    }
}

pub fn buy_with_token_ix(
    buyer: Pubkey,
    maker: Pubkey,
    treasury: Pubkey,
    asset: Pubkey,
    collection: Pubkey,
    payment_mint: Pubkey,
) -> Instruction {
    let accounts = nft_marketplace::accounts::BuyWithToken {
        buyer,
        maker,
        marketplace: marketplace_pda(),
        treasury,
        asset,
        collection,
        listing: listing_pda(&asset),
        payment_mint,
        buyer_payment_ata: ata(&buyer, &payment_mint),
        maker_payment_ata: ata(&maker, &payment_mint),
        treasury_payment_ata: ata(&treasury, &payment_mint),
        token_program: TOKEN_PROGRAM_ID,
        associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
        system_program: SYSTEM_PROGRAM_ID,
        core_program: MPL_CORE,
    };
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: accounts.to_account_metas(None),
        data: nft_marketplace::instruction::BuyWithToken {}.data(),
    }
}

pub fn make_offer_ix(buyer: Pubkey, asset: Pubkey, amount: u64) -> Instruction {
    let accounts = nft_marketplace::accounts::MakeOffer {
        buyer,
        marketplace: marketplace_pda(),
        asset,
        offer: offer_pda(&asset, &buyer),
        offer_vault: offer_vault_pda(&asset, &buyer),
        system_program: SYSTEM_PROGRAM_ID,
    };
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: accounts.to_account_metas(None),
        data: nft_marketplace::instruction::MakeOffer { amount }.data(),
    }
}

pub fn accept_offer_ix(
    maker: Pubkey,
    buyer: Pubkey,
    treasury: Pubkey,
    asset: Pubkey,
    collection: Pubkey,
) -> Instruction {
    let accounts = nft_marketplace::accounts::AcceptOffer {
        maker,
        buyer,
        marketplace: marketplace_pda(),
        treasury,
        asset,
        collection,
        offer: offer_pda(&asset, &buyer),
        offer_vault: offer_vault_pda(&asset, &buyer),
        system_program: SYSTEM_PROGRAM_ID,
        core_program: MPL_CORE,
    };
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: accounts.to_account_metas(None),
        data: nft_marketplace::instruction::AcceptOffer {}.data(),
    }
}

pub fn cancel_offer_ix(buyer: Pubkey, asset: Pubkey) -> Instruction {
    let accounts = nft_marketplace::accounts::CancelOffer {
        buyer,
        asset,
        offer: offer_pda(&asset, &buyer),
        offer_vault: offer_vault_pda(&asset, &buyer),
        system_program: SYSTEM_PROGRAM_ID,
    };
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: accounts.to_account_metas(None),
        data: nft_marketplace::instruction::CancelOffer {}.data(),
    }
}

pub fn create_collection_and_asset(
    svm: &mut LiteSVM,
    admin: &Keypair,
    owner: &Keypair,
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
        create_asset_ix(admin, &asset, &collection.pubkey(), &owner.pubkey()),
    );
    (collection, asset)
}

pub fn setup_marketplace(svm: &mut LiteSVM) -> MarketSetup {
    let admin = Keypair::new();
    let treasury = Keypair::new();
    let maker = Keypair::new();
    let buyer = Keypair::new();
    let payment_mint = Keypair::new();

    for kp in [&admin, &treasury, &maker, &buyer] {
        svm.airdrop(&kp.pubkey(), LAMPORTS).unwrap();
    }

    write_mint(
        svm,
        payment_mint.pubkey(),
        admin.pubkey(),
        TOKEN_DECIMALS,
    );

    let buyer_ata = ata(&buyer.pubkey(), &payment_mint.pubkey());
    write_token_account(
        svm,
        buyer_ata,
        payment_mint.pubkey(),
        buyer.pubkey(),
        LIST_PRICE_TOKEN * 2,
    );

    let (collection, asset) = create_collection_and_asset(svm, &admin, &maker);

    send(
        svm,
        &admin,
        &[&admin],
        initialize_ix(admin.pubkey(), treasury.pubkey(), FEE_BPS),
    );

    MarketSetup {
        admin,
        treasury,
        maker,
        buyer,
        collection,
        asset,
        payment_mint,
        marketplace: marketplace_pda(),
    }
}

pub fn load_marketplace(svm: &LiteSVM) -> Marketplace {
    let account = svm
        .get_account(&marketplace_pda())
        .expect("marketplace missing");
    Marketplace::try_deserialize(&mut account.data.as_ref()).unwrap()
}

pub fn load_listing(svm: &LiteSVM, asset: Pubkey) -> Listing {
    let account = svm
        .get_account(&listing_pda(&asset))
        .expect("listing missing");
    Listing::try_deserialize(&mut account.data.as_ref()).unwrap()
}

pub fn asset_owner(svm: &LiteSVM, asset: Pubkey) -> Pubkey {
    let account = svm.get_account(&asset).expect("asset missing");
    let base = BaseAssetV1::from_bytes(&account.data).expect("invalid asset");
    sdk_pk(base.owner)
}

pub fn split_payment(total: u64, fee_bps: u16) -> (u64, u64) {
    let fee = total * fee_bps as u64 / 10_000;
    (total - fee, fee)
}

pub fn list_sol(svm: &mut LiteSVM, setup: &MarketSetup, price: u64) {
    send(
        svm,
        &setup.maker,
        &[&setup.maker],
        list_ix(
            setup.maker.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
            price,
            NATIVE_MINT,
        ),
    );
}

pub fn list_token(svm: &mut LiteSVM, setup: &MarketSetup, price: u64) {
    send(
        svm,
        &setup.maker,
        &[&setup.maker],
        list_ix(
            setup.maker.pubkey(),
            setup.asset.pubkey(),
            setup.collection.pubkey(),
            price,
            setup.payment_mint.pubkey(),
        ),
    );
}
