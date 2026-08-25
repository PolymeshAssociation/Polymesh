# 13 — STO (Fundraising)

Sources: `pallets/sto/src/lib.rs`, `primitives/src/sto.rs`.
Related specs: [10-settlement](10-settlement.md) (instructions execute the swap),
[05-external-agents](05-external-agents.md) (`PolymeshV1PIA` group), [08-portfolio](08-portfolio.md)
(offering locks).

## 1. Purpose

Primary-issuance fundraisers: an asset agent offers `offering_asset` from a portfolio at tiered
prices against `raising_asset` (or off-chain funds). Investments settle atomically through the
settlement engine, so **asset compliance and transfer restrictions fully apply** to both legs.

## 2. Data model (pallets/sto/src/lib.rs)

- `Fundraiser { creator, offering_portfolio, offering_asset, raising_portfolio, raising_asset,
  tiers, venue_id, start, end: Option, status, minimum_investment }` (:131-159).
- `FundraiserTier { total, price, remaining }` (:177-188); ≤ `MAX_TIERS = 10` (:81). Prices are
  fixed-point ×10⁶.
- `FundraiserStatus`: `Live | Frozen | Closed | ClosedEarly` (:90-103).
- `FundingMethod::OnChain(PortfolioId) | OffChain(FundraiserReceiptDetails)` (:105-119);
  STO-specific receipts (`FundraiserReceipt`, primitives/src/sto.rs:40-51) are distinct from
  settlement leg receipts.
- Storage: `Fundraisers` (:384), `FundraiserCount` (:398), `FundraiserNames` (:403),
  `FundraiserOffchainAsset` (asset+id → Ticker; presence enables off-chain funding, :417-427).

No protocol fees anywhere in this pallet.

## 3. Extrinsics

| Extrinsic (idx) | Who | Behavior | Ref |
|---|---|---|---|
| `create_fundraiser(0)` | agent of offering asset (`ensure_agent_asset_perms` :498) + custody of offering & raising portfolios (:504-513) | venue must exist, be creator's, and be `VenueType::Sto` (:500-502); 1..=10 tiers, totals > 0 (:515-525); `start < end` (:530); **locks the total offering amount** (:538-542); status Live | :477-572 |
| `invest(1)` | **any DID** with custody of the investment (+funding) portfolios | §4 | :605 → :856-1076 |
| `freeze_fundraiser(2)` / `unfreeze_fundraiser(3)` | agent | toggle Frozen/Live (not-closed guard) | :645/:673 → :1078-1104 |
| `modify_fundraiser_window(4)` | agent | not closed & not expired; new `start < end` | :706-741 |
| `stop(5)` | **creator DID** (asset-perm check only) or any permissioned agent (:771-775) | sums tier `remaining`, **unlocks it** (:785-789); status `ClosedEarly` (end in future) else `Closed` (:790-793) | :762-802 |
| `enable_offchain_funding(6)` | creator or agent (:832-838) | registers the off-chain ticker | :824-851 |

`AgentGroup::PolymeshV1PIA` = all Sto extrinsics **except `invest`** + Asset
issue/redeem/controller_transfer (external-agents lib.rs:688-700).

## 4. `invest` flow (:856-1076)

1. Fundraiser Live (:879-882) and within `[start, end)` (:884-888).
2. **Tier consumption is vector order** (creation order — the "lowest price first" doc comment
   at :577-578 is inaccurate; tiers are not sorted): skip empty tiers, buy
   `min(tier.remaining, wanted)` per tier (:905-933). Purchase must be fully fillable
   (`InsufficientTokensRemaining` :935).
3. Cost = Σ `amount_in_tier × tier.price / 1_000_000` (checked math, :927-932);
   `cost ≥ minimum_investment` (:936-939); slippage guard
   `cost ≤ max_price × purchase_amount / 1_000_000` (:940-945).
4. Legs: always `offering_portfolio → investment_portfolio` for the offering asset (:960-965).
   - **OnChain funding**: second leg `funding_portfolio → raising_portfolio` for the raising
     asset at `cost` (:982-987); custody of the funding portfolio required (:968-972).
   - **OffChain funding** (:990-1016): requires `enable_offchain_funding`; validates an
     STO receipt — signer must be a **venue signer**, uid replay-protected via settlement's
     `ReceiptsUsed` (`mark_receipt_as_used`, sto:993-997 → settlement lib.rs:3138-3148);
     signature over `ChainScopedMessage { …, "Polymesh STO Fundraiser Receipt",
     FundraiserReceipt { fundraiser_id, investor, raiser, ticker, cost } }`
     (crypto.rs:86; :998-1014). **No second leg is added** — payment happens off-chain.
5. Offering amount is unlocked (:1019-1023), then a settlement instruction is created on the
   fundraiser's venue (`SettleOnAffirmation`, :1025-1034), the fundraiser side auto-affirmed
   (:1036-1050), and the investor side affirmed+executed **in the same transaction**
   (`affirm_and_execute_instruction`, settlement lib.rs:2618-2659 — STO-only entry point,
   executes immediately and non-retryably when no affirmations remain).
   Compliance/statistics run inside normal instruction execution.
6. Tier `remaining` decremented post-settlement (:1060-1062); `Invested` event.

Receiver-affirmation policy applies: the investor's portfolio is only added to the affirmation
set if their identity opted into mandatory receiver affirmation (:951-959, doc 09 §5).

## 5. Lifecycle notes

- Expiry is enforced only at invest time; an expired fundraiser keeps its remaining offering
  **locked until `stop` is called** — no automatic close/unlock.
- Sell-out does not auto-close (`Live` until `stop`).
- Frozen fundraisers reject investments but keep locks.

## 6. Invariants & review checklist

- [ ] Offering lock accounting: locked amount must always equal Σ tier `remaining` (lock at
      create, unlock per-invest and at stop). A drift strands or double-spends offering tokens.
- [ ] Investment must be atomic: unlock (:1019) is only sound because instruction creation +
      execution happen in the same transactional extrinsic — don't split this flow.
- [ ] Off-chain receipts: venue-signer check + `ReceiptsUsed` replay protection must precede any
      state change; STO receipts and settlement receipts share the uid replay space per signer.
- [ ] `stop`'s creator bypass (:771-775) intentionally lets the original creator wind down even
      after losing agent perms — reassess if creator trust model changes.
- [ ] Tier math is checked arithmetic throughout; price precision 10⁶ must match UI expectations.

## 7. Test map

`pallets/runtime/tests/src/sto_test.rs` (happy path :105-271 incl. same-block settlement
`Success`; unhappy :273; invalid fundraiser :417; expiry :513; window :560; freeze :615;
stop :651).
