# Polymesh precompiles (interface crate)

This crate contains the Solidity source used for:

- Rust ABI generation via alloy `sol!`
- the checked-in runtime bytecode served by the runtime precompile `CODE`
- Blockscout source verification

Current source of truth:

- `src/interfaces/FungibleAssetStub.sol`

Compiled runtime artifact:

- `src/interfaces/FungibleAssetStub.bin`

## Prerequisites

Run these against a local stack where:

- Polymesh dev chain is running with your precompile changes
- `eth-rpc` is running (default: `http://127.0.0.1:8545`)
- Blockscout backend is running (default: `http://127.0.0.1:4001`)

## 1) Build and verify the checked-in bytecode artifact

Generate the runtime bytecode from Solidity:

```bash
./scripts/build_precompile_stub.sh
```

Check that the checked-in artifact is up to date:

```bash
./scripts/build_precompile_stub.sh --check
```

## 2) Find candidate precompile token addresses

This script scans `Transfer` logs and prints unique token contract addresses:

```bash
./scripts/blockscout_find_precompile_addresses.sh
```

Optional env vars:

- `RPC_URL` (default `http://127.0.0.1:8545`)
- `FROM_BLOCK` (default `0x0`)
- `TO_BLOCK` (default `latest`)
- `TRANSFER_TOPIC` (default ERC-20 `Transfer` topic)

## 3) Confirm on-chain code matches the checked-in artifact

For an address from step 2:

```bash
./scripts/blockscout_check_precompile_code.sh 0x<address>
```

Expected result:

- `MATCH: on-chain code equals FungibleAssetStub.bin`

If it reports empty code (`0x`) or mismatch, restart with the latest chain/eth-rpc binaries and check again.

## 4) Submit verification to Blockscout

```bash
./scripts/blockscout_verify_precompile.sh 0x<address>
```

Expected immediate response:

- `{"message":"Smart-contract verification started"}`

## 5) Poll verification status

```bash
./scripts/blockscout_precompile_status.sh 0x<address>
```

Typical successful precompile outcome:

- `is_verified: true`
- `is_partially_verified: true`
- non-zero `abi_len`

`Partial Match` is expected for precompiles because there is no deployment transaction / creation bytecode to fully match.

## 6) Verify one address only

You only need to verify one address per precompile interface. Other addresses with the same runtime bytecode can pick up ABI metadata through Blockscout twin matching.

## Helper scripts

- `scripts/build_precompile_stub.sh`
- `scripts/blockscout_find_precompile_addresses.sh`
- `scripts/blockscout_check_precompile_code.sh`
- `scripts/blockscout_verify_precompile.sh`
- `scripts/blockscout_precompile_status.sh`
