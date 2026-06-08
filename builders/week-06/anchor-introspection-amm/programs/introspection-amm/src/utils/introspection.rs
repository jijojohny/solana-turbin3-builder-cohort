use crate::AmmError;
use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use solana_instructions_sysvar::{
    load_current_index_checked, load_instruction_at_checked,
};

pub fn parse_previous_burn_lp(
    instruction_sysvar: &AccountInfo,
    depositor: Pubkey,
    pool: Pubkey,
    lp_mint: Pubkey,
    depositor_lp_ata: Pubkey,
) -> Result<u64> {
    let current_index = load_current_index_checked(instruction_sysvar)
        .map_err(|_| error!(AmmError::MissingBurnInstruction))?;
    require!(current_index > 0, AmmError::MissingBurnInstruction);

    let prev_ix = load_instruction_at_checked(current_index as usize - 1, instruction_sysvar)
        .map_err(|_| error!(AmmError::InvalidBurnInstruction))?;

    require_keys_eq!(prev_ix.program_id, crate::ID, AmmError::InvalidBurnInstruction);
    require!(
        prev_ix.data.len() >= 8 + 8,
        AmmError::InvalidBurnInstruction
    );
    require!(
        prev_ix.data[0..8] == crate::instruction::BurnLp::DISCRIMINATOR[..],
        AmmError::InvalidBurnInstruction
    );

    let lp_amount = u64::from_le_bytes(
        prev_ix.data[8..16]
            .try_into()
            .map_err(|_| error!(AmmError::InvalidBurnInstruction))?,
    );
    require!(lp_amount > 0, AmmError::ZeroAmount);

    let account = |index: usize| -> Result<Pubkey> {
        prev_ix
            .accounts
            .get(index)
            .map(|meta| meta.pubkey)
            .ok_or(error!(AmmError::InvalidBurnInstruction))
    };

    require_keys_eq!(account(0)?, depositor, AmmError::InvalidBurnInstruction);
    require_keys_eq!(account(3)?, pool, AmmError::InvalidBurnInstruction);
    require_keys_eq!(account(4)?, lp_mint, AmmError::InvalidBurnInstruction);
    require_keys_eq!(account(5)?, depositor_lp_ata, AmmError::InvalidBurnInstruction);
    require!(prev_ix.accounts[0].is_signer, AmmError::InvalidBurnInstruction);

    Ok(lp_amount)
}
