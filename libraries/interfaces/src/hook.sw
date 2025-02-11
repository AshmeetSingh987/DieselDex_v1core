library;

use std::bytes::Bytes;
use ::data_structures::PoolId;

/// Hook for potential periphery features such as oracles etc.
abi IBaseHook {
    #[storage(read, write)]
    fn hook(
        pool_id: PoolId,
        sender: Identity,
        to: Identity,
        asset_0_in: u64,
        asset_1_in: u64,
        asset_0_out: u64,
        asset_1_out: u64,
        lp_token: u64,
    );
}

abi LPRewardsHook {
    #[storage(read, write)]
    fn on_mint(user: Identity, pool_id: PoolId, amount: u64);

    #[storage(read, write)]
    fn on_burn(user: Identity, pool_id: PoolId, amount: u64);

    #[storage(read)]
    fn get_user_rewards(user: Identity, pool_id: PoolId) -> u64;

    #[storage(read)]
    fn get_total_user_rewards(user: Identity) -> u64;

    #[storage(read)]
    fn get_user_reward_pools(user: Identity) -> Vec<PoolId>;
}