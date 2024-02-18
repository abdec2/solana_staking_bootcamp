// in this contract as given apy for all pool are not in decimals so we are not using apy in basis points

use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use solana_program::{clock::Clock, pubkey, pubkey::Pubkey};

declare_id!("GdA6TMYnwhhZbtHEypW7J89tpzAns3SnqRjc4d76ReVu");

pub mod constants;
pub mod errors;
pub mod events;
pub mod states;
use crate::{constants::*, errors::ErrorCode, events::*, states::*};

pub const OWNER_ADDRESS: Pubkey = pubkey!("4bRYs66kGxujekaRGHJjvjP4g7SCou28FZJ8LPDsyDnR");

#[program]
pub mod staking_program {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn create_pools(ctx: Context<CreatePools>, _apy: u8, _duration: u8) -> Result<()> {
        let pool_info = &mut ctx.accounts.pool_info_account;
        let clock = Clock::get()?;

        let timestamp: u64 = clock.unix_timestamp as u64;
        pool_info.apy = _apy;
        pool_info.duration = _duration as u64 * 30 * 24 * 60 * 60;
        pool_info.last_reward_time = timestamp;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, _amount: u64, _apy: u8, _duration: u8) -> Result<()> {
        let pool = &mut ctx.accounts.pool_info_account;
        let user = &mut ctx.accounts.user_info_account;
        let total_stats = &mut ctx.accounts.total_stats_account;
        let clock = Clock::get()?;

        update_pool(pool.as_mut());
        lock_pending_token(pool.as_mut(), user.as_mut(), total_stats.as_mut());

        let stake_amount = _amount
            .checked_mul(10u64.pow(ctx.accounts.mint.decimals as u32))
            .unwrap();

        if stake_amount > 0 {
            if user.amount == 0 {
                user.timestamp = clock.unix_timestamp as u64;
            }
            // implement Transfer token
            transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.user_token_account.to_account_info(),
                        to: ctx.accounts.stake_account.to_account_info(),
                        authority: ctx.accounts.signer.to_account_info(),
                    },
                ),
                stake_amount,
            )?;

            user.amount = user.amount.checked_add(stake_amount).unwrap();
            pool.total_supply = pool.total_supply.checked_add(stake_amount).unwrap();

            //emit deposit event
            emit!(DepositEvent {
                from: ctx.accounts.signer.key(),
                pool: pool.to_account_info().key(),
                amount: stake_amount,
                timestamp: clock.unix_timestamp
            });
        }

        user.reward_debt = user
            .amount
            .checked_mul(pool.acc_token_per_share)
            .and_then(|f| f.checked_div(10u64.pow(9 as u32)))
            .unwrap();

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, _amount: u64, _apy: u8, _duration: u8) -> Result<()> {
        let pool = &mut ctx.accounts.pool_info_account;
        let user = &mut ctx.accounts.user_info_account;
        let total_stats = &mut ctx.accounts.total_stats_account;
        let clock = Clock::get()?;

        let _amount_lamports = _amount
            .checked_mul(10u64.pow(ctx.accounts.mint.decimals as u32))
            .unwrap();

        require!(user.amount >= _amount_lamports, ErrorCode::InvalidAmount);

        let unstake_period = user.timestamp.checked_add(pool.duration).unwrap();

        // remove this code block for testing
        // require!(
        //     clock.unix_timestamp as u64 >= unstake_period,
        //     ErrorCode::StakingNotExpired
        // );
        // remove this code block for testing

        update_pool(pool.as_mut());
        lock_pending_token(pool.as_mut(), user.as_mut(), total_stats.as_mut());

        if _amount_lamports > 0 {
            user.amount = user.amount.checked_sub(_amount_lamports).unwrap();
            pool.total_supply = pool.total_supply.checked_sub(_amount_lamports).unwrap();

            // implemenet Transfer of stake amount
            let staker = ctx.accounts.signer.key();
            let pool_account = pool.to_account_info().key();
            let bump = ctx.bumps.stake_account;
            let signer: &[&[&[u8]]] =
                &[&[TOKEN_SEED, staker.as_ref(), pool_account.as_ref(), &[bump]]];

            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.stake_account.to_account_info(),
                        to: ctx.accounts.user_token_account.to_account_info(),
                        authority: ctx.accounts.stake_account.to_account_info(),
                    },
                    signer,
                ),
                _amount_lamports,
            )?;
        }

        if user.amount == 0 {
            user.timestamp = 0;
        }

        user.reward_debt = user
            .amount
            .checked_mul(pool.acc_token_per_share)
            .and_then(|f| f.checked_div(10u64.pow(9 as u32)))
            .unwrap();

        emit!(WithdrawEvent {
            from: ctx.accounts.signer.key(),
            pool: pool.to_account_info().key(),
            amount: _amount_lamports,
            timestamp: clock.unix_timestamp
        });

        Ok(())
    }

}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut, address = OWNER_ADDRESS)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed, 
        seeds = [VAULT_SEED],
        bump,
        payer = signer,
        token::mint = mint,
        token::authority = token_vault_account,
    )]
    pub token_vault_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed, 
        seeds = [TOTAL_STATS_SEED], 
        bump, 
        payer = signer,
        space = 8 + std::mem::size_of::<Total>()
    )]
    pub total_stats_account: Box<Account<'info, Total>>,

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
        seeds = [POOL_INFO_SEED, &[_apy].as_ref(), &[_duration].as_ref()], 
        bump, 
        payer = signer, 
        space = 8 + std::mem::size_of::<PoolInfo>()
    )]
    pub pool_info_account: Box<Account<'info, PoolInfo>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_amount: u64, _apy: u8, _duration: u8)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [TOTAL_STATS_SEED],
        bump,
    )]
    pub total_stats_account: Box<Account<'info, Total>>,

    #[account(
        mut, 
        seeds = [POOL_INFO_SEED, &[_apy].as_ref(), &[_duration].as_ref()], 
        bump, 
    )]
    pub pool_info_account: Box<Account<'info, PoolInfo>>,

    #[account(
        init_if_needed,
        seeds = [USER_INFO_SEED, signer.key().as_ref(), pool_info_account.to_account_info().key().as_ref()], 
        bump, 
        payer = signer, 
        space = 8 + std::mem::size_of::<UserInfo>()
    )]
    pub user_info_account: Box<Account<'info, UserInfo>>,

    #[account(
        init_if_needed,
        seeds = [TOKEN_SEED, signer.key().as_ref(), pool_info_account.to_account_info().key().as_ref()], 
        bump, 
        payer = signer, 
        token::mint = mint,
        token::authority = stake_account
    )]
    pub stake_account: Account<'info, TokenAccount>,
    #[account(
        mut, 
        associated_token::mint = mint, 
        associated_token::authority = signer,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_amount: u64, _apy: u8, _duration: u8)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [TOTAL_STATS_SEED],
        bump,
    )]
    pub total_stats_account: Box<Account<'info, Total>>,

    #[account(
        mut, 
        seeds = [POOL_INFO_SEED, &[_apy].as_ref(), &[_duration].as_ref()], 
        bump, 
    )]
    pub pool_info_account: Box<Account<'info, PoolInfo>>,

    #[account(
        mut,
        seeds = [USER_INFO_SEED, signer.key().as_ref(), pool_info_account.to_account_info().key().as_ref()], 
        bump, 
    )]
    pub user_info_account: Box<Account<'info, UserInfo>>,

    #[account(
        mut,
        seeds = [TOKEN_SEED, signer.key().as_ref(), pool_info_account.to_account_info().key().as_ref()], 
        bump, 
    )]
    pub stake_account: Account<'info, TokenAccount>,
    #[account(
        mut, 
        associated_token::mint = mint, 
        associated_token::authority = signer,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// internal function of the program
fn update_pool(pool_account: &mut Account<PoolInfo>) -> Result<()> {
    let clock = Clock::get()?;

    let timestamp: u64 = clock.unix_timestamp as u64;

    if timestamp <= pool_account.last_reward_time {
        return Ok(());
    }

    let lp_supply = pool_account.total_supply;

    if lp_supply == 0 {
        return Ok(());
    }

    let multiplier = getMultiplier(pool_account.last_reward_time, timestamp);
    let token_reward = &mut lp_supply
        .checked_mul(pool_account.apy as u64)
        .and_then(|f| f.checked_mul(multiplier))
        .and_then(|f| f.checked_div(100))
        .and_then(|f| f.checked_div(365 * 86400))
        .unwrap();

    let token_reward_changed = token_reward
        .checked_mul(10u64.pow(9 as u32))
        .and_then(|f| f.checked_div(lp_supply))
        .unwrap();
    pool_account.acc_token_per_share = pool_account
        .acc_token_per_share
        .checked_add(token_reward_changed)
        .unwrap();

    pool_account.last_reward_time = timestamp;

    Ok(())
}

fn lock_pending_token(
    pool: &mut Account<PoolInfo>,
    user: &mut Account<UserInfo>,
    total_stats: &mut Account<Total>,
) -> Result<()> {
    let pending = &mut user
        .amount
        .checked_mul(pool.acc_token_per_share)
        .and_then(|f| f.checked_div(10u64.pow(9 as u32)))
        .and_then(|f| f.checked_sub(user.reward_debt))
        .unwrap();

    user.reward_lockedup = user.reward_lockedup.checked_add(*pending).unwrap();
    total_stats.total_lockedup_rewards = total_stats
        .total_lockedup_rewards
        .checked_add(*pending)
        .unwrap();

    Ok(())
}

fn getMultiplier(from: u64, to: u64) -> u64 {
    let result = to.checked_sub(from).unwrap();
    result
}
