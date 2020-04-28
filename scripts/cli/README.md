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

The scripts that make up `npm test` are:

 - 0-create-identities
 - 1-poly-transfer
 - 2-key-management
 - 3-auth-join-did
 - 4-permission-management
 - 5-claim-management
 - 6-create-assets
 - 7-create-claim-rules
 - 8-asset-transfer
 - schema-test

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

