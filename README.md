[![Telegram](https://img.shields.io/badge/50k+-telegram-blue.svg)](https://t.me/polymathnetwork)

![Polymath logo](Polymath.png)

# Polymesh Blockchain

Polymesh is a blockchain for regulated securities and open finance.

# Whitepaper

<https://polymath.network/polymesh-whitepaper>

# Audit

See the `audit` folder for details of audits undertaken on the Polymesh code base.

Independent audits were completed by:  
https://www.srlabs.de/  
https://www.atredis.com/

# Networks

We run two public networks, the mainnet and the testnet.

Generally these two networks will be at the same version of Polymesh, although they may differ briefly during an upgrade cycle.

We provide linux binaries for each release.

The latest release for Polymesh can be found at:  
<https://github.com/PolymeshAssociation/Polymesh/releases>

Generally you should be able to run the latest release for both networks, although the on-chain version of the network might differ during upgrade cycles.

Below are simple instructions for running a non-operating node (i.e. a node that does not produce blocks or vote on the correctness of other blocks).

For more details on monitoring infrastructure for nodes and running an operator node, see the https://github.com/PolymeshAssociation/polymesh-tools repository.

## Polymesh Testnet

The Testnet does not offer incentives to users to participate and test with it. It has a simple onboarding process (no-KYC required) and a bridge allowing test KOVAN based POLY to be bridged to testnet POLYX.

The testnet also includes the `testUtils` pallet that allows easier onboarding for testing, and in addition each new account will receive 100,000 POLYX for testing purposes.

To run a node which connects to the Testnet, you can start your node with:

```bash
./target/release/polymesh --chain testnet
```

## Polymesh Mainnet

The public mainnet is the official Polymesh blockchain. Onboarding requires users to go through a simple KYC process (called Customer Due Diligence or CDD) in order to access the network.

ERC20 POLY can be bridged from Ethereum to the Polymesh Mainnet.

To run a node which connects to the Mainnet, you can start your node with:

```bash
./target/release/polymesh --chain mainnet
```

# Operators (aka Validators)

A guide to running an operator node can be found at:

<https://github.com/PolymeshAssociation/polymesh-tools/tree/main/docs/operator>

# Documentation

Further details on Polymesh concepts and networks can be found at:

<https://developers.polymesh.live/>

Code documentation can be found at:

<https://docs.polymesh.live/>

# Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Install required tools:

```bash
./scripts/init.sh
```

Build Wasm and native code:

```bash
cargo build --release
```

Run unit tests:

```bash
./scripts/test.sh
```

# Branches

- The `mainnet` branch tracks code deployed to the Polymesh Public Mainnet.
- The `testnet` branch tracks code deployed to the Polymesh Public Testnet.
- The `staging` branch tracks mainnet except during a release cycle where it is upgraded ahead of mainnet.
- The `tooling` branch tracks the next candidate release for mainnet.
- The `develop` branch is the working branch with the latest code changes.

# Development

## Single node development chain

You can start a development chain with:

```bash
./target/release/polymesh --dev
```

Detailed logs may be shown by running the node with the following environment variables set:
`RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/polymesh --dev`.

[Web Interface]: https://app.polymesh.live/#/explorer

To access the Polymesh Chain using the [Web Interface] do the following:

1. Click on the Polymesh logo in the top-left corner of the UI. You can then select "Local Node" under the Development section.

   > Note: if the `polymesh` node above is on a different machine than your browser (e.g., a server on your local network), you'll need to use a *"custom endpoint"*, e.g., `ws://192.168.0.100:9944/`.
   > The [Web Interface] uses `https`, but your `polymesh` instance does not, so you'll need `ws://` as opposed to `wss://`. You'll also need to use `http://httpapp.polymesh.live/` instead of [Web Interface]. Otherwise, you'll have problems with mixed-content blocking (https vs. http).
   > Finally, add `--rpc-external --ws-external --rpc-cors all` to the `polymesh` invocation above.

2. If you have [custom types definitions](https://github.com/PolymeshAssociation/Polymesh/blob/develop/polymesh_schema.json) that differ from the Polymesh Testnet, you can update these in [Settings](https://app.polymesh.live/#/settings) tab under the `Developer` section.

3. Reload the page.

## Multi-node local testnet

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units.

```bash
cd scripts/cli
npm install
./run.sh
```

This uses pm2 to run a local three node network for demonstrate simple consensus.

To stop the chain you can use:

```bash
./stop.sh
```

and to display log files you can use:

```bash
./log.sh
```

# Unit Tests

Unit tests are packaged with the Rust code. To run these, you can execute:

```bash
cargo test --package polymesh-runtime-tests  --features default_identity
cargo test --package pallet-staking
cargo test --package pallet-balances
cargo test --package polymesh-primitives
cargo test --package pallet-pips-rpc
cargo test --package pallet-transaction-payment
```

# Initialise

You can seed the network with some identities, claims, signing keys and assets by running the functional test.

```bash
cd scripts/cli
node run test
```

See [README](https://github.com/PolymeshAssociation/Polymesh/tree/develop/scripts/cli) for details.

# Benchmarks

Polymesh runtime benchmarks can be run with a command that specifies the pallet and the name of the
extrinsic to be benchmarked, for example:

```bash
cargo run --release --features runtime-benchmarks -- \
    benchmark pallet -p="*" -e="*"
```

Note that the CLI binary should be built in release mode and that the feature flag
`runtime-benchmarks` should be set to enable the CLI option `benchmark`.

# Debug

## Environment

Install GDB for your distribution.

## Build

Binary should be built in *debug mode*, using `cargo build` without `--release` parameter:

```bash
cargo build
```

Test cases are built in *debug mode* by default.

## Using GDB

Using `rust-gdb` you will get pretty printed values for more types than directly with `gdb`.

The following example, starts `gdb`, sets a breakpoint, and starts our compiled `polymesh`:

```bash
$> rust-gdb ./target/debug/polymesh
GNU gdb (Ubuntu 8.2.91.20190405-0ubuntu3) 8.2.91.20190405-git
Copyright (C) 2019 Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.
Type "show copying" and "show warranty" for details.
This GDB was configured as "x86_64-linux-gnu".
Type "show configuration" for configuration details.
For bug reporting instructions, please see:
<http://www.gnu.org/software/gdb/bugs/>.
Find the GDB manual and other documentation resources online at:
    <http://www.gnu.org/software/gdb/documentation/>.

For help, type "help".
Type "apropos word" to search for commands related to "word"...
Reading symbols from ./target/debug/polymesh...

(gdb) b balances/src/lib.rs : 390
Breakpoint 1 at 0x2b792d0: balances/src/lib.rs:390. (2 locations)

(gdb) run --dev
Starting program: ./target/debug/polymesh --dev
[Thread debugging using libthread_db enabled]
Using host libthread_db library "/lib/x86_64-linux-gnu/libthread_db.so.1".
2020-02-26 12:48:37 Running in --dev mode, RPC CORS has been disabled.
2020-02-26 12:48:37 Polymesh Node
...
```

# License

[LICENSE](https://github.com/PolymeshAssociation/Polymesh/blob/master/LICENSE)

# Substrate Framework

Polymesh is built using the [Substrate Framework](https://www.parity.io/what-is-substrate/).

# Polymath

[Polymath Website](https://polymath.network)
