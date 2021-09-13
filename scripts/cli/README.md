# Polymesh CLI

A small client-side Polymesh script to exercise major functionality in Polymesh.

Scripts to quickly run a local three node Polymesh testnet.

## Installation

```shell
$ cargo build --release #Build lastest node
$ npm install #Project deps
```

## Usage

To run the three node local Polymesh testnet:

```shell
# Start the node
$ ./run.sh
# Viewing Substrate logs
$ ./node_modules/.bin/pm2 log pmesh-primary-node
$ ./node_modules/.bin/pm2 log pmesh-peer-node-1
$ ./node_modules/.bin/pm2 log pmesh-peer-node-2
```

To run the scripts and execute transactions:

1. Build latest types `npm run build:types`
2. Compile Typescript files `npm run build`
3. Run integration tests `npm run test`

To stop the node use `./stop.sh`:


The test scripts include those named below, and most make up `npm test`:


 - 0_create_identities
 - 1_poly_transfer
 - 2_key_management
 - 3_auth_join_did
 - 4_permission_management
 - 5_claim_management
 - 6_create_assets
 - 7_create_claim_compliance
 - 8_asset_minting
 - 9_bridge_transfer
 - 10_governance_management
 - 11_A_settlement
 - 11_B_settlement
 - 12_external_agents

 To run a single script:
 ```shell
$ npm run 0_create-_dentities
```
## Updating types:
1. Build latest version `cargo build --release`
2. Install dependencies with: `npm install`
3. Run a node: `./run.sh`
4. Build new type definitions: `npm run build:types`
5. Stop the node: `./stop.sh`