# Polymesh CLI

A small client-side Polymesh script to exercise major functionality in Polymesh.

Scripts to quickly run a local three node Polymesh testnet.

## Installation

```shell
$ yarn install #Project deps
```

## Usage

To run the three node local Polymesh testnet:

```shell
# Orchestrate the environment
$ ./run.sh 
# Viewing Substrate logs
$ ./node_modules/.bin/pm2 log pmesh-primary-node
$ ./node_modules/.bin/pm2 log pmesh-peer-node-1
$ ./node_modules/.bin/pm2 log pmesh-peer-node-2
```

To run the scripts and execute transactions:

```shell
$ npm test
```

The test scripts include those named below, and most make up `npm test`:

 - 0_create_identities
 - 1_poly_transfer
 - 2_key_management
 - 3_auth_join_did
 - 4_permission_management
 - 5_claim_management
 - 6_create_assets
 - 7_create_claim_rules
 - 8_asset_transfer
 - 9_offchain_worker_test
 - schema_test

 The scripts either give a result of Passed or Failed.

 To run a single script:
 ```shell
$ npm run -s 0-create-identities
```

## Output

### Normal Run

```
$ npm test
+ node ./util/schema_check.js args...
Passed
Passed
Passed
Passed
Passed
Passed
Passed
Passed
Passed
Passed

```

