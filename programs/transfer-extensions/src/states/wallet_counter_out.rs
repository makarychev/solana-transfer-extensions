use anchor_lang::prelude::*;

#[account()]
#[derive(Default, InitSpace)]
pub struct WalletCounterOut {
    pub transfers_count: u64,
    pub owner: Pubkey,
}