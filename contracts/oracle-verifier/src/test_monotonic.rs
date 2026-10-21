#![allow(clippy::inconsistent_digit_grouping)]
#![cfg(test)]

//! Regression coverage for issue #775: readings for the same
//! `(data_type, key, oracle)` must advance strictly forward in time.
//! A replayed or out-of-order submission from a flaky feed used to be
//! accepted and could overwrite a fresher reading already folded into
//! the aggregate.

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    Env,
};

fn setup() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.ledger().set_timestamp(1_750_000_000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register(OracleVerifier, ());
    let client = OracleVerifierClient::new(&env, &contract_id);
    client.initialize(&admin);
    client.add_oracle(&admin, &oracle, &symbol_short!("weather"), &100u32);

    (env, admin, oracle, contract_id)
}

#[test]
fn first_submission_is_accepted() {
    let (env, _admin, oracle, contract_id) = setup();
    let client = OracleVerifierClient::new(&env, &contract_id);
    // No prior reading exists for this (data_type, key, oracle), so the
    // monotonic guard is a no-op.
    client.submit_data(
        &oracle,
        &symbol_short!("weather"),
        &symbol_short!("kis001"),
        &10i128,
        &100u32,
        &1_749_900_000u64,
    );
}

#[test]
fn second_submission_with_a_strictly_later_timestamp_is_accepted() {
    let (env, _admin, oracle, contract_id) = setup();
    let client = OracleVerifierClient::new(&env, &contract_id);
    client.submit_data(
        &oracle,
        &symbol_short!("weather"),
        &symbol_short!("kis001"),
        &10i128,
        &100u32,
        &1_749_900_000u64,
    );
    // Advance the ledger enough to clear MinSubmitInterval.
    env.ledger().set_timestamp(1_750_000_060);
    client.submit_data(
        &oracle,
        &symbol_short!("weather"),
        &symbol_short!("kis001"),
        &20i128,
        &100u32,
        &1_749_900_030u64,
    );
}

/// Error 37 = `StaleReadingTimestamp`: a second submission with the same
/// timestamp as the first is rejected (strictly-newer rule).
#[test]
#[should_panic(expected = "Error(Contract, #37)")]
fn equal_timestamp_resubmission_is_rejected() {
    let (env, _admin, oracle, contract_id) = setup();
    let client = OracleVerifierClient::new(&env, &contract_id);
    client.submit_data(
        &oracle,
        &symbol_short!("weather"),
        &symbol_short!("kis001"),
        &10i128,
        &100u32,
        &1_749_900_000u64,
    );
    env.ledger().set_timestamp(1_750_000_060);
    client.submit_data(
        &oracle,
        &symbol_short!("weather"),
        &symbol_short!("kis001"),
        &20i128,
        &100u32,
        &1_749_900_000u64, // same timestamp
    );
}

/// Error 37 = `StaleReadingTimestamp`: an older timestamp overwrites a
/// fresher reading only when the fix is absent; with it, the stale one
/// is rejected.
#[test]
#[should_panic(expected = "Error(Contract, #37)")]
fn earlier_timestamp_resubmission_is_rejected() {
    let (env, _admin, oracle, contract_id) = setup();
    let client = OracleVerifierClient::new(&env, &contract_id);
    client.submit_data(
        &oracle,
        &symbol_short!("weather"),
        &symbol_short!("kis001"),
        &10i128,
        &100u32,
        &1_749_900_060u64,
    );
    env.ledger().set_timestamp(1_750_000_060);
    // Try to overwrite with an older reading.
    client.submit_data(
        &oracle,
        &symbol_short!("weather"),
        &symbol_short!("kis001"),
        &20i128,
        &100u32,
        &1_749_900_030u64,
    );
}

/// The monotonic guard is scoped per (data_type, key, oracle). A lagged
/// clock on one key does not block submissions on an unrelated key.
#[test]
fn monotonic_guard_is_scoped_per_key() {
    let (env, _admin, oracle, contract_id) = setup();
    let client = OracleVerifierClient::new(&env, &contract_id);
    client.submit_data(
        &oracle,
        &symbol_short!("weather"),
        &symbol_short!("kis001"),
        &10i128,
        &100u32,
        &1_749_900_060u64,
    );
    env.ledger().set_timestamp(1_750_000_060);
    // Different key — an older timestamp here is allowed.
    client.submit_data(
        &oracle,
        &symbol_short!("weather"),
        &symbol_short!("kis002"),
        &20i128,
        &100u32,
        &1_749_900_030u64,
    );
}
