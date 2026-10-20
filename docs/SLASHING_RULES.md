# Oracle Staking, Challenge Period & Slashing Rules

## Overview

In decentralized parametric insurance, oracles supply ground-truth external measurements (e.g., rainfall, temperatures, flight status) that trigger automated claim payouts. To deter malicious or compromised oracles from submitting false triggers and instantly fleeing with their deposited collateral, the protocol enforces an economic bonding mechanism with a **mandatory challenge period** and **slashing rules**.

---

## 1. Oracle Stake Lifecycle

### 1.1 Deposit (`stake`)
- Oracles deposit collateral in the protocol's designated stake token (USDC or protocol governance token).
- An oracle's deposited stake must meet or exceed `min_stake` before it is eligible to submit data points for trigger evaluation.
- Staking requires the oracle's explicit signature (`oracle.require_auth()`).

### 1.2 Deactivation & Withdrawal Request (`request_stake_withdrawal`)
- An oracle cannot withdraw its stake while still active in the verifier pool.
- The oracle must first be removed or deactivated (`active = false`).
- To withdraw stake, the oracle submits a `request_stake_withdrawal`.
- This initiates a non-bypassable **challenge period** (default: 7 days / 604,800 seconds, configurable by governance/admin).
- A `StakeWithdrawalRequest` is recorded on-chain containing:
  - `oracle`: Address of the oracle.
  - `data_type`: Specific data feed category.
  - `amount`: Locked stake amount requested.
  - `requested_at`: Timestamp of request.
  - `unlock_at`: `requested_at + challenge_period`.

### 1.3 Cancellation (`cancel_stake_withdrawal`)
- The oracle may cancel an outstanding withdrawal request before completion, retaining their stake locked in the protocol.

### 1.4 Execution (`withdraw_stake`)
- Once `ledger().timestamp() >= unlock_at`, the oracle calls `withdraw_stake`.
- The contract verifies the challenge period has elapsed.
- If no slashing occurred, the full requested amount is transferred to the oracle.
- If slashing occurred during the challenge period, only the remaining unsashed stake is returned.

---

## 2. Slashing Protection

### 2.1 Trigger Events for Slashing
Oracles are subject to stake slashing under the following conditions:
1. **Provably Malicious / False Data**: Off-chain or cross-referenced proof that an oracle submitted fraudulent readings to falsely trigger or prevent claim payouts.
2. **Repeated Stale Submissions**: If configured with stale penalties, oracles submitting stale data exceeding `stale_threshold` receive penalties.
3. **Collusion or Consensus Manipulation**: Submitting outlier values attempting to skew median calculation.

### 2.2 Slashing Execution (`slash_oracle`)
- Admin or DAO governance executes `slash_oracle(admin, oracle, data_type, amount, reason)`.
- The slashed amount is deducted from the oracle's current stake.
- **Challenge Period Slashing**: If a `StakeWithdrawalRequest` is pending in its challenge period, the pending request's `amount` is automatically decremented by the slashed amount.
- Slashed tokens can be transferred to a designated treasury address (`SlashRecipient`) or burned.
- If 100% of the stake is slashed, the pending withdrawal is voided and subsequent `withdraw_stake` calls fail with `NoStake`.

---

## 3. Security Invariants

1. **No Instant Exit**: An oracle cannot submit data and withdraw stake in the same block or within the challenge window.
2. **Skin in the Game**: Any data submitted by an oracle is challengeable during the challenge period.
3. **Idempotence & State Cleanup**: Completed or fully slashed withdrawal requests are immediately purged from persistent storage.
