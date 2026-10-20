#![cfg(test)]

use soroban_sdk::{
    contract, contractimpl, testutils::Address as _,
    Address, Env, Symbol,
};

use crate::{RiskPool, RiskPoolClient};

#[contract]
pub struct MaliciousToken;

#[contractimpl]
impl MaliciousToken {
    pub fn set_pool(env: Env, pool: Address) {
        env.storage().instance().set(&Symbol::new(&env, "pool"), &pool);
    }

    pub fn transfer(env: Env, from: Address, _to: Address, amount: i128) {
        // Try to re-enter risk pool deposit during token transfer
        if let Some(pool_addr) = env.storage().instance().get::<_, Address>(&Symbol::new(&env, "pool")) {
            let pool = RiskPoolClient::new(&env, &pool_addr);
            // This should trigger the reentrancy guard
            pool.deposit(&from, &amount, &0, &false);
        }
    }

    pub fn balance(_env: Env, _id: Address) -> i128 {
        1_000_000_000_000
    }
}

#[test]
fn reentrancy_guard_blocks_recursive_call() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let backstop = Address::generate(&env);
    let lp = Address::generate(&env);
    let policy_engine = Address::generate(&env);
    let claims_processor = Address::generate(&env);

    let mal_token_id = env.register(MaliciousToken, ());
    let mal_client = MaliciousTokenClient::new(&env, &mal_token_id);

    let pool_id = env.register(RiskPool, ());
    let pool = RiskPoolClient::new(&env, &pool_id);

    mal_client.set_pool(&pool_id);

    pool.initialize(
        &admin,
        &mal_token_id,
        &treasury,
        &backstop,
        &Symbol::new(&env, "crop"),
        &policy_engine,
        &claims_processor,
    );

    // Calling deposit triggers token transfer which tries to re-enter deposit.
    // The re-entrant call fails with Error::Reentrancy (#47), causing the invocation to fail.
    let res = pool.try_deposit(&lp, &100_000_000, &0, &false);
    assert!(res.is_err(), "Reentrancy attack should be blocked by guard");
}

#[test]
fn reentrancy_guard_resets_after_normal_call() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let backstop = Address::generate(&env);
    let lp = Address::generate(&env);
    let policy_engine = Address::generate(&env);
    let claims_processor = Address::generate(&env);

    let usdc_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let usdc_admin = soroban_sdk::token::StellarAssetClient::new(&env, &usdc_id);
    usdc_admin.mint(&lp, &10_000_000_000);

    let pool_id = env.register(RiskPool, ());
    let pool = RiskPoolClient::new(&env, &pool_id);

    pool.initialize(
        &admin,
        &usdc_id,
        &treasury,
        &backstop,
        &Symbol::new(&env, "crop"),
        &policy_engine,
        &claims_processor,
    );

    // Sequential calls should succeed normally (guard is properly released after each call)
    let shares1 = pool.deposit(&lp, &100_000_000, &0, &false);
    assert!(shares1 > 0);

    let shares2 = pool.deposit(&lp, &100_000_000, &0, &false);
    assert!(shares2 > 0);
}
