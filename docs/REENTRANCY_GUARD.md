# Reentrancy Protection and Checks-Effects-Interactions Security Pattern

## Overview
Soroban smart contracts interact with external token contracts (such as USDC) and cross-contract interfaces. While Soroban's execution model is transactional, external calls (e.g., token transfers, callbacks, or reinsurer interfaces) create potential reentrancy attack surfaces if state is mutated after external invocations or if re-entrant entry points are unprotected.

## Security Pattern Implementation (#684)

### 1. RAII Reentrancy Guard
The `RiskPool` contract implements an RAII `ReentrancyGuard`:
- On function entry, `ReentrancyGuard::enter(&env)` verifies that `StorageKey::ReentrancyGuard` is not set.
- If already set, execution immediately panics with `Error::Reentrancy` (contract error code 47).
- Otherwise, `StorageKey::ReentrancyGuard` is set to `true`.
- When the function execution finishes and the guard goes out of scope, `Drop::drop` automatically cleans up the storage key.

### 2. Guarded Entry Points
The guard is active on all state-changing entry points that execute external calls or manage pool assets:
- `deposit`
- `withdraw`
- `claim_exit`
- `process_expired_exits`
- `emergency_withdraw`
- `receive_premium`
- `send_premium_to_treasury`
- `send_premium_to_backstop`
- `claim_yield`
- `execute_admin_withdrawal`
- `send_premium_to_reinsurer`
- `request_reinsurance_recovery`
- `request_exit`
- `cancel_exit`

### 3. Checks-Effects-Interactions (CEI) Compliance
All operations follow strict Checks-Effects-Interactions:
1. **Checks**: Verify caller authorization, pool active status, valid bounds, and capacity limits.
2. **Effects**: Update internal balances, total shares, LP positions, queues, and timelocks in persistent storage.
3. **Interactions**: Perform external token transfers and cross-contract calls only after all local storage state changes are committed.

In `receive_premium`, `execute_admin_withdrawal`, and `send_premium_to_reinsurer`, pool state variables (`AccumulatedPremium`, `TotalDeposited`, `ReinsurancePremiumPaid`) are updated before outgoing transfers.
