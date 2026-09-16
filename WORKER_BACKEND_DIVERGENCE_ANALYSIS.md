# Worker backend divergence — unbounded `WrappedCanonical` padding → VM guest heap trap → consensus split

## Summary

A signed `create_settlement` extrinsic can carry a valid confidential-asset proof whose leg
ciphertext has been padded with attacker-chosen trailing bytes. The proof still verifies, but the
padded request drives the PolkaVM / Wasmer worker guest past its fixed 20 MiB heap and traps it.
The worker maps that trap to `ExecuteWorkFailed`, and the pallet maps that to `InvalidProof`. A
Native-backend validator, which has no fixed heap, verifies the same request and commits the
settlement. Two validators running different backends therefore reach different state roots from the
same signed transaction — a consensus-safety divergence (chain fork; GRANDPA finality halt when
incompatible cohorts each hold ≳ 1/3 authority weight).

The trap floor sits at ~3.1–3.2 MiB of padding, below the ~7.5 MiB a single Normal-class extrinsic
is allowed, so the divergence is reachable by one in-block transaction.

Reachability is gated on validators running heterogeneous `POLYMESH_WORKER_BACKENDS`. A homogeneous
default set (Native-first) is unaffected because every node accepts.

## Two defects that compose

1. `polymesh-dart` (the outer wrapper crate — the SCALE / substrate-facing layer over the inner
   `polymesh-dart-bp` crypto crate): the leg ciphertext wrapper `WrappedCanonical<T>` has no length
   bound and its decode ignores trailing bytes — the padding lever.
2. Node worker: a guest execution fault (OOM / trap / panic) is caught and returned as
   `ExecuteWorkFailed`, which the pallet turns into `InvalidProof` — the fault-as-verdict conflation
   that turns a per-backend resource limit into a per-backend accept/reject decision.

Neither alone forks the chain. Together they do.

## The wrapper-crate (`polymesh-dart`) defect

Paths are in the `polymesh-dart` wrapper crate — the outer SCALE / substrate-facing crate (the pinned
dependency in this repo), distinct from the inner `polymesh-dart-bp` crypto crate whose
`bp_leg::LegEncryption` is the wrapped type. Line numbers are from the current `polymesh-dart` tree
and may drift by revision.

`WrappedCanonical<T>` stores an unbounded byte vector (`src/bp/encode.rs:398`):

```rust
pub struct WrappedCanonical<T> {
    wrapped: Vec<u8>,
    _marker: core::marker::PhantomData<T>,
}
```

Its SCALE `Decode` accepts the whole caller-supplied vector with no bound (`src/bp/encode.rs:491`),
and `decode()` deserializes `T` from the front of that vector without checking the vector is
exhausted (`src/bp/encode.rs:438`):

```rust
pub fn decode(&self) -> Result<T, Error> {
    Ok(T::deserialize_compressed(&*self.wrapped)?)   // reads what it needs, ignores the rest
}
```

`arkworks` `deserialize_compressed` reads exactly the bytes the object needs and leaves the rest, so
trailing bytes pass through and the proof still verifies. Equality compares raw bytes
(`src/bp/encode.rs:406`), and the on-chain settlement reference hashes the full SCALE encoding —
`SettlementRef(blake2_256(self))` (`src/bp/leg.rs:756`), where `blake2_256<T: Encode>` hashes
`self.encode()` (`src/lib.rs:27`) — so padding changes the stored bytes and the reference while the
decoded value and proof validity are unchanged.

The leg ciphertext uses the unbounded wrapper — `LegEncrypted(WrappedCanonical<LegEncryption>)`
(`src/bp/leg.rs:1333`); same for `MediatorEncryption`. The bounded sibling
`BoundedCanonical<T, S: Get<u32>>` already exists (`src/bp/encode.rs:506`) and is used for the inner
proof (`inner: BoundedCanonical<_, T::MaxInnerProofSize>`, `src/bp/leg.rs:1008`), the legs vector
(`BoundedVec<_, T::MaxSettlementLegs>`, `src/bp/leg.rs:743`) and the memo — but not for the
ciphertext wrappers.

## What "padding" means

The attacker takes a valid leg ciphertext (worst case ~782 bytes: hidden-asset leg, 2 mediators) and
appends attacker-chosen zero bytes to it — for example 3,145,728 zeros (3.0 MiB). Those trailing
zeros are the padding. They sit inside the leg's `wrapped: Vec<u8>`, after the real ciphertext. The
deserializer reads only the real bytes and never inspects the zeros, so the proof verifies. "3.1 MiB
of padding" means 3.1 MiB of junk bytes riding inside one leg.

## Why the VM crashes

### The fixed guest heap

Compiled for PolkaVM / Wasmer (not Native), the dart-v1 worker installs a global allocator over a
static array of fixed size (`worker/protocol/dart-v1/src/lib.rs:29`):

```rust
const HEAP_SIZE: usize = 20 * 1024 * 1024;               // 20 MiB, fixed
static mut GLOBAL_ALLOC: picoalloc::Mutex<
    picoalloc::Allocator<picoalloc::ArrayPointer<{ HEAP_SIZE }>>
> = { static mut ARRAY: picoalloc::Array<{ HEAP_SIZE }> = picoalloc::Array([0; HEAP_SIZE]); … };
```

Plus a separate 10 MiB scratch buffer for host↔guest transport
(`SCRATCH_SIZE = 10 * 1024 * 1024`, `worker/protocol/dart-v1/src/lib.rs:138`). The heap is the
region that matters for the crash. Native runs the same verification code on the host's system
allocator, with no such cap.

### The allocation cascade

The request bytes live in the scratch, not the heap. But decode → verify → respond makes several
full-size copies of the padding on the heap, and they coexist. "Allocating a copy" means the code
asks the allocator for a fresh heap block and writes the padding (or bytes derived 1:1 from it) into
it; every `Vec<u8>` that ends up holding the padding is a separate allocation whose size counts
against the 20 MiB ceiling.

For `create_settlement`:

| Copy | Where | Notes |
|---|---|---|
| WorkRequest inner vector | guest decodes `WorkRequest(pub Vec<u8>)` (`worker/common/src/lib.rs:462`) out of scratch — a heap `Vec<u8>` holding the whole request, padding included | avoidable (decode inner directly from scratch) |
| Decoded `leg_enc.wrapped` | decoding the `SettlementProof` allocates each leg's `wrapped` on the heap — where the padding lands as a live proof field | intrinsic; made tiny by bounding |
| `settlement_ref` re-encode | `id: proof.settlement_ref()` (`worker/protocol/dart-v1/src/verify.rs:301`) is `blake2_256(&proof)`, which re-encodes the whole padded proof into a transient buffer to hash it | avoidable (host already computes it) |
| Response leg echo + encoded response | the `CreateSettlement` handler clones every padded leg — `proof.legs.iter().map(|leg| leg.leg_enc().clone()).collect()` (`worker/protocol/dart-v1/src/verify.rs:299`) — into a `Vec<LegEncrypted>`, then SCALE-encodes the response | avoidable; the pallet discards it (see below) |

The padding is never inspected — `deserialize_compressed` ignores it, so verification would succeed.
The failure is a pure resource fault from copying dead weight.

### The crash chain

When an allocation cannot be satisfied in the 20 MiB arena:

1. `picoalloc`'s `GlobalAlloc::alloc` returns a null pointer.
2. Rust's allocation layer calls `handle_alloc_error`.
3. In the `no_std` guest that aborts — a wasm `unreachable` / PolkaVM trap. The instance faults
   mid-execution.
4. The host's `call_execute` gets `Err` from the VM runtime and maps it
   (`worker/src/backend.rs:308-321`) to `Ok(Err(ProtocolError::ExecuteWorkFailed))`. The code comment
   there states the intent: a failed `call_execute` is treated as the guest having panicked, so the
   proof is rejected.
5. The pallet turns any worker error into `Error::InvalidProof`
   (`pallets/confidential-assets/src/lib.rs:2972`).

The heap-probe measurement pinned the peak crossing 20,971,520 bytes (= 20 × 1024 × 1024) exactly at
the trap boundary — unambiguously the picoalloc ceiling, not the stack or scratch.

### Why Native survives the same input

Native is the same code on the host allocator. It allocates the same copies (tens of MiB) from
process RAM, completes verification, returns valid, and commits. Same bytes, same logic, different
allocator ceiling, opposite outcome. That asymmetry, converted to a verdict at step 5, is the bug.

## The memory math

Peak heap fits `peak(P) = B + m·P`, where `P` is padding size, `B` is the baseline (resident
parameter tables + verify working set at zero padding), and `m` is the measured amplification slope.

From the 16-leg settlement measurement: `B = 8.21 MiB`, `peak(3.0) = 19.41 MiB`, so
`m = (19.41 − 8.21) / 3.0 ≈ 3.7`. The trap floor solves `B + m·P = 20`:

```
P_floor = (20 − 8.21) / 3.7 ≈ 3.2 MiB
```

matching the observed floor of (3.00, 3.12] MiB.

Common mistake: `3.1 × 3.7 ≈ 11.5 MiB` is only the padding-driven term. The verifier already uses
~8 MiB at zero padding (resident tables + batched multi-leg verify), and that does not go away.
`11.5 + 8.2 ≈ 19.7 → 20 MiB wall`. The baseline is what makes the crossing happen at ~3.2 MiB rather
than at ~5.4 MiB (= 20 / 3.7). The "3.7" is an empirical slope, not a whole-number count of
duplicates: transient copies only partially overlap the peak, and encoded copies carry length-prefix
overhead, so the effective factor is fractional.

## Proof of concept

### Method

The worker's own test harness (`worker/tester`) drives a chosen backend directly against a
`WorkRequest`, so no multi-node network is needed. The leg is padded by bumping the SCALE length
prefix of its `wrapped` vector and appending zeros, which keeps the decoded value (and proof
validity) unchanged:

```rust
fn pad_leg(raw_leg_enc: &[u8], pad_len: usize) -> LegEncrypted {
    let mut input = &raw_leg_enc[..];
    let Compact(canonical_len) = Compact::<u32>::decode(&mut input).unwrap();
    let mut expanded = Compact((canonical_len as usize + pad_len) as u32).encode();
    expanded.extend_from_slice(input);
    expanded.resize(expanded.len() + pad_len, 0);
    <LegEncrypted as Decode>::decode(&mut &expanded[..]).unwrap()
}
```

The sweep runs each padded request on `native`, `wasmer`, and `polkavm` and records the raw nested
result: `Ok(Ok(_))` = processed (valid/invalid), `Ok(Err(ExecuteWorkFailed))` = guest trap,
`Err(WorkerError::ModuleMemoryError)` = host scratch cap. Peak heap is measured (Phase 2) with a
feature-gated `ProbeAlloc` high-water counter around the picoalloc allocator.

### Results — `SenderAffirmation` (smallest baseline; wasmer ≡ polkavm)

Native returns `Ok(Ok(valid))` at every size 0 → 20 MiB.

| pad | native | wasmer / polkavm |
|---|---|---|
| 0 – 3.9 MiB | Ok(Ok) | Ok(Ok) |
| 5.0 MiB | Ok(Ok) | Ok(Ok) |
| 5.25 MiB | Ok(Ok) | Ok(Ok) |
| 5.5 MiB | Ok(Ok) | iter0 Ok; iter1-3 trap |
| 5.75 MiB | Ok(Ok) | iter0 Ok; iter1-3 trap |
| 6.0 MiB | Ok(Ok) | ExecuteWorkFailed |
| 8.0 – 9.5 MiB | Ok(Ok) | ExecuteWorkFailed |
| ≥ 10 MiB | Ok(Ok) | ModuleMemoryError |

Floor: ~6.0 MiB cold (fresh instance), ~5.5 MiB warm (reused instance). Resident params ~4.13 MiB;
baseline peak 5.66 MiB; slope ~2.4× cold to 3.0× warm. This request type alone would be a down-rate:
its floor exceeds the block budget.

### Results — `create_settlement` (the attacked path; wasmer ≡ polkavm)

Native accepts all pads. Worst case: hidden-asset legs, 2 mediators (maximizes baseline).

| legs | unpadded proof | B (peak @ pad 0) | peak @ 3.0 MiB | trap floor |
|---|---|---|---|---|
| 1 | 7.8 KB | 7.10 MiB | 19.15 MiB | (3.12, 3.25] MiB |
| 2 | 15 KB | 7.16 | 19.17 | (3.12, 3.25] |
| 4 | 30 KB | 7.31 | 19.20 | (3.12, 3.25] |
| 8 | 61 KB | 7.61 | 19.27 | (3.12, 3.25] |
| 16 | 122 KB | 8.21 | 19.41 | (3.00, 3.12] |

At pad 3.12 MiB: native `Ok(Ok)`, wasmer and polkavm `Ok(Err(ExecuteWorkFailed))`. Above ~10 MiB
(`req_len` > 10,485,760) the host hits the scratch cap in `allocate_scratch_pad` →
`Err(ModuleMemoryError)`, which is a host error (not a guest trap) and triggers the runtime fallback
to Native → no divergence. That self-defeating regime is unreachable in one extrinsic anyway (block
length cap).

The floor barely depends on leg count — it is driven by total padding, not how it is split — and
sits ~2.4 MiB below the affirmation floor for two compounding reasons: a higher baseline (batched
multi-leg verify + larger legs) and an extra pad-sized copy from the response echo (the response was
observed growing 827 B → 10.5 MB with padding), giving slope ~3.7× vs ~3.0×.

### Consensus reachability

- Block length: a Normal-class extrinsic (`create_settlement`) is capped at
  `NORMAL_DISPATCH_RATIO 75% × MaximumBlockLength 10 MiB = 7.5 MiB`
  (`pallets/runtime/common/src/lib.rs:56`, `:76`, `:117`). A ~124 KB proof + ~3.1 MiB padding fits
  one block, above the ~3.2 MiB floor.
- Decode gate: the padded field is a raw `Vec<u8>`; SCALE decode allocates ≈ its own byte length
  (1:1, no amplification), so `DecodeWithMemTracking` — which exists to stop amplification decode
  bombs — has nothing to trip on. No per-decode budget below the length cap.
- Cross-validation: the external report's live three-validator run included a 4,106,353-byte
  extrinsic in a block on both branches and observed the fork at block 18. A 4.1 MiB padding is in
  the settlement trap window (3.2, 10) MiB, so the live observation matches this PoC. The first
  affirmation sweep contradicted the report only because it measured the wrong (smaller-baseline)
  request type.

### Repro

```sh
# Affirmation sweep (prebuilt blobs):
cargo run -p polymesh-worker-tester --release -- {native|wasmer|polkavm}

# Settlement sweep (generates fixtures under $SETTLEMENT_FIXTURE_DIR, default cwd):
cargo run -p polymesh-worker-tester --release --bin settlement_poc -- native 16

# Phase 2 peak heap (rebuild guest with the probe, then RUST_LOG=warn):
cd worker && cargo build --target=wasm32-unknown-unknown --no-default-features \
  --features wasm,heap_probe --release --lib -p polymesh-worker-protocol-dart-v1
```

PoC changes (guest instrumentation feature-gated behind `heap_probe`; the checked-in `.wasm` /
`.polkavm` blobs are the clean default build and are untouched):

```
M worker/tester/src/main.rs                  # pad_leg + affirmation padded-leg sweep
?? worker/tester/src/bin/settlement_poc.rs   # N-leg create_settlement generator + pad sweep
M worker/tester/Cargo.toml                   # rand_core / rand_chacha deps
M worker/protocol/dart-v1/src/lib.rs         # ProbeAlloc high-water probe (heap_probe feature)
M worker/protocol/dart-v1/Cargo.toml         # heap_probe feature
```

## Can we avoid the allocations?

Yes — three of the four padding copies are wasteful and removable. The key finding: the worker's
`create_settlement` response is discarded by the pallet.

`base_create_settlement` computes `settlement_ref` on the host (`pallets/confidential-assets/src/lib.rs:1954`),
clones the request's legs for storage (`proof.legs.clone()`, `:1969`), and calls
`submit_and_wait(...)?` (`:1983`). `submit_and_wait` returns `DispatchResult`
(`:2970`) — it only checks Ok/Err and throws the response payload away. So the worker's echoed `legs`
and `id` are built, serialized, transported, and discarded.

| Copy | Avoidable | How |
|---|---|---|
| Response leg echo + encoded response | Yes | Pallet discards it — return only what is used (or nothing). Biggest single reduction; it is the difference between settlement's 3.7× and affirmation's 3.0×. |
| Guest `settlement_ref` re-encode | Yes | Redundant — the host computes `settlement_ref` too. Drop from the response. |
| WorkRequest `Vec<u8>` indirection | Mostly | Decode the inner request directly from the scratch slice instead of copying it onto the heap first. |
| Proof's own `leg_enc.wrapped` | No | Intrinsic proof field — but bounding the wrapper makes it ~1 KB, so it stops mattering. |

Caveat: reducing allocations is defense-in-depth, not a fix. If the slope drops to ~1× then
`peak ≈ 8 + 1·P`, and the 7.5 MiB block cap gives `peak ≈ 15.4 MiB < 20 MiB` → no single-extrinsic
trap. But that margin is fragile: the ~8 MiB baseline is already most of the budget and grows with
future legs / protocol changes, warm fragmentation adds ~2 MiB, and it still leaves unbounded
attacker-controlled input size "handled" by downstream memory accounting rather than rejected at the
boundary. Optimizing allocations chases the symptom; bounding the input removes the cause.

## The fixes, ranked

1. Bound the wrapper (primary, input side). Switch `LegEncrypted` / `MediatorEncryption` — and every
   consensus-crossing `WrappedCanonical` (account, fee, auth device responses) — to
   `BoundedCanonical<_, S>` with a real ciphertext-size bound (a few KB), the pattern the inner proof
   already uses (`src/bp/leg.rs:1008`). `BoundedVec` rejects an oversized length at decode, before
   allocating. Once no request can carry MBs, the amplification is over a ~1 KB payload and is
   irrelevant. This alone closes the reachable vector.
2. Reject trailing bytes (cheap, correct). Add an input-exhaustion check to the canonical decode (and
   to `BoundedCanonical::decode`, `src/bp/encode.rs:547`, which also lacks it):

   ```rust
   pub fn decode(&self) -> Result<T, Error> {
       let mut input = &self.wrapped[..];
       let value = T::deserialize_compressed(&mut input)?;
       if !input.is_empty() {
           return Err(Error::TrailingCanonicalBytes);
       }
       Ok(value)
   }
   ```

   Kills residual within-bound malleability of `settlement_ref` and the stored bytes. Safe: `wrap()`
   always produces exact bytes, so no honest data is rejected.
3. Fault ≠ `InvalidProof` (backstop, node side). A guest trap / OOM must not be re-labeled as a proof
   verdict — fail uniformly across backends or run the deterministic fallback
   (`worker/src/backend.rs:308-321` → `pallets/confidential-assets/src/lib.rs:2972`). This closes the
   class: the next divergence source will not be size (a panic, a platform difference), and only this
   fix covers those.
4. Trim the wasteful copies (optional; perf + blast radius). Drop the discarded response echo
   (`worker/protocol/dart-v1/src/verify.rs:299`) and the redundant guest `settlement_ref`, decode
   from scratch. Do for normal-case memory, not as the safety mechanism.

## Severity and reachability

- HIGH at minimum: one signed `create_settlement` makes any VM-only validator diverge from Native
  validators; they reject each other's blocks (fork off / liveness loss for the VM cohort).
- Critical (GRANDPA finality halt) when incompatible Native and VM cohorts each hold ≳ 1/3 authority
  weight.
- Single gating condition, deployment-side: a validator cohort running VM-only backends (Native
  removed from `POLYMESH_WORKER_BACKENDS`). Default nodes are Native-first and accept, so a
  homogeneous default set is unaffected. This is a configuration fact, not a code gate — it is the
  only thing between "reachable" and "actively forking a given network".

## Key references

`polymesh-dart` wrapper crate (the pinned dependency; lines from current tree, may drift by revision):
- `src/bp/encode.rs:398` — `WrappedCanonical` (unbounded `Vec<u8>`)
- `src/bp/encode.rs:438` — `decode()` with no exhaustion check
- `src/bp/encode.rs:491` — `Decode` accepts the whole vector
- `src/bp/encode.rs:506`, `:547` — `BoundedCanonical` (bounded sibling; its `decode` also lacks the exhaustion check)
- `src/bp/leg.rs:756` — `settlement_ref = blake2_256(self)`
- `src/bp/leg.rs:1008`, `:743` — inner proof and legs already bounded
- `src/bp/leg.rs:1333` — `LegEncrypted(WrappedCanonical<…>)` unbounded
- limits in the `polymesh-dart-common` crate: `dart-common/src/lib.rs:48` `SETTLEMENT_MAX_LEGS = 16`, `:56` `MAX_INNER_PROOF_SIZE = 10 KiB`, `:24`/`:25` `MAX_ASSET_AUDITORS`/`MAX_ASSET_MEDIATORS = 2`

Node:
- `worker/protocol/dart-v1/src/lib.rs:29` `HEAP_SIZE = 20 MiB`, `:138` `SCRATCH_SIZE = 10 MiB`, `:442` `execute`
- `worker/src/backend.rs:296` host `execute`, `:308-321` trap → `ExecuteWorkFailed`
- `worker/common/src/lib.rs:462` `WorkRequest(pub Vec<u8>)`
- `worker/protocol/dart-v1/src/verify.rs:48` `CreateSettlement`, `:298-304` response echo + `settlement_ref`
- `pallets/confidential-assets/src/lib.rs:1953` `base_create_settlement`, `:1969` request legs, `:1983` submit, `:2970` `submit_and_wait` (`DispatchResult`), `:2972` `→ InvalidProof`
- `pallets/runtime/common/src/lib.rs:56` `NORMAL_DISPATCH_RATIO = 75%`, `:76` `MaximumBlockLength = 10 MiB`, `:117` per-class length
