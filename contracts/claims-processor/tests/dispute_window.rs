use parashield_claims_processor::{
    ClaimsProcessor, ClaimsProcessorClient, DEFAULT_DISPUTE_WINDOW, MIN_DISPUTE_WINDOW,
};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn dispute_window_is_nonzero_and_bounded() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract = env.register(ClaimsProcessor, ());
    let client = ClaimsProcessorClient::new(&env, &contract);
    client.initialize(
        &admin,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &60u64,
    );

    assert_eq!(client.get_dispute_window(), DEFAULT_DISPUTE_WINDOW);
    assert!(client.try_set_dispute_window(&admin, &0u64).is_err());
    client.set_dispute_window(&admin, &MIN_DISPUTE_WINDOW);
    assert_eq!(client.get_dispute_window(), MIN_DISPUTE_WINDOW);
}
