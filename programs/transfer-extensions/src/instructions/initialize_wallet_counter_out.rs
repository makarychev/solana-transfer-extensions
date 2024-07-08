use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::{seeds::COUNTER_OUT_SEED, states::WalletCounterOut};


#[derive(Accounts)]
pub struct InitializeWalletCounterOut<'info> {
    #[account(init, payer = payer, space = 8 + WalletCounterOut::INIT_SPACE,
        seeds = [
          COUNTER_OUT_SEED,
          &associated_token_account.key().to_bytes(),
        ],
        bump
    )]
    pub counter_out: Account<'info, WalletCounterOut>,
    
    #[account(
      associated_token::token_program = anchor_spl::token_interface::spl_token_2022::id(),
      associated_token::mint = mint,
      associated_token::authority = user_wallet,
    )]
    pub associated_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
      token::token_program = anchor_spl::token_interface::spl_token_2022::id(),
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: User wallet address
    pub user_wallet: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}


pub fn initialize_wallet_counter_out(
    ctx: Context<InitializeWalletCounterOut>,
) -> Result<()> {
    let counter_out = &mut ctx.accounts.counter_out;
    counter_out.transfers_count = 0;
    counter_out.owner = ctx.accounts.user_wallet.key();

    Ok(())
}