use anchor_lang::prelude::*;

#[account()]
#[derive(Default, InitSpace)]
pub struct MintCounterIn {
    pub transfers_count: u64,
    pub mint: Pubkey,
}