use anchor_amm::constants::{LP_MINT_SEED, POOL_SEED};
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
pub const FEE_BPS: u16 = 30;
pub const DECIMALS: u8 = 6;

pub const INITIAL_A: u64 = 1_000_000;
pub const INITIAL_B: u64 = 2_000_000;
pub const SWAP_IN: u64 = 100_000;

pub fn setup_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    svm.add_program(
        anchor_amm::ID,
        include_bytes!("../../../../target/deploy/anchor_amm.so"),
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

pub fn sorted_mints(a: Pubkey, b: Pubkey) -> (Pubkey, Pubkey) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

pub fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address(owner, mint)
}

pub fn pool_pda(mint_a: &Pubkey, mint_b: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[POOL_SEED, mint_a.as_ref(), mint_b.as_ref()],
        &anchor_amm::ID,
    )
    .0
}

pub fn lp_mint_pda(pool: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[LP_MINT_SEED, pool.as_ref()], &anchor_amm::ID).0
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

pub fn mint_supply(svm: &LiteSVM, mint: Pubkey) -> u64 {
    let account = svm.get_account(&mint).expect("mint missing");
    Mint::unpack(&account.data).expect("invalid mint").supply
}

pub fn setup_user_tokens(
    svm: &mut LiteSVM,
    user: &Pubkey,
    mint_a: Pubkey,
    mint_b: Pubkey,
    amount_a: u64,
    amount_b: u64,
) {
    write_token_account(svm, ata(user, &mint_a), mint_a, *user, amount_a);
    write_token_account(svm, ata(user, &mint_b), mint_b, *user, amount_b);
}

pub fn initialize_ix(payer: Pubkey, mint_a: Pubkey, mint_b: Pubkey, fee_bps: u16) -> Instruction {
    let pool = pool_pda(&mint_a, &mint_b);
    let lp_mint = lp_mint_pda(&pool);
    Instruction::new_with_bytes(
        anchor_amm::ID,
        &anchor_amm::instruction::Initialize { fee_bps }.data(),
        anchor_amm::accounts::Initialize {
            payer,
            mint_a,
            mint_b,
            pool,
            lp_mint,
            vault_a: ata(&pool, &mint_a),
            vault_b: ata(&pool, &mint_b),
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}

pub fn deposit_ix(
    depositor: Pubkey,
    mint_a: Pubkey,
    mint_b: Pubkey,
    amount_a: u64,
    amount_b: u64,
    min_lp_out: u64,
) -> Instruction {
    let pool = pool_pda(&mint_a, &mint_b);
    let lp_mint = lp_mint_pda(&pool);
    Instruction::new_with_bytes(
        anchor_amm::ID,
        &anchor_amm::instruction::Deposit {
            amount_a,
            amount_b,
            min_lp_out,
        }
        .data(),
        anchor_amm::accounts::Deposit {
            depositor,
            mint_a,
            mint_b,
            pool,
            lp_mint,
            vault_a: ata(&pool, &mint_a),
            vault_b: ata(&pool, &mint_b),
            depositor_ata_a: ata(&depositor, &mint_a),
            depositor_ata_b: ata(&depositor, &mint_b),
            depositor_lp_ata: ata(&depositor, &lp_mint),
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}

pub fn withdraw_ix(
    depositor: Pubkey,
    mint_a: Pubkey,
    mint_b: Pubkey,
    lp_amount: u64,
    min_amount_a: u64,
    min_amount_b: u64,
) -> Instruction {
    let pool = pool_pda(&mint_a, &mint_b);
    let lp_mint = lp_mint_pda(&pool);
    Instruction::new_with_bytes(
        anchor_amm::ID,
        &anchor_amm::instruction::Withdraw {
            lp_amount,
            min_amount_a,
            min_amount_b,
        }
        .data(),
        anchor_amm::accounts::Withdraw {
            depositor,
            mint_a,
            mint_b,
            pool,
            lp_mint,
            vault_a: ata(&pool, &mint_a),
            vault_b: ata(&pool, &mint_b),
            depositor_ata_a: ata(&depositor, &mint_a),
            depositor_ata_b: ata(&depositor, &mint_b),
            depositor_lp_ata: ata(&depositor, &lp_mint),
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}

pub fn swap_a_for_b_ix(
    trader: Pubkey,
    mint_a: Pubkey,
    mint_b: Pubkey,
    amount_in: u64,
    min_amount_out: u64,
) -> Instruction {
    let pool = pool_pda(&mint_a, &mint_b);
    Instruction::new_with_bytes(
        anchor_amm::ID,
        &anchor_amm::instruction::SwapAForB {
            amount_in,
            min_amount_out,
        }
        .data(),
        anchor_amm::accounts::Swap {
            trader,
            mint_a,
            mint_b,
            pool,
            vault_a: ata(&pool, &mint_a),
            vault_b: ata(&pool, &mint_b),
            trader_ata_a: ata(&trader, &mint_a),
            trader_ata_b: ata(&trader, &mint_b),
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}

pub fn swap_b_for_a_ix(
    trader: Pubkey,
    mint_a: Pubkey,
    mint_b: Pubkey,
    amount_in: u64,
    min_amount_out: u64,
) -> Instruction {
    let pool = pool_pda(&mint_a, &mint_b);
    Instruction::new_with_bytes(
        anchor_amm::ID,
        &anchor_amm::instruction::SwapBForA {
            amount_in,
            min_amount_out,
        }
        .data(),
        anchor_amm::accounts::Swap {
            trader,
            mint_a,
            mint_b,
            pool,
            vault_a: ata(&pool, &mint_a),
            vault_b: ata(&pool, &mint_b),
            trader_ata_a: ata(&trader, &mint_a),
            trader_ata_b: ata(&trader, &mint_b),
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}

pub struct PoolSetup {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub pool: Pubkey,
    pub lp_mint: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
}

pub fn setup_initialized_pool(
    svm: &mut LiteSVM,
    payer: &Keypair,
    authority: &Pubkey,
) -> PoolSetup {
    let raw_a = Pubkey::new_unique();
    let raw_b = Pubkey::new_unique();
    let (mint_a, mint_b) = sorted_mints(raw_a, raw_b);

    write_mint(svm, mint_a, *authority, DECIMALS);
    write_mint(svm, mint_b, *authority, DECIMALS);

    send(
        svm,
        payer,
        &[payer],
        initialize_ix(payer.pubkey(), mint_a, mint_b, FEE_BPS),
    );

    let pool = pool_pda(&mint_a, &mint_b);
    PoolSetup {
        mint_a,
        mint_b,
        pool,
        lp_mint: lp_mint_pda(&pool),
        vault_a: ata(&pool, &mint_a),
        vault_b: ata(&pool, &mint_b),
    }
}

pub fn setup_pool_with_liquidity(
    svm: &mut LiteSVM,
    payer: &Keypair,
    depositor: &Keypair,
) -> PoolSetup {
    let setup = setup_initialized_pool(svm, payer, &payer.pubkey());
    setup_user_tokens(
        svm,
        &depositor.pubkey(),
        setup.mint_a,
        setup.mint_b,
        INITIAL_A * 2,
        INITIAL_B * 2,
    );
    send(
        svm,
        depositor,
        &[depositor],
        deposit_ix(
            depositor.pubkey(),
            setup.mint_a,
            setup.mint_b,
            INITIAL_A,
            INITIAL_B,
            1,
        ),
    );
    setup
}
