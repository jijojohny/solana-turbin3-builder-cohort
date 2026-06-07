use crate::constants::BPS_DENOMINATOR;
use crate::MarketplaceError;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

pub fn split_payment(total: u64, fee_bps: u16) -> Result<(u64, u64)> {
    let fee = total
        .checked_mul(fee_bps as u64)
        .ok_or(MarketplaceError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(MarketplaceError::MathOverflow)?;
    let maker_amount = total
        .checked_sub(fee)
        .ok_or(MarketplaceError::MathOverflow)?;
    Ok((maker_amount, fee))
}

pub fn pay_sol<'info>(
    buyer: &AccountInfo<'info>,
    maker: &AccountInfo<'info>,
    treasury: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    total: u64,
    fee_bps: u16,
) -> Result<()> {
    let (maker_amount, fee) = split_payment(total, fee_bps)?;

    if maker_amount > 0 {
        transfer(
            CpiContext::new(
                system_program.key(),
                Transfer {
                    from: buyer.clone(),
                    to: maker.clone(),
                },
            ),
            maker_amount,
        )?;
    }

    if fee > 0 {
        transfer(
            CpiContext::new(
                system_program.key(),
                Transfer {
                    from: buyer.clone(),
                    to: treasury.clone(),
                },
            ),
            fee,
        )?;
    }

    Ok(())
}

pub fn pay_sol_from_pda<'info>(
    escrow: &AccountInfo<'info>,
    maker: &AccountInfo<'info>,
    treasury: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    total: u64,
    fee_bps: u16,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let (maker_amount, fee) = split_payment(total, fee_bps)?;
    let rent = Rent::get()?.minimum_balance(escrow.data_len());
    let available = escrow
        .lamports()
        .checked_sub(rent)
        .ok_or(MarketplaceError::MathOverflow)?;
    require!(available >= total, MarketplaceError::MathOverflow);

    if maker_amount > 0 {
        transfer(
            CpiContext::new_with_signer(
                system_program.key(),
                Transfer {
                    from: escrow.clone(),
                    to: maker.clone(),
                },
                signer_seeds,
            ),
            maker_amount,
        )?;
    }

    if fee > 0 {
        transfer(
            CpiContext::new_with_signer(
                system_program.key(),
                Transfer {
                    from: escrow.clone(),
                    to: treasury.clone(),
                },
                signer_seeds,
            ),
            fee,
        )?;
    }

    Ok(())
}

pub fn pay_tokens<'info>(
    buyer_ata: &InterfaceAccount<'info, TokenAccount>,
    maker_ata: &InterfaceAccount<'info, TokenAccount>,
    treasury_ata: &InterfaceAccount<'info, TokenAccount>,
    payment_mint: &InterfaceAccount<'info, Mint>,
    buyer: &Signer<'info>,
    token_program: &Interface<'info, TokenInterface>,
    total: u64,
    fee_bps: u16,
) -> Result<()> {
    let (maker_amount, fee) = split_payment(total, fee_bps)?;
    let decimals = payment_mint.decimals;

    if maker_amount > 0 {
        token_interface::transfer_checked(
            CpiContext::new(
                token_program.key(),
                TransferChecked {
                    from: buyer_ata.to_account_info(),
                    mint: payment_mint.to_account_info(),
                    to: maker_ata.to_account_info(),
                    authority: buyer.to_account_info(),
                },
            ),
            maker_amount,
            decimals,
        )?;
    }

    if fee > 0 {
        token_interface::transfer_checked(
            CpiContext::new(
                token_program.key(),
                TransferChecked {
                    from: buyer_ata.to_account_info(),
                    mint: payment_mint.to_account_info(),
                    to: treasury_ata.to_account_info(),
                    authority: buyer.to_account_info(),
                },
            ),
            fee,
            decimals,
        )?;
    }

    Ok(())
}
