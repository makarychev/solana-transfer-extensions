use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{seeds::COUNTER_OUT_SEED, states::MintCounterOut};


#[derive(Accounts)]
pub struct InitializeMintCounterOut<'info> {
    #[account(init, payer = payer, space = 8 + MintCounterOut::INIT_SPACE,
        seeds = [
          COUNTER_OUT_SEED,
          &mint.key().to_bytes(),
        ],
        bump
    )]
    pub counter_in: Account<'info, MintCounterOut>,
    
    #[account(
      token::token_program = anchor_spl::token_interface::spl_token_2022::id(),
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}


pub fn initialize_mint_counter_out(
    ctx: Context<InitializeMintCounterOut>,
) -> Result<()> {
    let counter_in = &mut ctx.accounts.counter_in;
    counter_in.transfers_count = 0;
    counter_in.mint = ctx.accounts.mint.key();

    Ok(())
}