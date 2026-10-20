use parashield_policy_engine::{PolicyEngine, PolicyEngineClient, ORACLE_CHANGE_TIMELOCK_SECONDS};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

#[test]
fn oracle_rotation_needs_guardian_and_timelock() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let guardian = Address::generate(&env);
    let usdc = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let old_oracle = env.register(PolicyEngine, ());
    let new_oracle = env.register(PolicyEngine, ());
    let contract = env.register(PolicyEngine, ());
    let client = PolicyEngineClient::new(&env, &contract);
    client.initialize(&admin, &usdc, &old_oracle);

    client.set_oracle_guardian(&admin, &guardian);
    client.propose_oracle_change(&admin, &new_oracle);
    client.approve_oracle_change(&guardian);
    assert!(client.try_execute_oracle_change(&admin).is_err());

    env.ledger().with_mut(|ledger| {
        ledger.timestamp += ORACLE_CHANGE_TIMELOCK_SECONDS;
    });
    client.execute_oracle_change(&admin);
    assert_eq!(client.get_oracle(), new_oracle);
}

#[test]
fn oracle_rotation_without_guardian_approval_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let guardian = Address::generate(&env);
    let usdc = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let old_oracle = env.register(PolicyEngine, ());
    let new_oracle = env.register(PolicyEngine, ());
    let contract = env.register(PolicyEngine, ());
    let client = PolicyEngineClient::new(&env, &contract);
    client.initialize(&admin, &usdc, &old_oracle);

    client.set_oracle_guardian(&admin, &guardian);
    client.propose_oracle_change(&admin, &new_oracle);
    env.ledger().with_mut(|ledger| {
        ledger.timestamp += ORACLE_CHANGE_TIMELOCK_SECONDS;
    });
    assert!(client.try_execute_oracle_change(&admin).is_err());
    assert_eq!(client.get_oracle(), old_oracle);
}
