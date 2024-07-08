use anchor_lang::prelude::*;

#[account()]
#[derive(Default, InitSpace)]
pub struct GlobalProgramData {
    pub transfers_count: u64,
}