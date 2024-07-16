use anchor_lang::prelude::*;
// use anchor_spl::token_2022::spl_token_2022::onchain::invoke_transfer_checked;

use crate::{errors::TransferExtensionsError, sol_sdk::invoke_transfer_checked, MultiTransfers};

pub fn multi_transfers_heap<'info>(
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

    msg!("Invoke transfer 1");
    msg!("Source balance: {}", ctx.accounts.source_account.amount);
    invoke_transfer_checked(
        ctx.accounts.token_program.key,
        ctx.accounts.source_account.to_account_info().clone(),
        mint.to_account_info().clone(),
        ctx.accounts.destination_account_1.to_account_info().clone(),
        ctx.accounts.signer.to_account_info().clone(),
        &ctx.remaining_accounts,
        amount1,
        decimals,
        &[],
    )?;

    msg!("Invoke transfer 2");
    let mut heap_data: Box<[u8; 13_859]> = Box::new([0; 13_859]); // 13_859 - OK; 13_869 - FAILED
    heap_data[0] = 1;
    heap_data[10333] = 3;

    Ok(())
}
