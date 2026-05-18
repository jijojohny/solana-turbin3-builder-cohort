use litesvm::LiteSVM;
use sha2::{Digest, Sha256};
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use solana_transaction::versioned::VersionedTransaction;

pub const PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("11111111111111111111111111111112");
pub const LAMPORTS: u64 = 10_000_000_000;
pub const DEPOSIT_AMOUNT: u64 = 1_000_000_000;

pub fn setup_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    svm.add_program(
        PROGRAM_ID,
        include_bytes!("../../target/deploy/anchorvault.so"),
    )
    .unwrap();
    svm
}

pub fn sighash(name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{name}").as_bytes());
    let hash = hasher.finalize();
    let mut out = [0u8; 8];
    out.copy_from_slice(&hash[..8]);
    out
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

/// Initialize vault for `user` and return PDAs.
pub fn setup_initialized_vault(svm: &mut LiteSVM, user: &Keypair) -> (Pubkey, Pubkey) {
    svm.airdrop(&user.pubkey(), LAMPORTS).unwrap();
    let vault_state = vault_state_pda(&user.pubkey());
    let vault = vault_pda(&vault_state);
    send(
        svm,
        user,
        &[user],
        initialize_ix(user.pubkey(), vault_state, vault),
    );
    (vault_state, vault)
}

pub fn vault_state_pda(user: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"state", user.as_ref()], &PROGRAM_ID).0
}

pub fn vault_pda(vault_state: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault", vault_state.as_ref()], &PROGRAM_ID).0
}

pub fn read_vault_state(svm: &LiteSVM, vault_state: Pubkey) -> (u8, u8) {
    let account = svm.get_account(&vault_state).expect("vault_state missing");
    let data = &account.data[8..];
    (data[0], data[1])
}

pub fn initialize_ix(user: Pubkey, vault_state: Pubkey, vault: Pubkey) -> Instruction {
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault_state, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: sighash("initialize").to_vec(),
    }
}

pub fn deposit_ix(user: Pubkey, vault: Pubkey, vault_state: Pubkey, amount: u64) -> Instruction {
    let mut data = sighash("deposit").to_vec();
    data.extend_from_slice(&amount.to_le_bytes());
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault, false),
            AccountMeta::new(vault_state, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

pub fn withdraw_ix(user: Pubkey, vault: Pubkey, vault_state: Pubkey, amount: u64) -> Instruction {
    let mut data = sighash("withdraw").to_vec();
    data.extend_from_slice(&amount.to_le_bytes());
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault, false),
            AccountMeta::new(vault_state, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

pub fn close_ix(user: Pubkey, vault: Pubkey, vault_state: Pubkey) -> Instruction {
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault, false),
            AccountMeta::new(vault_state, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: sighash("close").to_vec(),
    }
}
