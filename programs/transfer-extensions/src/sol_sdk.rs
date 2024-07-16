use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::extension::{transfer_hook, StateWithExtensions};
use anchor_spl::token_2022::spl_token_2022::instruction;
use anchor_spl::token_2022::spl_token_2022::solana_program::entrypoint::ProgramResult;
use anchor_spl::token_2022::spl_token_2022::solana_program::program::invoke_signed;
use anchor_spl::token_interface::spl_pod::slice::PodSlice;
use solana_program::instruction::Instruction;
use spl_tlv_account_resolution::account::ExtraAccountMeta;
use spl_tlv_account_resolution::error::AccountResolutionError;
use spl_transfer_hook_interface::error::TransferHookError;
use spl_transfer_hook_interface::get_extra_account_metas_address;
use spl_type_length_value::state::{TlvState, TlvStateBorrowed};

/// Helper to CPI into token-2022 on-chain, looking through the additional
/// account infos to create the proper instruction with the proper account infos
#[allow(clippy::too_many_arguments)]
pub fn invoke_transfer_checked<'a>(
    token_program_id: &Pubkey,
    source_info: AccountInfo<'a>,
    mint_info: AccountInfo<'a>,
    destination_info: AccountInfo<'a>,
    authority_info: AccountInfo<'a>,
    additional_accounts: &[AccountInfo<'a>],
    amount: u64,
    decimals: u8,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let mut cpi_instruction = instruction::transfer_checked(
        token_program_id,
        source_info.key,
        mint_info.key,
        destination_info.key,
        authority_info.key,
        &[], // add them later, to avoid unnecessary clones
        amount,
        decimals,
    )?;

    let mut cpi_account_infos = vec![
        source_info.clone(),
        mint_info.clone(),
        destination_info.clone(),
        authority_info.clone(),
    ];

    // if it's a signer, it might be a multisig signer, throw it in!
    additional_accounts
        .iter()
        .filter(|ai| ai.is_signer)
        .for_each(|ai| {
            cpi_account_infos.push(ai.clone());
            cpi_instruction
                .accounts
                .push(AccountMeta::new_readonly(*ai.key, ai.is_signer));
        });
    msg!("====!! Going to add extra accounts !!====");
    // scope the borrowing to avoid a double-borrow during CPI
    {
        let mint_data = mint_info.try_borrow_data()?;
        let mint =
            StateWithExtensions::<anchor_spl::token_2022::spl_token_2022::state::Mint>::unpack(
                &mint_data,
            )?;
        if let Some(program_id) = transfer_hook::get_program_id(&mint) {
            add_extra_accounts_for_execute_cpi(
                &mut cpi_instruction,
                &mut cpi_account_infos,
                &program_id,
                source_info,
                mint_info.clone(),
                destination_info,
                authority_info,
                amount,
                additional_accounts,
            )?;
        }
    }
    msg!("====!! Going to invoke_signed !!====");

    invoke_signed(&cpi_instruction, &cpi_account_infos, seeds)
}

#[allow(clippy::too_many_arguments)]
pub fn add_extra_accounts_for_execute_cpi<'a>(
    cpi_instruction: &mut Instruction,
    cpi_account_infos: &mut Vec<AccountInfo<'a>>,
    program_id: &Pubkey,
    source_info: AccountInfo<'a>,
    mint_info: AccountInfo<'a>,
    destination_info: AccountInfo<'a>,
    authority_info: AccountInfo<'a>,
    amount: u64,
    additional_accounts: &[AccountInfo<'a>],
) -> ProgramResult {
    msg!("\t====!! Inside add_extra_accounts_for_execute_cpi !!====");
    let validate_state_pubkey = get_extra_account_metas_address(mint_info.key, program_id);
    let validate_state_info = additional_accounts
        .iter()
        .find(|&x| *x.key == validate_state_pubkey)
        .ok_or(TransferHookError::IncorrectAccount)?;

    let program_info = additional_accounts
        .iter()
        .find(|&x| x.key == program_id)
        .ok_or(TransferHookError::IncorrectAccount)?;

    let mut execute_instruction = spl_transfer_hook_interface::instruction::execute(
        program_id,
        source_info.key,
        mint_info.key,
        destination_info.key,
        authority_info.key,
        &validate_state_pubkey,
        amount,
    );
    // Vec::with_capacity();
    let mut execute_account_infos = vec![
        source_info,
        mint_info,
        destination_info,
        authority_info,
        validate_state_info.clone(),
    ];

    msg!("\t====!! ExtraAccountMetaList::add_to_cpi_instruction !!====");
    // NOTE: Replaces sdk function with the same name custom implementation (now same impl as in SDK)
    // ExtraAccountMetaList::add_to_cpi_instruction::<spl_transfer_hook_interface::instruction::ExecuteInstruction>(
    add_to_cpi_instruction(
        &mut execute_instruction,
        &mut execute_account_infos,
        &validate_state_info.try_borrow_data()?,
        additional_accounts,
    )?;

    msg!("====!! Adding accounts from execute_instruction !!====");
    // Add only the extra accounts resolved from the validation state
    cpi_instruction
        .accounts
        .extend_from_slice(&execute_instruction.accounts[5..]);
    cpi_account_infos.extend_from_slice(&execute_account_infos[5..]);

    // Add the program id and validation state account
    msg!("\t====!! Adding program_id and validate_state_pubkey !!====");
    cpi_instruction
        .accounts
        .push(AccountMeta::new_readonly(*program_id, false));
    cpi_instruction
        .accounts
        .push(AccountMeta::new_readonly(validate_state_pubkey, false));
    cpi_account_infos.push(program_info.clone());
    cpi_account_infos.push(validate_state_info.clone());

    Ok(())
}

/// Add the additional account metas and account infos for a CPI
pub fn add_to_cpi_instruction<'a>(
    cpi_instruction: &mut Instruction,
    cpi_account_infos: &mut Vec<AccountInfo<'a>>,
    data: &[u8],
    account_infos: &[AccountInfo<'a>],
) -> std::result::Result<(), ProgramError> {
    msg!("??? inside ::add_to_cpi_instruction !!====");
    let state = TlvStateBorrowed::unpack(data)?;
    let bytes =
        state.get_first_bytes::<spl_transfer_hook_interface::instruction::ExecuteInstruction>()?;
    let extra_account_metas = PodSlice::<ExtraAccountMeta>::unpack(bytes)?;

    msg!(   
        "extra_account_metas.data() = {}",
        extra_account_metas.data().len()
    );
    // let mut it_idx = 0;
    // cpi_account_infos.iter().for_each(|x| {
    //     msg!(
    //         "account index [{}], data len: {}",
    //         it_idx,
    //         x.try_borrow_data().unwrap().len()
    //     );
    //     it_idx += 1;
    // });
    let mut idx = 5;
    for extra_meta in extra_account_metas.data().iter() {
        let mut meta = {
            // Create a list of `Ref`s so we can reference account data in the
            // resolution step
            let account_key_data_refs = cpi_account_infos
                .iter()
                .map(|info| {
                    let key = *info.key;
                    let data = info.try_borrow_data()?;
                    // msg!("key = {}, data length = {}", key, data.len());
                    // msg!("data = {:?}", data); // a lot of compute units
                    Ok((key, data))
                })
                .collect::<std::result::Result<Vec<_>, ProgramError>>()?;

            extra_meta.resolve(
                &cpi_instruction.data,
                &cpi_instruction.program_id,
                |usize| {
                    account_key_data_refs
                        .get(usize)
                        .map(|(pubkey, opt_data)| (pubkey, Some(opt_data.as_ref())))
                },
            )?
        };
        // msg!("de_escalate_account_meta");
        de_escalate_account_meta(&mut meta, &cpi_instruction.accounts);

        let account_info = account_infos
            .iter()
            .find(|&x| *x.key == meta.pubkey)
            .ok_or(AccountResolutionError::IncorrectAccount)?
            .clone();

        // msg!("====>>> [going to push meta to cpi_instruction.accounts and account_info to cpi_account_infos]");
        cpi_instruction.accounts.push(meta);
        // msg!(
        //     "account index [{}], data len: {}",
        //     idx,
        //     account_info.try_borrow_data()?.len()
        // );
        cpi_account_infos.push(account_info);
        idx += 1;
    }
    Ok(())
}

// /// Rewrite of the `add_to_cpi_instruction` function from the SDK
// pub fn add_to_cpi_instruction<'a>(
//     cpi_instruction: &mut Instruction,
//     cpi_account_infos: &mut Vec<AccountInfo<'a>>,
//     data: &[u8],
//     account_infos: &[AccountInfo<'a>],
// ) -> std::result::Result<(), ProgramError> {
//     msg!("??? inside ::add_to_cpi_instruction !!====");
//      let state = TlvStateBorrowed::unpack(data)?;
//     let bytes =
//         state.get_first_bytes::<spl_transfer_hook_interface::instruction::ExecuteInstruction>()?;
//     let extra_account_metas = PodSlice::<ExtraAccountMeta>::unpack(bytes)?;

//     msg!("extra_account_metas.data() = {}", extra_account_metas.data().len());
//     // Create a list of `Ref`s so we can reference account data in the
//     // resolution step
//     let mut acc_info = cpi_account_infos.clone();
//     let cpi_account_infos_len = cpi_account_infos.len();
//     {
//         let account_key_data_refs = cpi_account_infos
//             .iter()
//             .map(|info| {
//                 let key = *info.key;
//                 let data = info.try_borrow_data()?;
//                 // msg!("key = {}, data length = {}", key, data.len());
//                 // msg!("data = {:?}", data); // a lot of compute units
//                 Ok((key, data))
//             })
//             .collect::<std::result::Result<Vec<_>, ProgramError>>()?;
//         for extra_meta in extra_account_metas.data().iter() {
//             let mut meta = {
//                 extra_meta.resolve(
//                     &cpi_instruction.data,
//                     &cpi_instruction.program_id,
//                     |usize| {
//                         account_key_data_refs
//                             .get(usize)
//                             .map(|(pubkey, opt_data)| (pubkey, Some(opt_data.as_ref())))
//                     },
//                 )?
//             };
//             msg!("de_escalate_account_meta");
//             de_escalate_account_meta(&mut meta, &cpi_instruction.accounts);

//             let account_info = account_infos
//                 .iter()
//                 .find(|&x| *x.key == meta.pubkey)
//                 .ok_or(AccountResolutionError::IncorrectAccount)?
//                 .clone();

//             msg!("====>>> [going to push meta to cpi_instruction.accounts and account_info to cpi_account_infos]");
//             cpi_instruction.accounts.push(meta);
//             account_key_data_refs.push((*account_info.key, account_info.try_borrow_data()?));
//             acc_info.push(account_info);
//         }
//     }
//     cpi_account_infos.extend_from_slice(&acc_info[cpi_account_infos_len..]);
//     Ok(())
// }

/// De-escalate an account meta if necessary
fn de_escalate_account_meta(account_meta: &mut AccountMeta, account_metas: &[AccountMeta]) {
    // This is a little tricky to read, but the idea is to see if
    // this account is marked as writable or signer anywhere in
    // the instruction at the start. If so, DON'T escalate it to
    // be a writer or signer in the CPI
    let maybe_highest_privileges = account_metas
        .iter()
        .filter(|&x| x.pubkey == account_meta.pubkey)
        .map(|x| (x.is_signer, x.is_writable))
        .reduce(|acc, x| (acc.0 || x.0, acc.1 || x.1));
    // If `Some`, then the account was found somewhere in the instruction
    if let Some((is_signer, is_writable)) = maybe_highest_privileges {
        if !is_signer && is_signer != account_meta.is_signer {
            // Existing account is *NOT* a signer already, but the CPI
            // wants it to be, so de-escalate to not be a signer
            account_meta.is_signer = false;
        }
        if !is_writable && is_writable != account_meta.is_writable {
            // Existing account is *NOT* writable already, but the CPI
            // wants it to be, so de-escalate to not be writable
            account_meta.is_writable = false;
        }
    }
}
