use anchor_lang::prelude::*;
use anchor_lang::{
    prelude::Result,
    solana_program::{program::invoke, pubkey::Pubkey, system_instruction::transfer},
    Lamports,
};
use anchor_spl::{token_2022::ID as TOKEN_2022_PROGRAM_ID, token_interface::Mint};
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_tlv_account_resolution::{account::ExtraAccountMeta, seeds::Seed};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;
use transfer_extensions::program::TransferExtensions;
use transfer_extensions::seeds::{COUNTER_IN_SEED, COUNTER_OUT_SEED, GLOBAL_PROGRAM_DATA_SEED};

pub const META_LIST_ACCOUNT_SEED: &[u8] = b"extra-account-metas";

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(
      init,
      space = get_meta_list_size(account_manager_program.key)?,
      seeds = [
        META_LIST_ACCOUNT_SEED,
        mint.key().as_ref(),
      ],
      bump,
      payer = payer,
    )]
    /// CHECK: extra metas account
    pub extra_metas_account: UncheckedAccount<'info>,

    #[account(
        mint::token_program = TOKEN_2022_PROGRAM_ID,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account()]
    pub account_manager_program: Program<'info, TransferExtensions>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_extra_account_meta_list(
    ctx: Context<InitializeExtraAccountMetaList>,
) -> Result<()> {
    let extra_metas_account = &ctx.accounts.extra_metas_account;
    let metas = get_extra_account_metas(ctx.accounts.account_manager_program.key)?;
    let mut data = extra_metas_account.try_borrow_mut_data()?;
    ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &metas)?;

    Ok(())
}

pub fn get_meta_list_size(program_id: &Pubkey) -> Result<usize> {
    Ok(ExtraAccountMetaList::size_of(get_extra_account_metas(program_id)?.len()).unwrap())
}

pub fn get_extra_account_metas(program_id: &Pubkey) -> Result<Vec<ExtraAccountMeta>> {
    Ok(vec![
        // [index 5, 0] account manager program id
        ExtraAccountMeta::new_with_pubkey(
            program_id,
            false, // is_signer
            false, // is_writable
        )?,
        // [index 6, 1] counter in from
        ExtraAccountMeta::new_external_pda_with_seeds(
            5,
            &[
                Seed::Literal {
                    bytes: COUNTER_IN_SEED.to_vec(),
                },
                Seed::AccountKey { index: 0 },
            ],
            false,
            false,
        )?,
        // [index 7, 2]  counter in to
        ExtraAccountMeta::new_external_pda_with_seeds(
            5,
            &[
                Seed::Literal {
                    bytes: COUNTER_IN_SEED.to_vec(),
                },
                Seed::AccountKey { index: 2 },
            ],
            false,
            false,
        )?,
        // [index 8, 3] counter out from
        ExtraAccountMeta::new_external_pda_with_seeds(
            5,
            &[
                Seed::Literal {
                    bytes: COUNTER_OUT_SEED.to_vec(),
                },
                Seed::AccountKey { index: 0 },
            ],
            false,
            false,
        )?,
        // [index 9, 4] counter out to
        ExtraAccountMeta::new_external_pda_with_seeds(
            5,
            &[
                Seed::Literal {
                    bytes: COUNTER_OUT_SEED.to_vec(),
                },
                Seed::AccountKey { index: 2 },
            ],
            false,
            false,
        )?,
        // [index 10, 5] counter in mint
        ExtraAccountMeta::new_external_pda_with_seeds(
            5,
            &[
                Seed::Literal {
                    bytes: COUNTER_IN_SEED.to_vec(),
                },
                Seed::AccountKey { index: 1 },
            ],
            false,
            false,
        )?,
        // [index 11, 6] counter out mint
        ExtraAccountMeta::new_external_pda_with_seeds(
            5,
            &[
                Seed::Literal {
                    bytes: COUNTER_OUT_SEED.to_vec(),
                },
                Seed::AccountKey { index: 1 },
            ],
            false,
            false,
        )?,
        // [index 12, 7] global program data
        ExtraAccountMeta::new_external_pda_with_seeds(
            5,
            &[Seed::Literal {
                bytes: GLOBAL_PROGRAM_DATA_SEED.to_vec(),
            }],
            false,
            false,
        )?,
    ])
}

pub fn update_account_lamports_to_minimum_balance<'info>(
    account: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
) -> Result<()> {
    let extra_lamports = Rent::get()?.minimum_balance(account.data_len()) - account.get_lamports();
    if extra_lamports > 0 {
        invoke(
            &transfer(payer.key, account.key, extra_lamports),
            &[payer, account, system_program],
        )?;
    }
    Ok(())
}
