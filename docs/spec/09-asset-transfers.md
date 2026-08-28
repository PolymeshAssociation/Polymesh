# 09 — Asset Transfer Code Paths

Sources: `pallets/asset/src/lib.rs`, `pallets/settlement/src/lib.rs`, `pallets/nft/src/lib.rs`,
`primitives/src/asset.rs` (AssetHolder).
Related specs: [10-settlement](10-settlement.md) (instruction machinery),
[08-portfolio](08-portfolio.md), [06-compliance](06-compliance.md), [07-statistics](07-statistics.md).

## 1. Master map — every way tokens move

| Path | Entry | Compliance | Statistics | Notes |
|---|---|---|---|---|
| Settlement instruction leg execution | `transfer_assets` settlement:2215 → `Asset::base_transfer` asset:2745 / `Nft::base_nft_transfer` nft:601 | yes | yes (fungible) | the canonical cross-identity path |
| `Settlement::transfer_funds` same-DID branch | settlement:1596-1638 | **no** | **no** | direct holder-to-holder move within one identity |
| `Settlement::transfer_funds` cross-DID branch | settlement:3850-3962 | yes (via instruction) | yes | auto-created 1-leg instruction |
| `Asset::transfer_asset` / `Nft::transfer_nft` | asset:1708→2919 / nft:309→768 | (wraps transfer_funds) | — | account-holding UX wrappers |
| `Portfolio::move_portfolio_funds` | portfolio:568 | **no** | **no** | same-identity portfolio moves (doc 08 §5) |
| Controller transfer (fungible/NFT) | asset:2375 / nft:799 | **no** | updates only | agent-forced (doc 04 §5) |
| Locked-instruction execution | `simplified_asset_transfer` settlement:3501 | **no (checked at lock)** | **yes, re-verified** | doc 10 §6 |
| Issue / redeem | asset:4113 / asset:2226 | no | updates only | mint/burn, not transfers |

Same-identity moves are cheap by design: identity-level `BalanceOf` doesn't change, so
compliance, statistics, and checkpoints are all skipped soundly. Settlement instructions
**reject same-DID legs** at creation (`SameSenderReceiver`, settlement:2901/2915/2983), so the
fast paths above are the *only* same-identity routes.

## 2. Holders: portfolios vs accounts

Both leg endpoints are `AssetHolder = Portfolio(PortfolioId) | Account(AccountId32)`
(primitives/src/asset.rs:194-199).

- **Account-held** balances live in asset-pallet storage: `AssetBalance`, `LockedBalance`,
  `FrozenBalance`, `FrozenAccounts` (asset:602-659). The account must be linked to a DID
  (`asset_holder_did`, identity keys.rs:790-804 — else `IdentityNotFoundForAccountPortfolio`);
  identity-level `BalanceOf` still accrues to that DID. 0↔nonzero transitions maintain
  `AccountKeyRefCount` (asset:3673-3695) so holding keys can't be unlinked (doc 01 §2).
- Acting *for* an account holding: caller must be that exact key, or the DID's primary key
  (`ensure_account_permissions`, asset:3911-3928). Portfolio holdings: custody + portfolio
  permission (`ensure_holder_permissions`, asset:3889-3906).

## 3. The canonical validated transfer (`base_transfer`, asset:2745)

Called only from settlement leg execution (custody was already checked at affirmation —
comment asset:2755-2758). `validate_asset_transfer` (asset:3362-3431) checks **in order**:

1. asset exists & fungible (:3370-3374)
2. sender DID ≠ receiver DID (`SenderSameAsReceiver` :3379)
3. sender identity `BalanceOf` ≥ value (:3381); receiver overflow (:3385)
4. holdings valid: receiver portfolio exists / receiver account has DID (:3393 → 3815-3834);
   sender available balance = balance − locked − frozen ≥ value, sender holder not frozen,
   granularity (:3838-3867)
5. `is_controller_transfer` ⇒ **return early** (skip 6-9) (:3401-3404)
6. asset not frozen (:3406)
7. receiver DID active (:3408)
8. `Statistics::verify_transfer_restrictions` (:3414, doc 07)
9. `ComplianceManager::is_compliant` (:3426, doc 06)

Effects (`unverified_transfer_asset`, asset:4174-4244) in order: checkpoint pre-update for both
DIDs (:4193-4199) → `BalanceOf` ± (:4202) → (controller only: reduce sender frozen balance
:4207-4218) → holder balances via `set_holders_balance` incl. refcounts (:4221 → 3736-3750) →
statistics update (:4224) → `AssetBalanceUpdated` event (:4234).

## 4. Direct transfer UX (`transfer_funds` and wrappers)

`Settlement::transfer_funds(origin, from: Option<AssetHolder>, to: AssetHolder, fund)`
(settlement:1519 → `base_transfer_funds` 1561-1654). `from = None` defaults to the **caller's
account holding** (:1574-1578). `Asset::transfer_asset(asset_id, to: AccountId, amount, memo)`
(asset:1708) and `Nft::transfer_nft` (nft:309) wrap it with `to = AssetHolder::Account(to)` —
these wrappers never touch portfolios.

**Sender authorization** (`ensure_transfer_source_authorized`, settlement:1660-1695):
- Account source, caller == owner: implicit.
- Account source, caller ≠ owner: **spender mode**. Fungible funds consume the allowance via
  `Asset::spend_allowance(owner, caller, asset, amount)` (settlement:1673, asset:2782). NFT funds
  consume an approval via `Nft::spend_nft_approval(owner, caller, nfts)` — a collection-wide
  operator approval (`OperatorApproval`) authorizes every NFT and is not consumed, otherwise each
  NFT needs a per-token approval (`TokenApproval`) naming the caller, which is consumed on use
  (`InsufficientNFTApproval` otherwise). Both approvals are settable from the EVM through the
  ERC-721 precompile (doc 21 §5.1).
- Portfolio source: custody + portfolio permission (:1686-1692) — works cross-DID for custodians.

**Same-DID branch** (settlement:1596-1638): amount > 0, asset not frozen, holder not frozen,
available balance / NFT ownership; direct holder-balance move; `FundsTransferred` event.
No instruction, no compliance/statistics/checkpoints.

**Cross-DID branch** (`base_transfer_and_try_execute`, settlement:3850-3962): builds a
**one-leg instruction: venue = None, `SettleManual(current_block)`** (:3896-3905, executable
immediately, never scheduled); auto-affirms the sender side (locks tokens, :3908-3913); if the
caller also controls the receiver holding, affirms that too (:3916-3946); if no affirmations
remain pending (receiver pre-approved / default-skip) it **executes inline** (:3948-3955)
returning no id, else returns `Some(instruction_id)` for the receiver to act on. Full
compliance/statistics run at execution via `base_transfer`.

## 5. Receiver affirmation model (default: not required)

Whether the receiver leg auto-affirms is decided at instruction creation
(`skip_asset_holder_affirmation`, asset:3938-3958):

1. Governing DID = receiving portfolio's custodian-else-owner (portfolio:1181-1182), or the
   account's DID.
2. If that DID has **not** opted in via `Settlement::set_mandatory_receiver_affirmation`
   (settlement:1477-1492, storage `MandatoryReceiverAffirmation` settlement:683-687) ⇒ **skip
   (auto-affirm)** — the chain default.
3. If opted in, affirmation is still skipped when: asset globally exempt
   (`AssetsExemptFromAffirmation`, root-set, asset:547) OR receiver DID pre-approved the asset
   (`PreApprovedAsset`, asset:552, set via `pre_approve_asset` asset:1510) OR the specific
   portfolio is pre-approved (`PreApprovedPortfolios`, portfolio:316, custodian-set).

Receiver actions on a pending direct transfer: `Asset::receiver_affirm_asset_transfer`
(asset:1754 → settlement:3965-4012; account holdings only — portfolio receivers use the normal
`Settlement::affirm_instruction`) executes immediately after affirming;
`Asset::reject_asset_transfer` (asset:1786 → settlement:4058) — **either party** may reject a
Pending/Failed instruction.

## 6. Spender approvals (ERC-20 style allowances)

- `Asset::approve(asset_id, spender: AccountId, amount)` (asset:1812): amount 0 removes the
  entry; `Balance::MAX` = infinite (never decremented, asset:2792-2793). Storage `Allowances`
  NMap (owner, spender, asset) → Balance (asset:632-642). Event `Approval` (asset:348).
- Spend: only from settlement spender mode (asset `spend_allowance` :2782-2812;
  `InsufficientAllowance` :1969; `AllowanceSpent` event :355). Depletion to zero removes entry.
- **Allowance is consumed *before* instruction creation and is not refunded if the receiver
  later rejects the pending cross-DID transfer** — rejection only releases asset locks
  (settlement:2802). Spenders bear that risk; wallets should surface it.
- Owner/spender granularity is per **account key**, not per identity.
- RPC: `allowance(owner, spender, asset)` runtime API (rpc/runtime-api/src/asset.rs:53-57).
  EVM surface: `FungibleAssetStub.sol` approve/transferFrom (doc 21).

## 7. NFT specifics

`validate_nft_transfer` (nft:632-698): collection exists, cross-DID only, sender count, per-leg
limits (≤ `MaxNumberOfNFTsPerLeg` = 10, `ZeroCount`/dup checks nft:717-731), ownership + not
locked, receiver overflow; controller transfers return early; else sender-holder/asset frozen
checks, receiver DID active, **compliance** (nft:688). **No statistics for NFTs.** Effects:
per-DID `NumberOfNFTs` counts + per-NFT owner reassignment (nft:734-766).

## 8. Dry-run RPCs

- `asset_transfer_report(sender, receiver, asset, value, skip_locked_check)` (asset:3462-3570) —
  accumulates all failing checks; `skip_locked_check=true` ignores locks/frozen when sizing
  balance (used to pre-validate instructions whose locks are already placed).
- `nft_transfer_report` (nft:830-917) analogous.
- `Settlement::transfer_report(leg, skip_locked_check)` dispatches per leg type
  (settlement:3695-3724); `execute_instruction_report` uses `skip_locked_check=true`
  (settlement:3729-3759).

## 9. Invariants & review checklist

- [ ] Same-identity fast paths must verify `from.did == to.did` semantics precisely — any
      relaxation reintroduces unchecked cross-identity movement.
- [ ] Every path that changes identity-level `BalanceOf` must run checkpoint-advance +
      statistics-update; every path that doesn't change it must not (doc 04 §8, doc 11).
- [ ] Custody/permission is checked at affirmation (settlement) or at source-authorization
      (transfer_funds), **never** in `base_transfer` — don't add transfer-time custody checks
      (double-check) or remove affirmation-time ones (hole).
- [ ] Spender mode: allowance spend must precede instruction creation atomically with it
      (`#[transactional]` semantics); NFT spender mode must stay rejected.
- [ ] Receiver-affirmation skip logic: adding new receive paths must consult
      `skip_asset_holder_affirmation`, or opted-in identities lose their protection.
- [ ] `SenderSameAsReceiver` guards exist at instruction creation AND inside base transfers
      (asset:3379, nft:645) — keep the defense in depth (key-unlink edge case covered by
      `base_transfer.rs:429` test).

## 10. Test map

`pallets/runtime/tests/src/asset_pallet/{base_transfer.rs, asset_transfer.rs, allowances.rs,
controller_transfer.rs}`; `settlement_pallet/transfer_funds.rs` (23 scenarios: spender modes,
custody, frozen matrix, NFT variants); `settlement_pallet/reject_instruction.rs`.
