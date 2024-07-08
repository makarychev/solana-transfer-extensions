use anchor_lang::prelude::*;

declare_id!("14KA3wb3jtHft5MLy59VCJAAVDbCAduDydUAKDCEnipV");

pub mod instructions;
use instructions::*;

#[program]
pub mod transfer_hook {
    use super::*;

    /// execute transfer hook
    #[interface(spl_transfer_hook_interface::execute)]
    pub fn execute_transaction(ctx: Context<ExecuteTransferHook>, amount: u64) -> Result<()> {
        instructions::handler(ctx, amount)
    }

    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        instructions::initialize_extra_account_meta_list(ctx)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
