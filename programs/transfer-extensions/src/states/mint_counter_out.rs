use anchor_lang::prelude::*;

#[account()]
#[derive(Default, InitSpace)]
pub struct MintCounterOut {
    pub transfers_count: u64,
    pub mint: Pubkey,
}