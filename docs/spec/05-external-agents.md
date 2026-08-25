# 05 — External Agents (Asset Administration Permissions)

Sources: `pallets/external-agents/src/lib.rs`, `primitives/src/agent.rs`.
Related specs: [02-permissions](02-permissions.md) (layer-4 of the permission pipeline),
[04-asset-lifecycle](04-asset-lifecycle.md), [01-identity-keys](01-identity-keys.md) §5
(authorizations).

## 1. Purpose

Asset administration (mint, burn, freeze, compliance config, CAs, ...) is performed by
**external agents** of the asset. Each agent belongs to exactly one **agent group** per asset,
which resolves to a set of permitted pallets/extrinsics. The asset owner is just the initial
`Full` agent (doc 04 §3) — ownership itself confers no extra dispatch rights beyond the agent
system (ownership matters for ticker linking and receiving the owner role on transfer).

## 2. Data model

| Item | Shape | Ref |
|---|---|---|
| `GroupOfAgent` | (AssetId, DID) → `AgentGroup` | pallets/external-agents/src/lib.rs:120 |
| `AgentOf` | (DID, AssetId) → () reverse index | :108 |
| `GroupPermissions` | (AssetId, AGId) → `ExtrinsicPermissions` (custom groups) | :129 |
| `AGIdSequence` | AssetId → AGId (starts at 1) | :101 |
| `NumFullAgents` | AssetId → u32 | :125 |
| `AgentGroup` | `Full \| Custom(AGId) \| ExceptMeta \| PolymeshV1CAA \| PolymeshV1PIA` | primitives/src/agent.rs:15-31 |

### Group → permission resolution (`agent_permissions`, :669-702)

| Group | Resolves to |
|---|---|
| not an agent | `ExtrinsicPermissions::empty()` (:674) |
| `Full` | everything (:675) |
| `Custom(id)` | `GroupPermissions[asset, id]` or empty (:676) |
| `ExceptMeta` | everything **except the `ExternalAgents` pallet** (can't manage agents) (:679-681) |
| `PolymeshV1CAA` | only `CorporateAction`, `CorporateBallot`, `CapitalDistribution` pallets (:683-687) |
| `PolymeshV1PIA` | `Sto` except `invest` + `Asset::{issue, redeem, controller_transfer}` (:688-700) |

Permissions are `ExtrinsicPermissions` — same type as secondary-key extrinsic perms (doc 02 §2);
`Except` variant rejected for custom groups (`ExceptPermissionsNotAllowed`, :437-439).

## 3. Enforcement entry points (consumed by asset-scope pallets)

| Fn | Semantics | Ref |
|---|---|---|
| `ensure_asset_perms(origin, asset)` | identity pipeline + secondary-key **asset subset** check (`SecondaryKeyNotAuthorizedForAsset`). No agent check. | :637-655 |
| `ensure_agent_permissioned(asset, did)` | group permissions `sufficient_for(CurrentPalletName, CurrentDispatchableName)` else `UnauthorizedAgent`. Applies to **primary keys too**. | :657-667 |
| `ensure_agent_asset_perms(origin, asset)` | both of the above | :627-635 |
| `ensure_perms(origin, asset)` | `ensure_agent_asset_perms` → DID | :619-625 |

The group check reads the call metadata recorded by `StoreCallMetadata` (doc 02 §3), so nested
dispatches must swap metadata for agent checks to see the inner call.

Callers: asset (~29 sites), compliance-manager, statistics, checkpoint, corporate-actions,
ballot, distribution, sto, nft, settlement (venue-filter admin).

## 4. Agent management extrinsics

| Extrinsic (call_index) | Who may call | Behavior | Ref |
|---|---|---|---|
| `create_group(0)` | agent | new custom group; AGId from sequence; perms length-limited (:443), no `Except` | :225 → :406 |
| `set_group_permissions(1)` | agent | overwrite custom group perms; `NoSuchAG` if id invalid (:549-555) | :250 → :467 |
| `remove_agent(2)` | agent | remove target agent; last-Full guard | :275 → :490 |
| `abdicate(3)` | the agent itself (only `ensure_asset_perms` — **no group check**, :502) | remove self; last-Full guard | :296 → :501 |
| `change_group(4)` | agent | move target to another group; custom group must exist (:541-546) | :321 → :518 |
| `accept_become_agent(5)` | auth target | consume `BecomeAgent(asset, group)`; **issuer must be a permissioned agent at acceptance time** (:394); group must exist (:395); `AlreadyAnAgent` guard (:396) | :348 → :389 |
| `create_group_and_add_auth(6)` | agent | create group + issue `BecomeAgent` auth (optional expiry) | :359 → :450 |
| `create_and_change_custom_group(7)` | agent | create group + move existing agent into it atomically | :376 → :508 |

No protocol fees anywhere in this pallet; no cap on the number of agents (config has only
`WeightInfo`, :93-96).

## 5. Full-agent liveness protection

`try_mutate_agents_group` (:557-585) adjusts `NumFullAgents` on promote/demote;
`dec_full_count` (:604-612) uses `checked_sub(1).filter(|&x| x > 0)` → error
`RemovingLastFullAgent`: **an asset can never reach zero Full agents** via remove_agent /
abdicate / change_group. Ownership transfer swaps Full agents (add new then remove old — asset
lib.rs:2107-2108) preserving the invariant.

## 6. Becoming an agent

`BecomeAgent(asset_id, group)` authorizations are created via the generic
`Identity::add_authorization` or `create_group_and_add_auth`. Issuer competence is checked at
**acceptance**, not issuance (:394) — a stale auth from a since-removed agent is unusable.
Acceptance requires the target to pass the identity pipeline (:390).
Initial agent: `unchecked_add_agent` (:587-601) — called by asset creation (asset lib.rs:4097),
ownership transfer, and benchmarking/tests.

## 7. Invariants & review checklist

- [ ] Every asset-admin extrinsic in any pallet must call `ensure_agent_asset_perms` (or
      `ensure_perms`), not just `ensure_asset_perms` — the latter skips the group check.
- [ ] `NumFullAgents ≥ 1` for every asset with agents; all group mutations must go through
      `try_mutate_agents_group`.
- [ ] Custom group ids validated against `AGIdSequence` (`ensure_agent_group_valid`, :541-546)
      wherever accepted as input.
- [ ] `ExceptMeta` must keep excluding the `ExternalAgents` pallet, else privilege escalation
      (agent adds/removes agents).
- [ ] Wrappers dispatching inner calls must swap call metadata or agent checks evaluate the
      wrong extrinsic name (doc 02 §3).
- [ ] `GroupPermissions` are per-asset: check new code doesn't read another asset's group id.

## 8. Test map

`pallets/runtime/tests/src/external_agents_test.rs` (group create/set-perms, remove/abdicate/
change, multi-group perms :382, Except rejection :427-444), `asset_test.rs` (agent setup),
`corporate_actions_test.rs` (CAA group usage).
