export const { ApiPromise, WsProvider } = require("@polkadot/api");
export const { Keyring } = require("@polkadot/keyring");
export const BN = require("bn.js");
export const cli = require("command-line-args");
export const cliProg = require("cli-progress");
const childProc = require("child_process");
export const colors = require("colors");

export const fs = require("fs");
export const path = require("path");

// Helps track the size delta for
export let current_storage_size = 0;

// Updated by the CLI option
let STORAGE_DIR;
export let nonces = new Map();
export let entities = [];
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

// Amount to seed each key with
export let transfer_amount = 10 * 10 ** 12;

// Parse CLI args and compute tx count
const opts = {
  accounts: 5,
  claim_accounts: 5,
  claims: 5,
  prepend: "demo",
  fast: false,
  dir: "/tmp/pmesh-primary-node"
};

// CLI args variables
let n_accounts = opts.accounts;
let n_claim_accounts = opts.claim_accounts;
let n_claims = opts.claims;
let prepend = opts.prepend;
let fast = opts.fast;

const keyring = new Keyring({ type: "sr25519" });
STORAGE_DIR = opts.dir;
const initial_storage_size = duDirSize(STORAGE_DIR);
current_storage_size = initial_storage_size;

// Initialization Main is used to generate all entities e.g (Alice, Bob, Dave)
// and Master keys, Signing keys, and Claim keys.
async function initMain(api) {
  console.log(
    `Welcome to Polymesh Stats Collector. Creating ${n_accounts} accounts and DIDs, with ${n_claims} claims per DID.`
  );

  console.log(
    `Initial storage size (${STORAGE_DIR}): ${initial_storage_size / 1024}MB`
  );

  generateEntity(api, "Alice");
  generateEntity(api, "Bob");
  generateEntity(api, "Dave");

  // Create `n_accounts` master key accounts
  console.log("Generating Master Keys");
  generateMasterKeys(api);

  // Create `n_accounts` signing key accounts
  console.log("Generating Signing Keys");
  generateSigningKeys(api);

  // Create `n_accounts` claim key accounts
  console.log("Generating Claim Keys");
  generateClaimKeys(api);

  updateStorageSize(STORAGE_DIR);
}

let generateEntity = async function(api, name) {
  let entity = keyring.addFromUri(`//${name}`, { name: `${name}` });
  entities[name] = entity;
  let entityRawNonce = await api.query.system.accountNonce(
    entities[name].address
  );
  let entity_nonce = new BN(entityRawNonce.toString());
  nonces.set(entities[name].address, entity_nonce);
};

const generateMasterKeys = async function(api) {
  for (let i = 0; i < n_accounts; i++) {
    master_keys.push(
      keyring.addFromUri("//IssuerMK" + prepend + i.toString(), {
        name: i.toString()
      })
    );
    let accountRawNonce = await api.query.system.accountNonce(
      master_keys[i].address
    );
    let account_nonce = new BN(accountRawNonce.toString());
    nonces.set(master_keys[i].address, account_nonce);
  }
};

const generateSigningKeys = async function(api) {
  for (let i = 0; i < n_accounts; i++) {
    signing_keys.push(
      keyring.addFromUri("//IssuerSK" + prepend + i.toString(), {
        name: i.toString()
      })
    );
    let accountRawNonce = await api.query.system.accountNonce(
      signing_keys[i].address
    );
    let account_nonce = new BN(accountRawNonce.toString());
    nonces.set(signing_keys[i].address, account_nonce);
  }
};

const generateClaimKeys = async function(api) {
  for (let i = 0; i < n_claim_accounts; i++) {
    claim_keys.push(
      keyring.addFromUri("//ClaimIssuerMK" + prepend + i.toString(), {
        name: i.toString()
      })
    );
    let claimIssuerRawNonce = await api.query.system.accountNonce(
      claim_keys[i].address
    );
    let account_nonce = new BN(claimIssuerRawNonce.toString());
    nonces.set(claim_keys[i].address, account_nonce);
  }
};

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
        let new_block_ts = parseInt(
          JSON.stringify(timestamp_extrinsic.raw["method"].args[0].raw)
        );
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
};

// Use the `du` command to obtain recursive directory size
function duDirSize(dir) {
  let cmd = `du -s ${dir}`;
  let re = /(\d+)/;
  let output = childProc.execSync(cmd).toString();
  let results = output.match(re);
  return new Number(results[1]);
}

// Updating storage size
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
        .signAndSend(accounts[i], { nonce: nonces.get(accounts[i].address) });
    }
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
  }
  await blockTillPoolEmpty(api);
  for (let i = 0; i < accounts.length; i++) {
    const d = await api.query.identity.keyToIdentityIds(accounts[i].publicKey);
    dids.push(d.raw.asUnique);
  }
  return dids;
};

export {
  duDirSize,
  updateStorageSize,
  blockTillPoolEmpty,
  createIdentities,
  initMain,
  n_claim_accounts,
  n_accounts,
  STORAGE_DIR,
  initial_storage_size,
  current_storage_size
};
