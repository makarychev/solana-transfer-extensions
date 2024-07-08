use anchor_lang::prelude::*;
pub mod instructions;
use instructions::*;
pub mod states;
pub mod seeds;
pub mod errors;

declare_id!("4MNxsMM7niQkurWFyDvzhVbD3wHQFyAhnGjrvuYPi6Zu");

#[program]
pub mod transfer_extensions {
    use super::*;

    pub fn initialize_program_data(
        ctx: Context<InitializeProgramData>,
    ) -> Result<()> {
        instructions::initialize_program_data(ctx)
    }

    pub fn initialize_wallet_counter_in(
        ctx: Context<InitializeWalletCounterIn>,
    ) -> Result<()> {
        instructions::initialize_wallet_counter_in(ctx)
    }

    pub fn initialize_wallet_counter_out(
        ctx: Context<InitializeWalletCounterOut>,
    ) -> Result<()> {
        instructions::initialize_wallet_counter_out(ctx)
    }

    pub fn initialize_mint_counter_in(
        ctx: Context<InitializeMintCounterIn>,
    ) -> Result<()> {
        instructions::initialize_mint_counter_in(ctx)
    }

    pub fn initialize_mint_counter_out(
        ctx: Context<InitializeMintCounterOut>,
    ) -> Result<()> {
        instructions::initialize_mint_counter_out(ctx)
    }

    pub fn multi_transfers<'info>(
        ctx: Context<'_, '_, '_, 'info, MultiTransfers<'info>>,
        amount1: u64,
        amount2: u64,
    ) -> Result<()> {
        instructions::multi_transfers(ctx, amount1, amount2)
    }
}


