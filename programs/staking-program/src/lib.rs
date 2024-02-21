use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    // token::{transfer, Transfer},
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use solana_program::clock::Clock;

declare_id!("BdNMPi4nW22Gg1DNMoK9aX2P9Q6zw3HdkxfZ1rYJquac");

pub mod constants {
    pub const VAULT_SEED: &[u8] = b"vault";
    pub const STAKE_INFO_SEED: &[u8] = b"stake_info";
    pub const TOKEN_SEED: &[u8] = b"token";
}

#[program]
pub mod staking_program {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64, lock_period: u64) -> Result<()> {
        let stake_info = &mut ctx.accounts.stake_info_account;

        let check_period = match lock_period {
            1 | 3 | 6 | 9 | 12 => true,
            _ => false,
        };

        if !check_period {
            return Err(ErrorCode::InvalidPeriod.into());
        }

        let apy = match lock_period {
            1 => 1,
            3 => 4,
            6 => 8,
            9 => 12,
            12 => 18,
            _ => 0,
        };

        if stake_info.is_staked {
            return Err(ErrorCode::IsStaked.into());
        }

        if amount <= 0 {
            return Err(ErrorCode::NoTokens.into());
        }

        let clock = Clock::get()?;

        stake_info.stake_at = clock.unix_timestamp;
        stake_info.is_staked = true;
        stake_info.lock_period = lock_period * 30 * 24 * 60 * 60; // months converted into seconds
        stake_info.apy = apy;

        let stake_amount = (amount)
            .checked_mul(10u64.pow(ctx.accounts.mint.decimals as u32))
            .unwrap();

        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.stake_account.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            stake_amount,
            ctx.accounts.mint.decimals,
        )?;

        Ok(())
    }

    pub fn destake(ctx: Context<DeStake>) -> Result<()> {
        let stake_info = &mut ctx.accounts.stake_info_account;

        if !stake_info.is_staked {
            return Err(ErrorCode::NotStaked.into());
        }

        let clock = Clock::get()?;

        let slots_passed = (clock.unix_timestamp - stake_info.stake_at) as u64;

        if slots_passed < stake_info.lock_period {
            return Err(ErrorCode::StakingNotExpired.into());
        }

        let stake_amount = ctx.accounts.stake_account.amount;

        let apy = stake_info.apy;

        let per_year_reward = (stake_amount * apy as u64) / 100;

        let per_second_reward = (per_year_reward / (365 * 24 * 60 * 60)) as u64;

        let reward = slots_passed as u64 * per_second_reward;

        let bump = ctx.bumps.token_vault_account;
        let signer: &[&[&[u8]]] = &[&[constants::VAULT_SEED, &[bump]]];

        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.token_vault_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    authority: ctx.accounts.token_vault_account.to_account_info(),
                },
                signer,
            ),
            reward,
            ctx.accounts.mint.decimals,
        )?;

        let staker = ctx.accounts.signer.key();
        let bump = ctx.bumps.stake_account;
        let signer: &[&[&[u8]]] = &[&[constants::TOKEN_SEED, staker.as_ref(), &[bump]]];

        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.stake_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    authority: ctx.accounts.stake_account.to_account_info(),
                },
                signer,
            ),
            stake_amount,
            ctx.accounts.mint.decimals,
        )?;

        stake_info.is_staked = false;
        stake_info.stake_at = clock.unix_timestamp;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed, 
        seeds = [constants::VAULT_SEED],
        bump,
        payer = signer,
        token::mint = mint,
        token::authority = token_vault_account,
    )]
    pub token_vault_account: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [constants::STAKE_INFO_SEED, signer.key.as_ref()], 
        bump, 
        payer = signer, 
        space = 8 + std::mem::size_of::<StakeInfo>()
    )]
    pub stake_info_account: Account<'info, StakeInfo>,

    #[account(
        init_if_needed,
        seeds = [constants::TOKEN_SEED, signer.key.as_ref()], 
        bump, 
        payer = signer, 
        token::mint = mint,
        token::authority = stake_account
    )]
    pub stake_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account: UncheckedAccount<'info>,

    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DeStake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut, 
        seeds = [constants::VAULT_SEED],
        bump,
    )]
    pub token_vault_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [constants::STAKE_INFO_SEED, signer.key.as_ref()], 
        bump, 
    )]
    pub stake_info_account: Account<'info, StakeInfo>,

    #[account(
        mut,
        seeds = [constants::TOKEN_SEED, signer.key.as_ref()], 
        bump,
    )]
    pub stake_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account: UncheckedAccount<'info>,

    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct StakeInfo {
    pub stake_at: i64,
    pub is_staked: bool,
    pub lock_period: u64, // in months
    pub apy: u8,          // Annual Percentage Yield
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
}
