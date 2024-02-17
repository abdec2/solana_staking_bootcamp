use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use solana_program::clock::Clock;
use solana_program::{pubkey, pubkey::Pubkey};

declare_id!("4WExSBSHZDY4ZD83vhatop16D63yG2odScTzVifrYV1t");

pub mod constants {
    pub const VAULT_SEED: &[u8] = b"vault";
    pub const STAKE_INFO_SEED: &[u8] = b"stake_info";
    pub const POOL_INFO_SEED: &[u8] = b"pool_info";
    pub const TOKEN_SEED: &[u8] = b"token";
}

pub const OWNER_ADDRESS: Pubkey = pubkey!("4bRYs66kGxujekaRGHJjvjP4g7SCou28FZJ8LPDsyDnR");

#[program]
pub mod staking_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn create_pools(ctx: Context<CreatePools>, _apy: u8, _duration: u8) -> Result<()> {
        let pool_info = &mut ctx.accounts.pool_info_account;
        pool_info.apy = _apy;
        pool_info.duration = _duration as u64 * 30 * 24 * 60 * 60;
        Ok(())
    }

    pub fn test(ctx: Context<Test>) -> Result<()> {
        update_pool(ctx.accounts.pool_info_account.as_mut());
        Ok(())
    }

}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut, address = OWNER_ADDRESS)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed, 
        seeds = [constants::VAULT_SEED],
        bump,
        payer = signer,
        token::mint = mint,
        token::authority = token_vault_account,
    )]
    pub token_vault_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_apy: u8, _duration: u8)]
pub struct CreatePools<'info> {
    #[account(mut, address = OWNER_ADDRESS)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [constants::POOL_INFO_SEED, &[_apy].as_ref(), &[_duration].as_ref()], 
        bump, 
        payer = signer, 
        space = 8 + std::mem::size_of::<PoolInfo>()
    )]
    pub pool_info_account: Box<Account<'info, PoolInfo>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Test<'info> {
    #[account(mut, address = OWNER_ADDRESS)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub pool_info_account: Box<Account<'info, PoolInfo>>,
}

#[account]
pub struct PoolInfo {
    pub duration: u64,            // total lock period in seconds
    pub apy: u8,                  // apy
    pub last_reward_time: u64,    // Last block time that reward distribution occurs.
    pub acc_token_per_share: u64, // Accumulated reward per share, rewardPerTokenStored
    pub total_supply: u64,        // total supply in this pool
}

#[account]
pub struct UserInfo {
    pub amount: u64,
    pub reward_debt: u64,     // user reward per token paid
    pub reward_lockedup: u64, //rewards
    pub timestamp: u64,       //stake time
}

#[error_code]
pub enum ErrorCode {
    #[msg("Tokens are already staked.")]
    IsStaked,
    #[msg("Tokens not staked.")]
    NotStaked,
    #[msg("No Tokens to stake.")]
    NoTokens,
    #[msg("Staking period not expired")]
    StakingNotExpired,
    #[msg("Invalid staking period")]
    InvalidPeriod,
    #[msg("Not Allowed")]
    NotAllowed,
}

fn update_pool(account_info: &mut Account<PoolInfo>) -> Result<()> {
    let clock = Clock::get()?;
    account_info.last_reward_time = clock.unix_timestamp as u64;
    msg!("some var: {:?}", account_info.duration);
    Ok(())
}