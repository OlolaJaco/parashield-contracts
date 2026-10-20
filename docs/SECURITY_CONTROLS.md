# Contract security controls

## Policy oracle changes

Oracle rotation is a three-step operation. The admin calls
`propose_oracle_change`, the independently configured oracle guardian calls
`approve_oracle_change`, and the admin calls `execute_oracle_change` after the
48-hour review period. A pending proposal cannot be replaced, an unapproved
proposal cannot execute, and changing the guardian does not bypass the delay.

## Claims processor guardian multisig

The admin bootstraps the initial guardian set with `set_guardians`. Each
non-empty set must have a threshold from 1 through the set size, and duplicate
guardian addresses are rejected. After bootstrap, `set_guardians` only proposes
a replacement; the currently configured threshold must approve it through
`approve_guardian_config` before it takes effect. An active guardian set cannot
be cleared or disabled by setting its threshold to zero. The initial
admin-controlled bootstrap is the only configuration change that does not
require approval from an existing set. `guardian_config_proposed` and
`guardian_config_approved` report the change and approval progress;
`guardians_updated` is emitted only when the change takes effect.

## Policy expiration event

When the authorized claims processor expires a policy, PolicyEngine emits the
`policy_expired` event with a `PolicyExpired` payload containing `policy_id`
(`u128`) and `timestamp` (`u64`, the current ledger timestamp).

## Claim dispute windows

The claims processor defaults to a seven-day dispute window. The admin may set
the window from one hour (`MIN_DISPUTE_WINDOW`) through 30 days
(`MAX_DISPUTE_WINDOW`), inclusive. Zero is invalid because it would remove a
claimant's opportunity to dispute a rejected or partially paid claim. Pending
claims remain disputable because no decision timestamp exists yet.

## LP NFT ID space

LP NFT IDs are sequential nonzero `u64` values. Minting the final representable
ID fails with `Error::Overflow`; the counter never wraps to zero or overwrites
an earlier NFT. In practice this makes exhaustion theoretical while keeping
the terminal behavior explicit and safe.

## Governance delegation

Delegation is deliberately one hop. Self-delegation, delegation to an account
that has delegated onward, and delegation by an account that already receives
delegations are rejected. A proposed edge that points back to its delegator is
identified specifically as `CircularDelegation`, so no cycle can enter the
stored delegation graph or be traversed while calculating voting weight.
