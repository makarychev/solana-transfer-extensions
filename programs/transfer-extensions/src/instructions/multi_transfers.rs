use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::onchain::invoke_transfer_checked;
use anchor_spl::token_interface::{Mint, Token2022, TokenAccount};

use crate::errors::TransferExtensionsError;

#[derive(Accounts)]
pub struct MultiTransfers<'info> {
    #[account(mut,
      associated_token::token_program = token_program,
      associated_token::mint = mint,
      associated_token::authority = signer,
    )]
    source_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut,
      token::mint = mint,
      token::token_program = token_program,
    )]
    destination_account_1: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut,
      token::mint = mint,
      token::token_program = token_program,
    )]
    destination_account_2: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
      token::token_program = token_program,
    )]
    mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    signer: Signer<'info>,

    #[account(
      constraint = token_program.key() == anchor_spl::token_interface::spl_token_2022::id(),
    )]
    pub token_program: Program<'info, Token2022>,
}

pub fn multi_transfers<'info>(
    ctx: Context<'_, '_, '_, 'info, MultiTransfers<'info>>,
    amount1: u64,
    amount2: u64,
) -> Result<()> {
    msg!("Multi transfers");
    require!(
        amount1 > 0 && amount2 > 0,
        TransferExtensionsError::AmountMustBeGreaterThanZero
    );

    let mint = &ctx.accounts.mint;
    let decimals = mint.decimals;

    let split_at_pos = ctx.remaining_accounts.len() / 2;
    msg!("Invoke transfer 1");
    invoke_transfer_checked(
        ctx.accounts.token_program.key,
        ctx.accounts.source_account.to_account_info().clone(),
        mint.to_account_info().clone(),
        ctx.accounts.destination_account_1.to_account_info().clone(),
        ctx.accounts.signer.to_account_info().clone(),
        &ctx.remaining_accounts[..split_at_pos],
        amount1,
        decimals,
        &[],
    )?;

    let remaining_accounts2 = &ctx.remaining_accounts[split_at_pos..];

    msg!("Invoke transfer 2");
    invoke_transfer_checked(
        ctx.accounts.token_program.key,
        ctx.accounts.source_account.to_account_info().clone(),
        mint.to_account_info().clone(),
        ctx.accounts.destination_account_2.to_account_info().clone(),
        ctx.accounts.signer.to_account_info().clone(),
        remaining_accounts2,
        amount2,
        decimals,
        &[],
    )?;

    Ok(())
}
