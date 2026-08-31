# Polymesh Chain Logic Specification

Specification of the Polymesh blockchain runtime logic, written for engineers and AI agents
reviewing or modifying this codebase. Each document describes one subsystem: its data model,
extrinsics with their authorization requirements, core flows, cross-pallet interactions, and the
invariants a reviewer should check when the code changes.

## Conventions

- Code citations use `path:line` (`symbol_name`). Line numbers drift as code changes; the symbol
  name is authoritative — re-locate with `rg` if a line number is stale.
- "GC" = Governance Council (root or committee origins). "DID" = identity (`IdentityId`).
  "Primary key" / "secondary key" refer to the account keys attached to a DID.
- Extrinsic tables list the *effective* authorization: what `origin` must be and which
  permission checks are applied after origin resolution.
- POLYX is the native token (6 decimals; `ONE_POLY = 1_000_000`).

## Repository architecture

- **Node** (`src/`): standard Substrate node (BABE/GRANDPA consensus, BEEFY/MMR).
  Binary entry `src/bin/main.rs`; service wiring `src/service.rs`; chain specs `src/chain_spec/`.
- **Three runtimes** (`pallets/runtime/{develop,testnet,mainnet}`): share one identical
  `spec_version` (checked by `scripts/check_spec_and_cargo_version.sh`). Most configuration and
  the runtime macro scaffolding live in `pallets/runtime/common/src/runtime.rs` (macro
  `misc_pallet_impls!` / common types) with per-chain constants in each runtime's
  `constants.rs`/`runtime.rs`. Runtime changes usually must be wired in all three.
- **Forked polkadot-sdk**: all `sp-*`/`sc-*`/`frame-*`/`pallet-staking`/`pallet-revive` deps come
  from `PolymeshAssociation/polkadot-sdk` (branch pinned in root `Cargo.toml`
  `[workspace.dependencies]`). The fork mainly exists to support Polymesh's identity/permission
  system (e.g. fee-payer redirection hooks, revive origin handling).
- **Shared tests**: `pallets/runtime/tests/` (`polymesh-runtime-tests`, `ExtBuilder`-based mock
  runtime). `integration/` is a separate workspace driving a live chain over RPC.
- **Weights**: central in `pallets/weights/src/*.rs`, not inside pallets.

### Runtime differences

| Pallet | develop | testnet | mainnet |
|---|---|---|---|
| `Sudo` | yes | yes (`sudo` key) | **no** |
| `ConfidentialAssets` (index 70) | yes | yes | **no** |
| `Revive` (index 80) | yes | yes | yes |

Everything else is identical modulo constants (e.g. settlement lock periods, CA defaults).

### Pallet map (index → pallet, develop runtime `pallets/runtime/develop/src/runtime.rs`)

| # | Pallet | Source | Spec doc |
|---|---|---|---|
| 0–6 | System, Babe, Timestamp, Indices, Authorship, Balances, TransactionPayment | forked SDK + `pallets/runtime/common` | [14](14-fees-and-extensions.md) |
| 51 | PolymeshTransactionPayment | `pallets/transaction-payment` | [14](14-fees-and-extensions.md) |
| 7 | Identity | `pallets/identity` | [01](01-identity-keys.md), [02](02-permissions.md), [03](03-claims.md) |
| 8 | DidRegistrars (group Instance2) | `pallets/group` | [22](22-treasury-committees.md) |
| 9–14 | Polymesh/Technical/Upgrade committees + memberships | `pallets/committee`, `pallets/group` | [22](22-treasury-committees.md) |
| 15 | MultiSig | `pallets/multisig` | [16](16-multisig.md) |
| 16 | Validators | `pallets/validators` | [20](20-staking-validators.md) |
| 17 | Staking | forked SDK `pallet-staking` | [20](20-staking-validators.md) |
| 18–23 | Offences, Session, AuthorityDiscovery, Grandpa, Historical, ImOnline | forked SDK | [20](20-staking-validators.md) |
| 25 | Sudo (develop/testnet only) | forked SDK | — |
| 26 | Asset | `pallets/asset` | [04](04-asset-lifecycle.md), [09](09-asset-transfers.md) |
| 27 | CapitalDistribution | `pallets/corporate-actions/src/distribution` | [12](12-corporate-actions.md) |
| 28 | Checkpoint | `pallets/asset/src/checkpoint` | [11](11-checkpoints.md) |
| 29 | ComplianceManager | `pallets/compliance-manager` | [06](06-compliance.md) |
| 30 | CorporateAction | `pallets/corporate-actions` | [12](12-corporate-actions.md) |
| 31 | CorporateBallot | `pallets/corporate-actions/src/ballot` | [12](12-corporate-actions.md) |
| 32 | Permissions | `pallets/permissions` | [02](02-permissions.md) |
| 33 | Pips | `pallets/pips` | [19](19-pips.md) |
| 34 | Portfolio | `pallets/portfolio` | [08](08-portfolio.md) |
| 35 | ProtocolFee | `pallets/protocol-fee` | [14](14-fees-and-extensions.md) |
| 36 | Scheduler | forked SDK | — |
| 37 | Settlement | `pallets/settlement` | [10](10-settlement.md) |
| 38 | Statistics | `pallets/statistics` | [07](07-statistics.md) |
| 39 | Sto | `pallets/sto` | [13](13-sto.md) |
| 40 | Treasury | `pallets/treasury` | [22](22-treasury-committees.md) |
| 41 | Utility | `pallets/utility` | [17](17-utility.md) |
| 42 | Base | `pallets/base` | — (length-limit helpers) |
| 43 | ExternalAgents | `pallets/external-agents` | [05](05-external-agents.md) |
| 44 | Relayer | `pallets/relayer` | [15](15-relayer.md) |
| 48 | Preimage | forked SDK | — |
| 49 | Nft | `pallets/nft` | [04](04-asset-lifecycle.md), [09](09-asset-transfers.md) |
| 50 | ElectionProviderMultiPhase | forked SDK | [20](20-staking-validators.md) |
| 52–54 | Beefy, Mmr, MmrLeaf | forked SDK | — |
| 55 | MultiBlockMigrations | forked SDK | — |
| 70 | ConfidentialAssets (develop/testnet) | `pallets/confidential-assets` | [18](18-confidential-assets.md) |
| 80 | Revive | forked SDK `pallet-revive` + `precompiles/` | [21](21-revive-evm.md) |

## Cross-cutting design (read first)

1. **Identity-first**: almost every extrinsic resolves the caller's account key to a DID before
   doing anything. Accounts are cheap; identities carry claims, portfolios, asset roles.
   One account key belongs to at most one DID (or one multisig).
2. **Layered permissions**: a call passes up to four gates —
   (a) key→DID resolution + DID-not-frozen,
   (b) secondary-key *extrinsic* permission (pallet/function subsets, recorded per-call by the
   `StoreCallMetadata` transaction extension),
   (c) secondary-key *asset* / *portfolio* subsets checked by the target pallet,
   (d) asset-scoped *agent group* permission (external-agents) for asset admin calls.
   Primary keys skip (b)/(c) but not (d).
3. **Transfers are settlement-centric**: every asset movement (including the direct
   `Asset::transfer_asset` UX and ERC-20-style allowance spends) funnels into the settlement
   engine's instruction machinery, which enforces custody, affirmations, compliance, statistics,
   and venue filtering. Same-DID portfolio moves skip compliance/statistics.
4. **Authorizations**: privileged relationship changes (join identity, rotate primary key, become
   agent, transfer ticker/portfolio custody...) are two-phase: issuer creates an `Authorization`,
   target accepts it. The *issuer* pays the acceptance fees (see doc 14).

## Document index (recommended reading order)

| Doc | Subsystem | Status |
|---|---|---|
| [01-identity-keys.md](01-identity-keys.md) | DIDs, primary/secondary keys, authorizations | done |
| [02-permissions.md](02-permissions.md) | Permission data model + enforcement pipeline | done |
| [03-claims.md](03-claims.md) | Identity claims, issuers, CDD status | done |
| [04-asset-lifecycle.md](04-asset-lifecycle.md) | Fungible + NFT asset lifecycle | done |
| [05-external-agents.md](05-external-agents.md) | Asset agents & agent groups | done |
| [06-compliance.md](06-compliance.md) | Compliance requirements & evaluation | done |
| [07-statistics.md](07-statistics.md) | Transfer restrictions (statistics) | done |
| [08-portfolio.md](08-portfolio.md) | Portfolios & custodianship | done |
| [09-asset-transfers.md](09-asset-transfers.md) | All transfer code paths | done |
| [10-settlement.md](10-settlement.md) | Venues, instructions, locking | done |
| [11-checkpoints.md](11-checkpoints.md) | Balance snapshots & schedules | done |
| [12-corporate-actions.md](12-corporate-actions.md) | CAs, ballots, capital distributions | done |
| [13-sto.md](13-sto.md) | STO fundraising | done |
| [14-fees-and-extensions.md](14-fees-and-extensions.md) | TxExtension, fee payment, protocol fees | done |
| [15-relayer.md](15-relayer.md) | Fee subsidies | done |
| [16-multisig.md](16-multisig.md) | Multisig accounts & proposals | done |
| [17-utility.md](17-utility.md) | Batching & call wrappers | done |
| [18-confidential-assets.md](18-confidential-assets.md) | DART confidential assets | done |
| [19-pips.md](19-pips.md) | On-chain governance (PIPs) | done |
| [20-staking-validators.md](20-staking-validators.md) | Staking + permissioned validators | done |
| [21-revive-evm.md](21-revive-evm.md) | EVM/ETH support & precompiles | done |
| [22-treasury-committees.md](22-treasury-committees.md) | Treasury, committees, group instances | done |
