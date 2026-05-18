use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::system_instruction;

use crate::state::VaultState;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        seeds = [b"state", user.key().as_ref()],
        bump,
        space = 8 + VaultState::INIT_SPACE,
    )]
    pub vault_state: Account<'info, VaultState>,

    /// CHECK: Vault PDA created in the instruction handler.
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.vault_state.set_inner(VaultState {
            vault_bump: bumps.vault,
            state_bump: bumps.vault_state,
        });

        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(0);
        let vault_state_key = self.vault_state.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault",
            vault_state_key.as_ref(),
            &[bumps.vault],
        ]];

        invoke_signed(
            &system_instruction::create_account(
                self.user.key,
                self.vault.key,
                lamports,
                0,
                &anchor_lang::solana_program::system_program::ID,
            ),
            &[
                self.user.to_account_info(),
                self.vault.to_account_info(),
                self.system_program.to_account_info(),
            ],
            signer_seeds,
        )?;

        Ok(())
    }
}
