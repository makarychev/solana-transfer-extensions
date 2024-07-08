use anchor_lang::prelude::*;

use crate::{seeds::GLOBAL_PROGRAM_DATA_SEED, states::GlobalProgramData};


#[derive(Accounts)]
pub struct InitializeProgramData<'info> {
    #[account(init, payer = payer, space = 8 + GlobalProgramData::INIT_SPACE,
        seeds = [GLOBAL_PROGRAM_DATA_SEED],
        bump
    )]
    pub program_counter: Account<'info, GlobalProgramData>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}


pub fn initialize_program_data(
    _ctx: Context<InitializeProgramData>,
) -> Result<()> {
    msg!("Greetings from: {:?}", _ctx.program_id);
    Ok(())
}