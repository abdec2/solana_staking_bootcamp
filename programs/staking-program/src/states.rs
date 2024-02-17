use anchor_lang::prelude::*;

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

#[account]
pub struct Total {
    pub total_lockedup_rewards: u64,
    pub total_claimed_rewards: u64,
}
