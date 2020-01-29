const { ApiPromise, WsProvider } = require("@polkadot/api");
export { ApiPromise, WsProvider };
const {Keyring} = require("@polkadot/keyring");
export {Keyring};
export const BN = require("bn.js");
export const cli = require("command-line-args");
export const cliProg = require("cli-progress");
const childProc = require("child_process");
export const colors = require('colors');

export const fs = require("fs");
export const path = require("path");


// Helps track the size delta for
export let current_storage_size = 0;

// Updated by the CLI option
export let STORAGE_DIR;
export let nonces = new Map();
export let alice, bob, dave;
export let master_keys = [];
export let signing_keys = [];
export let claim_keys = [];
export let sk_roles = [[0], [1], [2], [1, 2]];

export let fail_count = 0;
export let fail_type = {};
export let block_sizes = {};
export let block_times = {};

export let synced_block = 0;
export let synced_block_ts = 0;


// const cli_opts = [
//   {
//     name: "accounts", // Number of transactions/accounts to use per step
//     alias: "n",
//     type: Number,
//     defaultValue: 30
//   },
//   {
//     name: "claim_accounts", // Number of transactions/accounts to use per step
//     alias: "t",
//     type: Number,
//     defaultValue: 5
//   },
//   {
//     name: "claims", // How many claims to add to the `ntxs` DIDs
//     alias: "c",
//     type: Number,
//     defaultValue: 10
//   },
//   {
//     name: "prepend", // Prepend for secretUrl for uniqueness
//     alias: "p",
//     type: String,
//     defaultValue: ""
//   },
//   {
//     name: "dir", // Substrate storage dir
//     alias: "d",
//     type: String,
//     defaultValue: "/tmp/pmesh-primary-node"
//   },
//   {
//     name: "fast", // Substrate storage dir
//     alias: "f",
//     type: Boolean,
//     defaultValue: false
//   }
// ];
//exports.PrintNearestStore = async function(api) {
const blockTillPoolEmpty = async function(api) {
  let prev_block_pending = 0;
  let done_something = false;
  let done = false;
  const unsub = await api.rpc.chain.subscribeNewHeads(async header => {
    let last_synced_block = synced_block;
    if (header.number > last_synced_block) {
      for (let i = last_synced_block + 1; i <= header.number; i++) {
        let block_hash = await api.rpc.chain.getBlockHash(i);
        let block = await api.rpc.chain.getBlock(block_hash);
        block_sizes[i] = block["block"]["extrinsics"].length;
        if (block_sizes[i] > 2) {
          done_something = true;
        }
        let timestamp_extrinsic = block["block"]["extrinsics"][0];
        let new_block_ts = parseInt(JSON.stringify(timestamp_extrinsic.raw["method"].args[0].raw));
        block_times[i] = new_block_ts - synced_block_ts;
        synced_block_ts = new_block_ts;
        synced_block = i;
      }
    }
    let pool = await api.rpc.author.pendingExtrinsics();
    if (done_something && pool.length == 0) {
      unsub();
      done = true;
    }
  });
  // Should use a mutex here...
  while (!done) {
    await new Promise(resolve => setTimeout(resolve, 1000));
  }
}

// Use the `du` command to obtain recursive directory size
function duDirSize(dir) {
  let cmd = `du -s ${dir}`;
  let re = /(\d+)/;
  let output = childProc.execSync(cmd).toString();
  let results = output.match(re);
  return new Number(results[1]);
}

function updateStorageSize(dir) {
  let new_storage_size = duDirSize(dir);
  current_storage_size = new_storage_size;
}

// Create a new DID for each of accounts[]
const createIdentities = async function(api, accounts, fast) {
  let dids = [];
  if (!("CREATE IDENTITIES" in fail_type)) {
    fail_type["CREATE IDENTITIES"] = 0;
  }
  for (let i = 0; i < accounts.length; i++) {
    if (fast) {
      await api.tx.identity
        .registerDid([])
        .signAndSend(accounts[i],
          { nonce: nonces.get(accounts[i].address) });
    } 
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
  }
  await blockTillPoolEmpty(api);
  for (let i = 0; i < accounts.length; i++) {
    const d = await api.query.identity.keyToIdentityIds(accounts[i].publicKey);
    dids.push(d.raw.asUnique);
  }
  return dids;
}

export { duDirSize, updateStorageSize, blockTillPoolEmpty, createIdentities};


