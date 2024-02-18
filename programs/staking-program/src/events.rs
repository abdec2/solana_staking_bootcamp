use anchor_lang::prelude::*;

#[event]
pub struct DepositEvent {
    pub from: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawEvent {
    pub from: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}
