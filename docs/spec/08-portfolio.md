# 08 — Portfolios & Custodianship

Sources: `pallets/portfolio/src/lib.rs`, `primitives/src/identity_id.rs` (PortfolioId),
`primitives/src/portfolio.rs` (Fund).
Related specs: [02-permissions](02-permissions.md) (portfolio subset checks),
[09-asset-transfers](09-asset-transfers.md), [10-settlement](10-settlement.md) (custody checked
at affirmation), [13-sto](13-sto.md), [12-corporate-actions](12-corporate-actions.md) (lock users).

## 1. Purpose

Portfolios partition an identity's asset holdings. Each portfolio can be placed under the
**custody** of another identity — the custodian (not the owner) then controls fund movements out
of it. Assets can also be held directly by account keys (doc 09 §7); portfolios are the
identity-native holding container.

## 2. Data model

- `PortfolioId { did, kind }`; `PortfolioKind = Default | User(PortfolioNumber)`
  (primitives/src/identity_id.rs:260-265, 294-302). The Default portfolio always exists;
  User portfolios are created explicitly (numbers from 1, identity_id.rs:244-248).
- `Fund { description: Fungible { asset_id, amount } | NonFungible(NFTs), memo }`
  (primitives/src/portfolio.rs:25-52) — the unit of movement.

### Storage (pallets/portfolio/src/lib.rs)

| Item | Key → Value | Ref |
|---|---|---|
| `Portfolios` / `NameToNumber` / `NextPortfolioNumber` | naming + existence of user portfolios | :222/:236/:215 |
| `PortfolioAssetBalances` / `PortfolioAssetCount` | fungible balances / count of nonzero assets | :254/:249 |
| `PortfolioLockedAssets` | locked amount per (portfolio, asset) | :267 |
| `PortfolioNFT` / `PortfolioLockedNFT` | held / locked NFTs | :291/:304 |
| `PortfolioCustodian` | portfolio → custodian DID; `None` ⇒ owner | :279 |
| `PortfoliosInCustody` | reverse custody index | :283 |
| `PreApprovedPortfolios` | (portfolio, asset) receive-affirmation skip | :316 |
| `AllowedCustodians` | (owner, trusted) → bool — may create custody portfolios | :320 |
| `PortfolioFrozenAssets` / `FrozenPortfolios` | frozen amount / frozen flag per (portfolio, asset), written by asset pallet (doc 04 §3) | :325/:337 |

## 3. Extrinsics & authorization

| Extrinsic (call_index) | Who may call | Behavior | Ref |
|---|---|---|---|
| `create_portfolio(0)` | any permissioned DID | unique name; number from sequence | :410 → :672 |
| `delete_portfolio(1)` | **owner who still holds custody** + portfolio perms | requires zero assets & zero NFTs (`PortfolioNotEmpty` :439-446) | :425 |
| `rename_portfolio(2)` | **owner** (portfolio perm; no custody check :493-497) | unique-name rename | :479 |
| `quit_portfolio_custody(3)` | current custodian | custody reverts to owner (:534-538) | :527 |
| `accept_portfolio_custody(4)` | auth target | consume `PortfolioCustody` auth (§4) | :542 → :850 |
| `move_portfolio_funds(5)` | **custodian of source** (+ portfolio perms) | same-identity move (§5) | :566 |
| `pre_approve_portfolio(6)` / `remove_portfolio_pre_approval(7)` | **custodian** | toggle per-portfolio receive pre-approval | :595/:614 → :1016/:1038 |
| `allow_identity_to_create_portfolios(8)` / `revoke_create_portfolios_permission(9)` | owner | manage `AllowedCustodians` (self-add rejected :1065) | :629/:643 |
| `create_custody_portfolio(10)` | trusted DID (in `AllowedCustodians`) | creates portfolio under **owner's** DID, custody immediately to caller — no auth round-trip (:1090-1114) | :658 |

No protocol fees in this pallet. `MaxNumberOfFungibleMoves = 10` / `MaxNumberOfNFTsMoves = 100`
bound weights (`pallets/runtime/develop/src/runtime.rs:155-156`).

## 4. Custody model

- Resolver: `custodian(pid) = PortfolioCustodian.unwrap_or(pid.did)` (:693-696).
- Transfer: current custodian (owner initially) issues `AuthorizationData::PortfolioCustody(pid)`
  (primitives/src/authorization.rs:49); target accepts (`base_accept_portfolio_custody` :850-875).
  Rules: **Default portfolios cannot have custodians** (:855-858); auth must be issued by the
  *current custodian* (:860-861) — custody can be passed onward; accepting as the owner resets to
  `None` (:865-867).
- Rights split:

| Action | Owner | Custodian |
|---|---|---|
| move funds out | only if custodian | yes (:902-906) |
| delete | only if still custodian (:450-454) | no (not owner) |
| rename | yes (no custody needed) | no |
| pre-approve receives | no (unless custodian) | yes (:1023-1027) |
| settlement affirmation for the portfolio | custody required (settlement lib.rs:1686-1692) | yes |
| receiver-affirmation policy governing DID | custodian if set, else owner (:1181-1182) | — |

Layer-3 permission checks (doc 02 §5): `ensure_portfolio_custody` (:798),
`ensure_user_portfolio_permission` (:782, secondary-key subset),
`ensure_portfolio_custody_and_permission` (:814), `ensure_portfolio_validity` (:759).

## 5. `move_portfolio_funds` (same-identity moves)

`base_move_portfolio_funds` (:566-584 → checks :888-911, :915-945, effects :967-1014):
1. `from != to` (`DestinationIsSamePortfolio`); **`from.did == to.did` required**
   (`DifferentIdentityPortfolios` :897) — strictly intra-identity.
2. Source: custody + portfolio permission (:902-906). Destination: validity + secondary-key
   portfolio permission only — **destination custody not required** (:909).
3. Per fund: amount > 0, no duplicate assets, source portfolio not frozen for the asset,
   asset not frozen, sufficient free balance (balance − locked, :921-931); NFTs owned & unlocked
   (:948-964).
4. **No compliance/statistics/checkpoint involvement** — identity-level `BalanceOf` unchanged.
5. Funds in deleted portfolios remain recoverable via this call (source existence not re-checked;
   doc comment :550).

## 6. Locks (who locks portfolio assets)

`PortfolioLockedAssets` amounts stack (`unchecked_lock_tokens` :844-848); locked balance shows in
balance but blocks moves/redeem/transfers (available = balance − locked − frozen).

| Locker | Lock | Unlock | Ref |
|---|---|---|---|
| Settlement affirmation | `lock_asset` per leg on affirm | on reject/withdraw/execution | settlement lib.rs:1697-1713/1715-1731 → asset lib.rs:3774/3794 |
| STO | offering locked at fundraiser creation | on stop / per-investment | pallets/sto/src/lib.rs:538-542, 785-789, 1019-1023 |
| Capital distributions | CAA locks distribution amount | reclaim/remove/per-claim | pallets/corporate-actions/src/distribution/mod.rs:699, 607-610 |
| NFT locks | `lock_nft`/`unlock_nft` | — | :1164/:1169 via nft lib.rs:1067/1090 |

## 7. Invariants & review checklist

- [ ] All fund-out paths must check **custody of the source** (`ensure_portfolio_custody*`);
      destination custody is deliberately not required — receiving is gated by affirmation
      policy instead (doc 09 §5).
- [ ] `move_portfolio_funds` must stay same-identity (`DifferentIdentityPortfolios`) — relaxing
      it would bypass compliance/statistics entirely.
- [ ] `PortfolioAssetCount` must track 0↔nonzero balance transitions (`transition_asset_count`
      :748-756) — delete-empty depends on it.
- [ ] Locked ≤ balance must hold; lock without balance check only via trusted internal paths
      (`unchecked_lock_tokens` callers).
- [ ] Default portfolios: always valid, cannot be deleted/renamed/custodied — check new code
      doesn't assume a `Portfolios` entry exists for them.
- [ ] `create_custody_portfolio` bypasses the auth round-trip by design; it must stay gated on
      `AllowedCustodians` (`MissingOwnersPermission` :1097-1100).

## 8. Test map

`pallets/runtime/tests/src/portfolio.rs` (locks :389/:473, custody auths :56/:565/:1222,
affirmation-skip :1025-1073); `settlement_pallet/transfer_funds.rs` (custody paths :345/:382);
`asset_pallet/issue.rs` (:154). Integration: `integration/tests/portfolio.rs`,
`portfolio_custody.rs`.
