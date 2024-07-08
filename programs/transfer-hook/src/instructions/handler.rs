use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};
use transfer_extensions::{
    program::TransferExtensions,
    states::{
        GlobalProgramData, MintCounterIn, MintCounterOut,
        WalletCounterIn, WalletCounterOut,
    },
};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct ExecuteTransferHook<'info> {
    #[account(
      token::mint = mint,
      token::token_program = anchor_spl::token_interface::spl_token_2022::id(),
    )]
    pub source_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
      token::token_program = anchor_spl::token_interface::spl_token_2022::id(),
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
      token::mint = mint,
      token::token_program = anchor_spl::token_interface::spl_token_2022::id(),
    )]
    pub destination_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: can be any account
    pub owner_delegate: UncheckedAccount<'info>,

    /// CHECK: meta list account
    #[account(
      seeds = [b"extra-account-metas", mint.key().as_ref()],
      bump,
    )]
    pub extra_metas_account: UncheckedAccount<'info>,

    pub additional_account_1: Program<'info, TransferExtensions>,

    pub wallet_counter_in_from: Account<'info, WalletCounterIn>,

    pub wallet_counter_in_to: Account<'info, WalletCounterIn>,

    pub wallet_counter_out_from: Account<'info, WalletCounterOut>,

    pub wallet_counter_out_to: Account<'info, WalletCounterOut>,

    pub mint_counter_in: Account<'info, MintCounterIn>,

    pub mint_counter_out: Account<'info, MintCounterOut>,

    pub global_program_data: Account<'info, GlobalProgramData>,
}

pub fn handler(ctx: Context<ExecuteTransferHook>, amount: u64) -> Result<()> {
    msg!("Executing transfer hook with amount: {:?}", amount);

    let wallet_counter_in_to = &mut ctx.accounts.wallet_counter_in_from;
    wallet_counter_in_to.transfers_count = wallet_counter_in_to.transfers_count.checked_add(1).unwrap();
    let wallet_counter_out_from = &mut ctx.accounts.wallet_counter_out_to;
    wallet_counter_out_from.transfers_count = wallet_counter_out_from.transfers_count.checked_add(1).unwrap();

    let mint_counter_in = &mut ctx.accounts.mint_counter_in;
    mint_counter_in.transfers_count = mint_counter_in.transfers_count.checked_add(1).unwrap();
    let mint_counter_out = &mut ctx.accounts.mint_counter_out;
    mint_counter_out.transfers_count = mint_counter_out.transfers_count.checked_add(1).unwrap();

    let global_program_data = &mut ctx.accounts.global_program_data;
    global_program_data.transfers_count = global_program_data.transfers_count.checked_add(1).unwrap();

    Ok(())
}
