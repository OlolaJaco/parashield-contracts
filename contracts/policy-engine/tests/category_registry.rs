use parashield_policy_engine::{
    CreateProductParams, PolicyEngine, PolicyEngineClient, TriggerComparison, TriggerType,
};
use soroban_sdk::{
    symbol_short,
    testutils::Address as _,
    Address, Env, Symbol,
};

fn setup() -> (Env, Address, Address, PolicyEngineClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let usdc = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let id = env.register(PolicyEngine, ());
    let client = PolicyEngineClient::new(&env, &id);
    client.initialize(&admin, &usdc, &Address::generate(&env));
    (env, admin, usdc, client)
}

fn params(name: Symbol, category: Symbol, oracle_key: Symbol, expected_token: Address) -> CreateProductParams {
    CreateProductParams {
        name,
        category,
        oracle_key,
        expected_token,
        trigger_type: TriggerType::Threshold,
        oracle_data_type: symbol_short!("weather"),
        trigger_threshold: 50_000_000,
        trigger_comparison: TriggerComparison::LessThan,
        coverage_min: 100_000_000,
        coverage_max: 10_000_000_000,
        premium_rate_bps: 500,
        max_duration_days: 365,
    }
}

#[test]
fn default_registered_categories_accepted() {
    let (_, admin, usdc, client) = setup();
    let pid = client.create_product(
        &admin,
        &params(symbol_short!("p_crop"), symbol_short!("crop"), symbol_short!("kis2606"), usdc),
    );
    assert!(pid > 0);
}

#[test]
fn unregistered_category_is_rejected() {
    let (_, admin, usdc, client) = setup();
    let res = client.try_create_product(
        &admin,
        &params(symbol_short!("p_bad"), symbol_short!("unreg"), symbol_short!("kis2606"), usdc),
    );
    assert!(res.is_err(), "Products with unregistered categories must be rejected");
}

#[test]
fn dynamic_category_registration_flow() {
    let (_, admin, usdc, client) = setup();
    let new_cat = symbol_short!("carbon");

    // Rejected before registration
    assert!(!client.is_category_registered(&new_cat));
    let err = client.try_create_product(
        &admin,
        &params(symbol_short!("p_carb"), new_cat.clone(), symbol_short!("kis2606"), usdc.clone()),
    );
    assert!(err.is_err());

    // Admin registers category
    client.register_category(&admin, &new_cat);
    assert!(client.is_category_registered(&new_cat));

    // Now creation succeeds
    let pid = client.create_product(
        &admin,
        &params(symbol_short!("p_carb"), new_cat.clone(), symbol_short!("kis2606"), usdc.clone()),
    );
    assert!(pid > 0);

    // Admin unregisters category
    client.unregister_category(&admin, &new_cat);
    assert!(!client.is_category_registered(&new_cat));

    // Further creation is rejected
    let err2 = client.try_create_product(
        &admin,
        &params(symbol_short!("p_carb2"), new_cat, symbol_short!("kis2607"), usdc),
    );
    assert!(err2.is_err());
}

#[test]
fn category_registry_contains_defaults() {
    let (_, _, _, client) = setup();
    let registered = client.get_registered_categories();
    assert!(registered.contains(&symbol_short!("crop")));
    assert!(registered.contains(&symbol_short!("flight")));
    assert!(registered.contains(&symbol_short!("disaster")));
    assert!(registered.contains(&symbol_short!("health")));
    assert!(registered.contains(&symbol_short!("defi")));
}
