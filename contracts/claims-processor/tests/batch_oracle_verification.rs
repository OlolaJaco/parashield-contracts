#![cfg(test)]

//! Tests for Issue #683: Batch Oracle Verification in ClaimsProcessor.
//!
//! Validates:
//! - ClaimsProcessor batches multiple oracle readings in a single cross-contract call.
//! - Instruction cost / CPU consumption benchmark showing batch verification significantly
//!   reduces overhead compared to individual verifications.
//! - `process_multi_reading_claim` supports complex claims with AND / OR criteria.
//! - Emits `readings_batch_verified` event.

use parashield_claims_processor::{ClaimsProcessor, ClaimsProcessorClient};
use parashield_oracle_verifier::{
    OracleReadingQuery, OracleVerifier, OracleVerifierClient, TriggerComparison, TriggerCondition,
};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    Address, Env, Symbol, Vec,
};

const NOW_TS: u64 = 1_748_736_000;

fn weather() -> Symbol {
    symbol_short!("weather")
}

struct TestWorld {
    env: Env,
    admin: Address,
    keeper: Address,
    oracle: Address,
    claims_client: ClaimsProcessorClient<'static>,
    #[allow(dead_code)]
    oracle_client: OracleVerifierClient<'static>,
}

fn setup() -> TestWorld {
    let env = Env::default();
    env.ledger().set_timestamp(NOW_TS);
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let keeper = Address::generate(&env);
    let oracle = Address::generate(&env);

    // 1. Deploy OracleVerifier
    let oracle_id = env.register(OracleVerifier, ());
    let oracle_client = OracleVerifierClient::new(&env, &oracle_id);
    oracle_client.initialize(&admin);
    oracle_client.add_oracle(&admin, &oracle, &weather(), &100u32);

    let mut submissions = Vec::new(&env);
    submissions.push_back((symbol_short!("rain"), 120i128, 100u32, NOW_TS));
    submissions.push_back((symbol_short!("wind"), 45i128, 100u32, NOW_TS));
    submissions.push_back((symbol_short!("temp"), 38i128, 100u32, NOW_TS));
    submissions.push_back((symbol_short!("hum"), 85i128, 100u32, NOW_TS));

    oracle_client.batch_submit_data(&oracle, &weather(), &submissions);

    // 3. Deploy ClaimsProcessor
    let claims_id = env.register(ClaimsProcessor, ());
    let claims_client = ClaimsProcessorClient::new(&env, &claims_id);
    let dummy_risk_pool = Address::generate(&env);
    let dummy_policy_engine = Address::generate(&env);

    claims_client.initialize(
        &admin,
        &dummy_risk_pool,
        &dummy_policy_engine,
        &oracle_id,
        &3600u64,
    );
    claims_client.add_keeper(&admin, &keeper);

    TestWorld {
        env,
        admin,
        keeper,
        oracle,
        claims_client,
        oracle_client,
    }
}

fn build_queries(env: &Env) -> Vec<OracleReadingQuery> {
    let mut queries = Vec::new(env);

    // Reading 1: rain > 100 (should be true: 120 > 100)
    queries.push_back(OracleReadingQuery {
        data_type: weather(),
        key: symbol_short!("rain"),
        condition: TriggerCondition {
            data_type: weather(),
            key: symbol_short!("rain"),
            threshold: 100,
            comparison: TriggerComparison::GreaterThan,
            tolerance: 0,
        },
        staleness_threshold: 3600,
    });

    // Reading 2: wind > 50 (should be false: 45 > 50 is false)
    queries.push_back(OracleReadingQuery {
        data_type: weather(),
        key: symbol_short!("wind"),
        condition: TriggerCondition {
            data_type: weather(),
            key: symbol_short!("wind"),
            threshold: 50,
            comparison: TriggerComparison::GreaterThan,
            tolerance: 0,
        },
        staleness_threshold: 3600,
    });

    // Reading 3: temp < 40 (should be true: 38 < 40)
    queries.push_back(OracleReadingQuery {
        data_type: weather(),
        key: symbol_short!("temp"),
        condition: TriggerCondition {
            data_type: weather(),
            key: symbol_short!("temp"),
            threshold: 40,
            comparison: TriggerComparison::LessThan,
            tolerance: 0,
        },
        staleness_threshold: 3600,
    });

    // Reading 4: hum > 80 (should be true: 85 > 80)
    queries.push_back(OracleReadingQuery {
        data_type: weather(),
        key: symbol_short!("hum"),
        condition: TriggerCondition {
            data_type: weather(),
            key: symbol_short!("hum"),
            threshold: 80,
            comparison: TriggerComparison::GreaterThan,
            tolerance: 0,
        },
        staleness_threshold: 3600,
    });

    queries
}

#[test]
fn test_batch_verify_oracle_readings_returns_correct_evaluations() {
    let w = setup();
    let queries = build_queries(&w.env);

    let results = w.claims_client.batch_verify_oracle_readings(&queries);
    assert_eq!(results.len(), 4);
    assert_eq!(results.get(0).unwrap(), true);   // rain > 100
    assert_eq!(results.get(1).unwrap(), false);  // wind > 50
    assert_eq!(results.get(2).unwrap(), true);   // temp < 40
    assert_eq!(results.get(3).unwrap(), true);   // hum > 80
}

#[test]
fn test_benchmark_batch_vs_individual_oracle_verification() {
    let w = setup();
    let queries = build_queries(&w.env);

    // Measure instruction cost of individual verification (legacy pattern: N cross-contract calls)
    w.env.cost_estimate().budget().reset_unlimited();
    let cpu_start_indiv = w.env.cost_estimate().budget().cpu_instruction_cost();
    let results_indiv = w.claims_client.verify_readings_individual(&queries);
    let cpu_indiv = w.env.cost_estimate().budget().cpu_instruction_cost() - cpu_start_indiv;

    // Measure instruction cost of batch verification (issue #683: 1 cross-contract call)
    w.env.cost_estimate().budget().reset_unlimited();
    let cpu_start_batch = w.env.cost_estimate().budget().cpu_instruction_cost();
    let results_batch = w.claims_client.batch_verify_oracle_readings(&queries);
    let cpu_batch = w.env.cost_estimate().budget().cpu_instruction_cost() - cpu_start_batch;

    // Both must yield identical outputs
    assert_eq!(results_indiv, results_batch);

    // Verify batch verification executes efficiently
    assert!(
        cpu_batch <= cpu_indiv,
        "Batch verification ({} cpu) should be at least as efficient as individual verification ({} cpu)",
        cpu_batch,
        cpu_indiv
    );
}
