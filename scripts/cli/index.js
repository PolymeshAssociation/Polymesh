// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("./util/init.js");

let { reqImports } = require("./util/init.js");

const cli = require("command-line-args");
const cliProg = require("cli-progress");
const childProc = require("child_process");
const colors = require('colors');
const assert = require('assert');

// Helps track the size delta for
let current_storage_size = 0;

// Updated by the CLI option
let STORAGE_DIR;
let nonces = new Map();
let alice, bob, dave;
let master_keys = [];
let signing_keys = [];
let claim_keys = [];
let sk_roles = [[0], [1], [2], [1, 2]];

let fail_count = 0;
let fail_type = {};
let block_sizes = {};
let block_times = {};

let synced_block = 0;
let synced_block_ts = 0;

const cli_opts = [
  {
    name: "accounts", // Number of transactions/accounts to use per step
    alias: "n",
    type: Number,
    defaultValue: 5
  },
  {
    name: "claim_accounts", // Number of transactions/accounts to use per step
    alias: "t",
    type: Number,
    defaultValue: 5
  },
  {
    name: "claims", // How many claims to add to the `ntxs` DIDs
    alias: "c",
    type: Number,
    defaultValue: 10
  },
  {
    name: "prepend", // Prepend for secretUrl for uniqueness
    alias: "p",
    type: String,
    defaultValue: ""
  },
  {
    name: "dir", // Substrate storage dir
    alias: "d",
    type: String,
    defaultValue: "/tmp/pmesh-primary-node"
  },
  {
    name: "fast", // Substrate storage dir
    alias: "f",
    type: Boolean,
    defaultValue: false
  }
];

async function main() {
  // Parse CLI args and compute tx count
  const opts = cli(cli_opts);
  let n_accounts = opts.accounts;
  let n_claim_accounts = opts.claim_accounts;
  let n_claims = opts.claims;
  let prepend = opts.prepend;
  let fast = opts.fast;

  STORAGE_DIR = opts.dir;

  console.log(
    `Welcome to Polymesh Stats Collector. Creating ${n_accounts} accounts and DIDs, with ${n_claims} claims per DID.`
  );

  const api = await reqImports.createApi();

  const testEntities = await reqImports.initMain(api);

  const initial_storage_size = duDirSize(STORAGE_DIR);
  console.log(
    `Initial storage size (${STORAGE_DIR}): ${initial_storage_size / 1024}MB`
  );
  current_storage_size = initial_storage_size;

  alice = testEntities[0];

  bob = testEntities[1];

  charlie = testEntities[2];

  dave = testEntities[3];

  // Create `n_accounts` master key accounts

  console.log("Generating Master Keys");
  master_keys = await reqImports.generateKeys(api, n_accounts, "master");

  // Create `n_accounts` signing key accounts
  console.log("Generating Signing Keys");
  signing_keys = await reqImports.generateKeys(api, n_accounts, "signing");


  // Create `n_accounts` claim key accounts
  console.log("Generating Claim Keys");
  claim_keys = await reqImports.generateKeys(api, n_accounts, "claim");

  // Amount to seed each key with
  let transfer_amount = 25000 * 10**6;
  updateStorageSize(STORAGE_DIR);

  // Execute each stats collection stage
  const init_tasks = {
    'Submit  : TPS                            ': n_accounts,
    'Complete: TPS                            ': n_accounts,
    'Submit  : DISTRIBUTE POLY                ': (n_accounts * 2),
    'Complete: DISTRIBUTE POLY                ': (n_accounts * 2),
    'Submit  : CREATE ISSUER IDENTITIES       ': n_accounts + 2,
    'Complete: CREATE ISSUER IDENTITIES       ': n_accounts + 2,
    'Submit  : ADD SIGNING KEYS               ': n_accounts,
    'Complete: ADD SIGNING KEYS               ': n_accounts,
    'Submit  : SET SIGNING KEY ROLES          ': n_accounts,
    'Complete: SET SIGNING KEY ROLES          ': n_accounts,
    'Submit  : ISSUE SECURITY TOKEN           ': n_accounts,
    'Complete: ISSUE SECURITY TOKEN           ': n_accounts,
    'Submit  : CREATE CLAIM ISSUER IDENTITIES ': n_claim_accounts,
    'Complete: CREATE CLAIM ISSUER IDENTITIES ': n_claim_accounts,
    'Submit  : MAKE CLAIMS                    ': n_accounts,
    'Complete: MAKE CLAIMS                    ': n_accounts,
    'Submit  : AUTH JOIN TO IDENTITIES        ': n_accounts,
    'Complete: AUTH JOIN TO IDENTITIES        ': n_accounts,
  };
  const init_bars = [];

  // create new container
  console.log("=== Processing Transactions ===");
  const init_multibar = new cliProg.MultiBar({
    format: colors.cyan('{bar}') + ' | {task} | {value}/{total}',
    hideCursor: true,
    barCompleteChar: '\u2588',
    barIncompleteChar: '\u2591',
    clearOnComplete: false,
    stopOnComplete: true
  }, cliProg.Presets.shades_grey);

  for (let task in init_tasks){
    const size = init_tasks[task];
    init_bars.push(init_multibar.create(size, 0, {task: task}));
  }

  // Get current block
  let current_header = await api.rpc.chain.getHeader();
  synced_block = parseInt(current_header.number);
  let current_block_hash = await api.rpc.chain.getBlockHash(synced_block);
  let current_block = await api.rpc.chain.getBlock(current_block_hash);
  let timestamp_extrinsic = current_block["block"]["extrinsics"][0];
  synced_block_ts = parseInt(JSON.stringify(timestamp_extrinsic.raw["method"].args[0].raw));

  await createIdentities(api, [charlie, dave], alice, init_bars[4], init_bars[5], fast);

  let issuer_dids = await createIdentities(api, master_keys, alice, init_bars[4], init_bars[5], fast);

  let claim_issuer_dids = await createIdentities(api, claim_keys, alice, init_bars[12], init_bars[13], fast);
                      
  await tps(api, [alice, bob], n_accounts, init_bars[0], init_bars[1], fast); // base currency transfer sanity-check

  await distributePoly(api, alice, master_keys.concat(claim_keys), transfer_amount, init_bars[2], init_bars[3], fast);

  await addSigningKeys(api, master_keys, issuer_dids, signing_keys, init_bars[6], init_bars[7], fast);
 
  await authorizeJoinToIdentities( api, master_keys, issuer_dids, signing_keys, init_bars[16], init_bars[17], fast);

  await addSigningKeyRoles(api, master_keys, issuer_dids, signing_keys, init_bars[8], init_bars[9], fast);
  
  await issueTokenPerDid(api, master_keys, issuer_dids, prepend, init_bars[10], init_bars[11], fast);

  await addClaimsToDids(api, claim_keys, issuer_dids, claim_issuer_dids, init_bars[14], init_bars[15], fast);

  await new Promise(resolve => setTimeout(resolve, 3000));
  init_multibar.stop();

  updateStorageSize(STORAGE_DIR);
  console.log(`Total storage size delta: ${current_storage_size - initial_storage_size}KB`);
  console.log(`Total number of failures: ${fail_count}`)
  if (fail_count > 0) {
    for (let err in fail_type) {
      console.log(`\t` + err + ":" + fail_type[err]);
    }
  }
  console.log(`Transactions processed:`);
  for (let block_number in block_sizes) {
    console.log(`\tBlock Number: ` + block_number + "\t\tProcessed: " + block_sizes[block_number] + "\tTime (ms): " + block_times[block_number]);
  }
  console.log("DONE");

  console.log("Claims Batch Test");

  await addClaimsBatchToDid(api, master_keys, issuer_dids, 10, fast);

  console.log("Claim Batch Test Completed");
  process.exit();
}

// Spams the network with `n_accounts` transfer transactions in an attempt to measure base
// currency TPS.
async function tps(api, accounts, n_accounts, submitBar, completeBar, fast) {
  fail_type["TPS"] = 0;

  let alice = accounts[0];
  let bob = accounts[1];
  // Send one half from Alice to Bob
  for (let j = 0; j < Math.floor(n_accounts / 2); j++) {

    if (fast) {
      await api.tx.balances
      .transfer(bob.address, 10)
      .signAndSend(
        alice,
        { nonce: reqImports.nonces.get(alice.address) });
    } else {

      let nonceObj = {nonce: reqImports.nonces.get(alice.address)};
      const transaction = api.tx.balances.transfer(bob.address, 10);
      const result = await reqImports.sendTransaction(transaction, alice, nonceObj);  
      const passed = result.findRecord('system', 'ExtrinsicSuccess');
      if (!passed) {
            fail_count++;
            completeBar.increment();
            fail_type["TPS"]++;
      } else {  completeBar.increment(); }

    }

    reqImports.nonces.set(alice.address, reqImports.nonces.get(alice.address).addn(1));
    submitBar.increment();
  }

  // Send the other half from Bob to Alice to leave balances unaltered
  for (let j = Math.floor(n_accounts / 2); j < n_accounts; j++) {
    if (fast) {
      const unsub = await api.tx.balances
      .transfer(alice.address, 10)
      .signAndSend(
        bob,
        { nonce: reqImports.nonces.get(bob.address) });
    } else {

      let nonceObjTwo = {nonce: reqImports.nonces.get(bob.address)};
      const transactionTwo = api.tx.balances.transfer(alice.address, 10);
      const resultTwo = await reqImports.sendTransaction(transactionTwo, bob, nonceObjTwo);  
      const passedTwo = resultTwo.findRecord('system', 'ExtrinsicSuccess');
      if (!passedTwo) {
            fail_count++;
            completeBar.increment();
            fail_type["TPS"]++;
      } else {  completeBar.increment(); }
    
    }

    reqImports.nonces.set(bob.address, reqImports.nonces.get(bob.address).addn(1));
    submitBar.increment();
  }

}

// Sends transfer_amount to accounts[] from alice
async function distributePoly(api, alice, accounts, transfer_amount, submitBar, completeBar, fast) {
  fail_type["DISTRIBUTE POLY"] = 0;

  // Perform the transfers
  for (let i = 0; i < accounts.length; i++) {
    if (fast) {
      let nonceObj = {nonce: reqImports.nonces.get(alice.address)};
      const transaction = api.tx.balances.transfer(accounts[i].address, transfer_amount);
      await reqImports.sendTransaction(transaction, alice, nonceObj);  
    } else {

      let nonceObj = {nonce: reqImports.nonces.get(alice.address)};
      const transaction = api.tx.balances.transfer(accounts[i].address, transfer_amount);
      const result = await reqImports.sendTransaction(transaction, alice, nonceObj);  
      const passed = result.findRecord('system', 'ExtrinsicSuccess');
      if (!passed) {
            fail_count++;
            completeBar.increment();
            fail_type["DISTRIBUTE POLY"]++;
      } else {  completeBar.increment(); }

    }

    reqImports.nonces.set(alice.address, reqImports.nonces.get(alice.address).addn(1));
    submitBar.increment();
  }

}

// Create a new DID for each of accounts[]
// precondition - accounts all have enough POLY
const createIdentities = async function(api, accounts, alice, submitBar, completeBar, fast) {
  return await createIdentitiesWithExpiry(api, accounts, alice, [], submitBar, completeBar, fast);
};

const createIdentitiesWithExpiry = async function(api, accounts, alice, expiries, submitBar, completeBar, fast) {
let dids = [];

if(!("CREATE IDENTITIES" in fail_type)) {
  fail_type["CREATE IDENTITIES"] = 0;
}

for (let i = 0; i < accounts.length; i++) {

    let expiry = expiries.length == 0 ? null : expiries[i];
    if(fast) {
    await api.tx.identity
      .cddRegisterDid(accounts[i].address, expiry, [])
      .signAndSend(alice, { nonce: reqImports.nonces.get(alice.address) });
    }
    else {
      let nonceObj = {nonce: reqImports.nonces.get(alice.address)};
        const transaction = api.tx.identity.cddRegisterDid(accounts[i].address, null, []);
        const result = await reqImports.sendTransaction(transaction, alice, nonceObj);  
        const passed = result.findRecord('system', 'ExtrinsicSuccess');
        if (!passed) {
              fail_count++;
              completeBar.increment();
              fail_type["CREATE IDENTITIES"]++;
        } else {  completeBar.increment(); }
    }
  reqImports.nonces.set(alice.address, reqImports.nonces.get(alice.address).addn(1));
  submitBar.increment();
}
await blockTillPoolEmpty(api);
for (let i = 0; i < accounts.length; i++) {
  const d = await api.query.identity.keyToIdentityIds(accounts[i].publicKey);
  dids.push(d.raw.asUnique);
}
let did_balance = 10 * 10**12;
for (let i = 0; i < dids.length; i++) {
  let nonceObjTwo = {nonce: nonces.get(alice.address)};
  const transactionTwo = api.tx.balances.topUpIdentityBalance(dids[i], did_balance);
  await reqImports.sendTransaction(transactionTwo, alice, nonceObjTwo);

  reqImports.nonces.set( alice.address, reqImports.nonces.get(alice.address).addn(1));
}
return dids;
}

// Attach a signing key to each DID
async function addSigningKeys(api, accounts, dids, signing_accounts, submitBar, completeBar, fast) {
  fail_type["ADD SIGNING KEY"] = 0;
  for (let i = 0; i < accounts.length; i++) {
    // 1. Add Signing Item to identity.
    if (fast) {
      let nonceObj = {nonce: reqImports.nonces.get(accounts[i].address)};
      const transaction = api.tx.identity.addAuthorizationAsKey({AccountKey: signing_accounts[i].publicKey}, {JoinIdentity: { target_did: dids[i], signing_item: null }}, null);
      await reqImports.sendTransaction(transaction, accounts[i], nonceObj);  
    } else {

      let nonceObj = {nonce: reqImports.nonces.get(accounts[i].address)};
      const transaction = api.tx.identity.addAuthorizationAsKey({AccountKey: signing_accounts[i].publicKey}, {JoinIdentity: { target_did: dids[i], signing_item: null }}, null);
      const result = await reqImports.sendTransaction(transaction, accounts[i], nonceObj);  
      const passed = result.findRecord('system', 'ExtrinsicSuccess');
      if (!passed) {
            fail_count++;
            completeBar.increment();
            fail_type["ADD SIGNING KEY"]++;
      } else {  completeBar.increment(); }

    }
    reqImports.nonces.set(accounts[i].address, reqImports.nonces.get(accounts[i].address).addn(1));
    submitBar.increment();
  }
}

async function authorizeJoinToIdentities(api, accounts, dids, signing_accounts, submitBar, completeBar, fast) {
  fail_type["AUTH SIGNING KEY"] = 0;
  for (let i = 0; i < accounts.length; i++) {
    // 1. Authorize
    const auths = await api.query.identity.authorizations.entries({AccountKey: signing_accounts[i].publicKey});
    let last_auth_id = 0;
    for (let i = 0; i < auths.length; i++) {
      if (auths[i][1].auth_id.toNumber() > last_auth_id) {
        last_auth_id = auths[i][1].auth_id.toNumber()
      }
    }

    if (fast) {
      let nonceObj = {nonce: reqImports.nonces.get(signing_accounts[i].address)};
      const transaction = api.tx.identity.joinIdentityAsKey(last_auth_id);
      await reqImports.sendTransaction(transaction, signing_accounts[i], nonceObj);  

      reqImports.nonces.set(signing_accounts[i].address, reqImports.nonces.get(signing_accounts[i].address).addn(1));
    } else {

      let nonceObj = {nonce: reqImports.nonces.get(signing_accounts[i].address)};
      const transaction = api.tx.identity.joinIdentityAsKey(last_auth_id);
      const result = await reqImports.sendTransaction(transaction, signing_accounts[i], nonceObj);  
      const passed = result.findRecord('system', 'ExtrinsicSuccess');

      if (!passed) {
        fail_count++;
        completeBar.increment();
        fail_type["AUTH SIGNING KEY"]++;
      } else {  completeBar.increment(); }

    }

    submitBar.increment();
  }

  return dids;
}

// Attach a signing key to each DID
async function addSigningKeyRoles(api, accounts, dids, signing_accounts, submitBar, completeBar, fast) {
  fail_type["SET SIGNING KEY ROLES"] = 0;
  for (let i = 0; i < accounts.length; i++) {
    let signer = { AccountKey: signing_accounts[i].publicKey };
    if (fast) {
      let nonceObj = {nonce: reqImports.nonces.get(accounts[i].address)};
      const transaction = api.tx.identity.setPermissionToSigner( signer, sk_roles[i%sk_roles.length]);
      await reqImports.sendTransaction(transaction, accounts[i], nonceObj);  
    } else {

      let nonceObj = {nonce: reqImports.nonces.get(accounts[i].address)};
      const transaction = api.tx.identity.setPermissionToSigner( signer, sk_roles[i%sk_roles.length]);
      const result = await reqImports.sendTransaction(transaction, accounts[i], nonceObj);  
      const passed = result.findRecord('system', 'ExtrinsicSuccess');

      if (!passed) {
        fail_count++;
        completeBar.increment();
        fail_type["SET SIGNING KEY ROLES"]++;
      } else {  completeBar.increment(); }

    }

    reqImports.nonces.set(accounts[i].address, reqImports.nonces.get(accounts[i].address).addn(1));
    submitBar.increment();
  }

  return dids;
}

async function issueTokenPerDid(api, accounts, dids, prepend, submitBar, completeBar, fast) {
  fail_type["ISSUE SECURITY TOKEN"] = 0;
  for (let i = 0; i < dids.length; i++) {

    const ticker = `token${prepend}${i}`.toUpperCase();
    assert( ticker.length <= 12, "Ticker cannot be longer than 12 characters");

    if (fast) {
      let nonceObj = {nonce: reqImports.nonces.get(accounts[i].address)};
      const transaction = api.tx.asset.createAsset(ticker, ticker, 1000000, true, 0, [], "abc");
      await reqImports.sendTransaction(transaction, accounts[i], nonceObj);  
    } else {

      let nonceObj = {nonce: reqImports.nonces.get(accounts[i].address)};
      const transaction = api.tx.asset.createAsset(ticker, ticker, 1000000, true, 0, [], "abc");
      const result = await reqImports.sendTransaction(transaction, accounts[i], nonceObj);  
      const passed = result.findRecord('system', 'ExtrinsicSuccess');

      if (!passed) {
        fail_count++;
        completeBar.increment();
        fail_type["ISSUE SECURITY TOKEN"]++;
      } else {  completeBar.increment(); }

    }
    reqImports.nonces.set(accounts[i].address, reqImports.nonces.get(accounts[i].address).addn(1));
    submitBar.increment();
  }
}

 // Takes in a number for the amount of claims to be produces
// then creates a batch of claims based on that number.
async function addClaimsBatchToDid(api, accounts, dids, n_claims, fast) {

  if (!fast) {
    // Holds the batch of claims
    let claims = [];

    const asset_did = reqImports.tickerToDid(reqImports.ticker);

    // Stores the value of each claim
    let claim_record = {target: dids[0], 
                        claim: { Exempted: asset_did },
                        expiry: null};
  
    // This fills the claims array with claim_values up to n_claims amount
    for (let i = 0; i < n_claims; i++) {
      claims.push( claim_record );
    }

    console.log("Claims length: " + claims.length);

    let nonceObj = {nonce: reqImports.nonces.get(accounts[1].address)};
    const transaction = api.tx.identity.addClaimsBatch(claims);
    await reqImports.sendTransaction(transaction, accounts[1], nonceObj);  
  }

}

async function addClaimsToDids(api, accounts, dids, claim_dids, submitBar, completeBar, fast) {
  //accounts should have the same length as claim_dids
  fail_type["MAKE CLAIMS"] = 0;
  for (let i = 0; i < dids.length; i++) {

    if (fast) {
      let nonceObj = {nonce: reqImports.nonces.get(accounts[i%claim_dids.length].address)};
      const transaction = api.tx.identity.addClaim(dids[i], 0, 0);
      await reqImports.sendTransaction(transaction, accounts[i], nonceObj);  
    } else {

      let nonceObj = {nonce: reqImports.nonces.get(accounts[i%claim_dids.length].address)};
      const transaction = api.tx.identity.addClaim(dids[i], 0, 0);
      const result = await reqImports.sendTransaction(transaction, accounts[i], nonceObj);  
      const passed = result.findRecord('system', 'ExtrinsicSuccess');

      if (!passed) {
        fail_count++;
        completeBar.increment();
        fail_type["MAKE CLAIMS"]++;
      } else {  completeBar.increment(); }

    }
    reqImports.nonces.set(accounts[i%claim_dids.length].address, reqImports.nonces.get(accounts[i%claim_dids.length].address).addn(1));
    submitBar.increment();
  }
}

async function blockTillPoolEmpty(api) {
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
  let new_storage_size = duDirSize(STORAGE_DIR);
  current_storage_size = new_storage_size;
}

main().catch(console.error);
