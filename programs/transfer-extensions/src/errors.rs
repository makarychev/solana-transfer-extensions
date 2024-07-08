use anchor_lang::prelude::*;

#[error_code]
pub enum TransferExtensionsError {
  #[msg("Amount must be greater than 0")]
  AmountMustBeGreaterThanZero,
}