use anchor_escrow::constants::ESCROW_SEED;
use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_account::Account;
use solana_instruction::Instruction;
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
pub const DEPOSIT: u64 = 1_000_000;
pub const RECEIVE: u64 = 500_000;
pub const SEED: u64 = 99;

pub fn setup_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    svm.add_program(
        anchor_escrow::ID,
        include_bytes!("../../../../target/deploy/anchor_escrow.so"),
    )
    .unwrap();
    svm
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

pub fn escrow_pda(maker: &Pubkey, seed: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[ESCROW_SEED, maker.as_ref(), &seed.to_le_bytes()],
        &anchor_escrow::ID,
    )
    .0
}

pub fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address(owner, mint)
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

pub fn token_balance(svm: &LiteSVM, address: Pubkey) -> u64 {
    let account = svm.get_account(&address).expect("token account missing");
    TokenAccount::unpack(&account.data)
        .expect("invalid token account")
        .amount
}

pub fn setup_token_holders(
    svm: &mut LiteSVM,
    maker: &Keypair,
    taker: &Keypair,
    mint_a: Pubkey,
    mint_b: Pubkey,
) {
    write_mint(svm, mint_a, maker.pubkey(), 6);
    write_mint(svm, mint_b, maker.pubkey(), 6);
    write_token_account(svm, ata(&maker.pubkey(), &mint_a), mint_a, maker.pubkey(), DEPOSIT * 2);
    write_token_account(
        svm,
        ata(&taker.pubkey(), &mint_b),
        mint_b,
        taker.pubkey(),
        RECEIVE * 2,
    );
}

pub fn make_ix(
    maker: Pubkey,
    mint_a: Pubkey,
    mint_b: Pubkey,
    maker_ata_a: Pubkey,
    escrow: Pubkey,
    vault: Pubkey,
    seed: u64,
    deposit: u64,
    receive: u64,
) -> Instruction {
    Instruction::new_with_bytes(
        anchor_escrow::ID,
        &anchor_escrow::instruction::Make {
            seed,
            deposit,
            receive,
        }
        .data(),
        anchor_escrow::accounts::Make {
            maker,
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}

pub fn take_ix(
    taker: Pubkey,
    maker: Pubkey,
    mint_a: Pubkey,
    mint_b: Pubkey,
    escrow: Pubkey,
    vault: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        anchor_escrow::ID,
        &anchor_escrow::instruction::Take {}.data(),
        anchor_escrow::accounts::Take {
            taker,
            maker,
            mint_a,
            mint_b,
            taker_ata_a: ata(&taker, &mint_a),
            taker_ata_b: ata(&taker, &mint_b),
            maker_ata_b: ata(&maker, &mint_b),
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}

pub fn refund_ix(
    maker: Pubkey,
    mint_a: Pubkey,
    maker_ata_a: Pubkey,
    escrow: Pubkey,
    vault: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        anchor_escrow::ID,
        &anchor_escrow::instruction::Refund {}.data(),
        anchor_escrow::accounts::Refund {
            maker,
            mint_a,
            maker_ata_a,
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}

pub fn setup_open_escrow(
    svm: &mut LiteSVM,
    maker: &Keypair,
    taker: &Keypair,
    mint_a: Pubkey,
    mint_b: Pubkey,
    seed: u64,
) -> (Pubkey, Pubkey) {
    setup_token_holders(svm, maker, taker, mint_a, mint_b);
    let escrow = escrow_pda(&maker.pubkey(), seed);
    let vault = ata(&escrow, &mint_a);
    send(
        svm,
        maker,
        &[maker],
        make_ix(
            maker.pubkey(),
            mint_a,
            mint_b,
            ata(&maker.pubkey(), &mint_a),
            escrow,
            vault,
            seed,
            DEPOSIT,
            RECEIVE,
        ),
    );
    (escrow, vault)
}
