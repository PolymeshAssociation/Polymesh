const { ApiPromise, WsProvider } = require("@polkadot/api");
const { Keyring } = require("@polkadot/keyring");
const { stringToU8a, u8aToHex } = require("@polkadot/util");
const BN = require("bn.js");
const cli = require("command-line-args");
const cliProg = require("cli-progress");
const childProc = require("child_process");
const colors = require('colors');

const fs = require("fs");
const path = require("path");

// Helps track the size delta for
let current_storage_size = 0;

// Updated by the CLI option
let STORAGE_DIR;
let nonces = new Map();
let alice, bob, dave;
let master_keys = [];
let signing_keys = [];

let fail_count = 0;
let fail_type = {};

const cli_opts = [
  {
    name: "ntxs", // Number of transactions/accounts to use per step
    alias: "n",
    type: Number,
    defaultValue: 1000
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
  }
];

async function main() {
  // Parse CLI args and compute tx count
  const opts = cli(cli_opts);
  let n_txes = opts.ntxs;
  let n_claims = opts.claims;
  let prepend = opts.prepend;
  STORAGE_DIR = opts.dir;

  console.log(
    `Welcome to Polymesh Stats Collector. Creating ${n_txes} accounts and DIDs, with ${n_claims} claims per DID.`
  );

  const filePath = path.join(
    __dirname + "/../../../polymesh/polymesh_schema.json"
  );
  const customTypes = JSON.parse(fs.readFileSync(filePath, "utf8"));

  // const ws_provider = new WsProvider("ws://78.47.58.121:9944/");
  const ws_provider = new WsProvider("ws://127.0.0.1:9944/");
  const api = await ApiPromise.create({
    types: customTypes,
    provider: ws_provider
  });
  const keyring = new Keyring({ type: "sr25519" });

  const initial_storage_size = duDirSize(STORAGE_DIR);
  console.log(
    `Initial storage size (${STORAGE_DIR}): ${initial_storage_size / 1024}MB`
  );
  current_storage_size = initial_storage_size;

  alice = keyring.addFromUri("//Alice", { name: "Alice" });
  let aliceRawNonce = await api.query.system.accountNonce(alice.address);
  let alice_nonce = new BN(aliceRawNonce.toString());
  nonces.set(alice.address, alice_nonce);

  bob = keyring.addFromUri("//Bob", { name: "Bob" });
  let bobRawNonce = await api.query.system.accountNonce(bob.address);
  let bob_nonce = new BN(bobRawNonce.toString());
  nonces.set(bob.address, bob_nonce);

  dave = keyring.addFromUri("//Dave", { name: "Dave" });
  let daveRawNonce = await api.query.system.accountNonce(dave.address);
  let dave_nonce = new BN(daveRawNonce.toString());
  nonces.set(dave.address, dave_nonce);

  // Create `n_txes` master key accounts
  for (let i = 0; i < n_txes; i++) {
    master_keys.push(
      keyring.addFromUri("//IssuerMK" + prepend + i.toString(), { name: i.toString() })
    );
    let accountRawNonce = await api.query.system.accountNonce(master_keys[i].address);
    let account_nonce = new BN(accountRawNonce.toString());
    nonces.set(master_keys[i].address, account_nonce);  
  }

  // Create `n_txes` signing key accounts
  for (let i = 0; i < n_txes; i++) {
    signing_keys.push(
      keyring.addFromUri("//IssuerSK" + prepend + i.toString(), { name: i.toString() })
    );
    let accountRawNonce = await api.query.system.accountNonce(signing_keys[i].address);
    let account_nonce = new BN(accountRawNonce.toString());
    nonces.set(signing_keys[i].address, account_nonce);  
  }

  let transfer_amount = 5000000000;
  updateStorageSize(STORAGE_DIR);

  // Execute each stats collection stage
  const init_tasks = {
    'Submit  : TPS                 ': n_txes,
    'Complete: TPS                 ': n_txes,
    'Submit  : DISTRIBUTE POLY     ': n_txes * 2,
    'Complete: DISTRIBUTE POLY     ': n_txes * 2,
    'Submit  : CREATE IDENTITIES   ': n_txes,
    'Complete: CREATE IDENTITIES   ': n_txes,
    'Submit  : ADD SIGNING KEYS    ': n_txes,
    'Complete: ADD SIGNING KEYS    ': n_txes,
    'Submit  : ISSUE SECURITY TOKEN': n_txes,
    'Complete: ISSUE SECURITY TOKEN': n_txes,
    'Submit  : ADD CLAIM ISSUERS   ': n_txes,
    'Complete: ADD CLAIM ISSUERS   ': n_txes,
    'Submit  : ADD CLAIMS          ': n_txes,
    'Complete: ADD CLAIMS          ': n_txes,
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

  await tps(api, keyring, n_txes, init_bars[0], init_bars[1]); // base currency transfer sanity-check
  await distributePoly(api, keyring, master_keys.concat(signing_keys), transfer_amount, init_bars[2], init_bars[3]);
  await blockTillPoolEmpty(api, n_txes);
  let dids = await createIdentities(api, master_keys, prepend, init_bars[4], init_bars[5]);
  await addSigningKeys(api, master_keys, dids, signing_keys, prepend, init_bars[6], init_bars[7]);
  await issueTokenPerDid(api, master_keys, dids, prepend, init_bars[8], init_bars[9]);
  await addClaimIssuersToDids(api, master_keys, dids, init_bars[10], init_bars[11]);
  await addNClaimsToDids(api, master_keys, dids, n_claims, init_bars[12], init_bars[13]);
  await blockTillPoolEmpty(api, n_txes);

  updateStorageSize(STORAGE_DIR);

  init_multibar.stop();
  console.log('\n');
  console.log(`Total storage size delta: ${current_storage_size - initial_storage_size}KB`);
  console.log(`Total number of failures: ${fail_count}`)
  if (fail_count > 0) {
    for (let err in fail_type) {
      console.log(`\t` + err + ":" + fail_type[err]);
    }
  }
  console.log("DONE");
  process.exit();
}

// Spams the network with `n_txes` transfer transactions in an attempt to measure base
// currency TPS.
async function tps(api, keyring, n_txes, submitBar, completeBar) {
  fail_type["TPS"] = 0;
  // Send one half from Alice to Bob
  for (let j = 0; j < Math.floor(n_txes / 2); j++) {
    const unsub = await api.tx.balances
      .transfer(bob.address, 10)
      .signAndSend(
        alice,
        { nonce: nonces.get(alice.address) },
        ({ events = [], status }) => {
          if (status.isFinalized) {
            let transfer_ok = false;
            events.forEach(({ phase, event: { data, method, section } }) => {
              if (section == "balances" && method == "Transfer") {
                completeBar.increment();
                transfer_ok = true;
              }
            });

            if (!transfer_ok) {
              fail_count++;
              fail_type["TPS"]++;
              completeBar.increment();
            }

            unsub();
          }
        }
      );
    nonces.set(alice.address, nonces.get(alice.address).addn(1));
    submitBar.increment();
  }

  // Send the other half from Bob to Alice to leave balances unaltered
  for (let j = Math.floor(n_txes / 2); j < n_txes; j++) {
    const unsub = await api.tx.balances
      .transfer(alice.address, 10)
      .signAndSend(
        bob,
        { nonce: nonces.get(bob.address) },
        ({ events = [], status }) => {
          if (status.isFinalized) {
            let transfer_ok = false;
            events.forEach(({ phase, event: { data, method, section } }) => {
              if (section == "balances" && method == "Transfer") {
                completeBar.increment();
                transfer_ok = true;
              }
            });

            if (!transfer_ok) {
              fail_count++;
              completeBar.increment();
              fail_type["TPS"]++;
            }

            unsub();
          }
        }
      );
    nonces.set(bob.address, nonces.get(bob.address).addn(1));
    submitBar.increment();
  }

}

// Sends transfer_amount to accounts[] from alice
async function distributePoly(api, keyring, accounts, transfer_amount, submitBar, completeBar) {
  fail_type["DISTRIBUTE POLY"] = 0;

  // Perform the transfers
  for (let i = 0; i < accounts.length; i++) {
    const unsub = await api.tx.balances
      .transfer(accounts[i].address, transfer_amount)
      .signAndSend(
        alice,
        { nonce: nonces.get(alice.address) },
        ({ events = [], status }) => {
          if (status.isFinalized) {
            let transfer_ok = false;
            events.forEach(({ phase, event: { data, method, section } }) => {
              if (section == "balances" && method == "Transfer") {
                transfer_ok = true;
                completeBar.increment();
              }
            });

            if (!transfer_ok) {
              fail_count++;
              completeBar.increment();
              fail_type["DISTRIBUTE POLY"]++;
            }

            unsub();
          }
        }
      );
    nonces.set(alice.address, nonces.get(alice.address).addn(1));
    submitBar.increment();
  }

}

// Create a new DID for each of accounts[]
async function createIdentities(api, accounts, prepend, submitBar, completeBar) {
  let dids = [];
  fail_type["CREATE IDENTITIES"] = 0;
  for (let i = 0; i < accounts.length; i++) {
    const did = "did:poly:" + prepend + i;
    dids.push(did);

    const unsub = await api.tx.identity
      .registerDid(did, [])
      .signAndSend(accounts[i],
        { nonce: nonces.get(accounts[i].address) },
        ({ events = [], status }) => {
        if (status.isFinalized) {
          let new_did_ok = false;
          events.forEach(({ phase, event: { data, method, section } }) => {
            if (section == "identity" && method == "NewDid") {
              new_did_ok = true;
              completeBar.increment();
            }
          });

          if (!new_did_ok) {
            fail_count++;
            completeBar.increment();
            fail_type["CREATE IDENTITIES"]++;
          }
          unsub();
        }
      });
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    submitBar.increment();
  }

  return dids;
}

// Attach a signing key to each DID
async function addSigningKeys(api, accounts, dids, signing_accounts, prepend, submitBar, completeBar) {
  fail_type["ADD SIGNING KEY"] = 0;
  for (let i = 0; i < accounts.length; i++) {
    const unsub = await api.tx.identity
      .addSigningKeys(dids[i], [])
      .signAndSend(accounts[i],
        { nonce: nonces.get(accounts[i].address) },
        ({ events = [], status }) => {
        if (status.isFinalized) {
          let tx_ok = false;
          events.forEach(({ phase, event: { data, method, section } }) => {
            if (section == "identity" && method == "SigningKeysAdded") {
              tx_ok = true;
              completeBar.increment();
            }
          });

          if (!tx_ok) {
            fail_count++;
            completeBar.increment();
            fail_type["ADD SIGNING KEY"]++;
          }
          unsub();
        }
      });
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    submitBar.increment();
  }

  return dids;
}

// Attach a signing key to each DID
async function addSigningKeys(api, accounts, dids, signing_accounts, prepend, submitBar, completeBar) {
  fail_type["ADD SIGNING KEY"] = 0;
  for (let i = 0; i < accounts.length; i++) {
    const unsub = await api.tx.identity
      .addSigningKeys(dids[i], [])
      .signAndSend(accounts[i],
        { nonce: nonces.get(accounts[i].address) },
        ({ events = [], status }) => {
        if (status.isFinalized) {
          let tx_ok = false;
          events.forEach(({ phase, event: { data, method, section } }) => {
            if (section == "identity" && method == "SigningKeysAdded") {
              tx_ok = true;
              completeBar.increment();
            }
          });

          if (!tx_ok) {
            fail_count++;
            completeBar.increment();
            fail_type["ADD SIGNING KEY"]++;
          }
          unsub();
        }
      });
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    submitBar.increment();
  }

  return dids;
}

async function issueTokenPerDid(api, accounts, dids, prepend, submitBar, completeBar) {
  fail_type["ISSUE SECURITY TOKEN"] = 0;
  for (let i = 0; i < dids.length; i++) {

    const ticker = `token${prepend}${i}`;

    const unsub = await api.tx.asset
      .createToken(dids[i], ticker, ticker, 1000000, true)
      .signAndSend(accounts[i],
        { nonce: nonces.get(accounts[i].address) },        
        ({ events = [], status }) => {
        if (status.isFinalized) {
          let new_token_ok = false;
          events.forEach(({ phase, event: { data, method, section } }) => {
            if (section == "asset" && method == "IssuedToken") {
              new_token_ok = true;
              completeBar.increment();
            }
          });

          if (!new_token_ok) {
            fail_count++;
            completeBar.increment();
            fail_type["ISSUE SECURITY TOKEN"]++;
          }
          unsub();
        }
      });
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    submitBar.increment();
  }
}

async function addClaimIssuersToDids(api, accounts, dids, submitBar, completeBar) {
  fail_type["ADD CLAIM ISSUERS"] = 0;
  for (let i = 0; i < dids.length; i++) {

    const unsub = await api.tx.identity
      .addClaimIssuer(dids[i], dids[i])
      .signAndSend(accounts[i],
        { nonce: nonces.get(accounts[i].address) },
        ({ events = [], status }) => {
        if (status.isFinalized) {
          let new_issuer_ok = false;
          events.forEach(({ phase, event: { data, method, section } }) => {
            if (section == "identity" && method == "NewClaimIssuer") {
              new_issuer_ok = true;
              completeBar.increment();
            }
          });

          if (!new_issuer_ok) {
            fail_count++;
            completeBar.increment();
            fail_type["ADD CLAIM ISSUERS"]++;
          }
          unsub();
        }
      });
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    submitBar.increment();

  }
}

async function addNClaimsToDids(api, accounts, dids, n_claims, submitBar, completeBar) {
  fail_type["ADD CLAIMS"] = 0;
  for (let i = 0; i < dids.length; i++) {
    let claims = [];
    for (let j = 0; j < n_claims; j++) {
      claims.push({
        topic: 0,
        schema: 0,
        bytes: `claim${i}-${j}`
      });
    }

    const unsub = await api.tx.identity
      .addClaim(dids[i], dids[i], claims)
      .signAndSend(accounts[i],
        { nonce: nonces.get(accounts[i].address) },        
        ({ events = [], status }) => {
        if (status.isFinalized) {
          let new_claim_ok = false;
          events.forEach(({ phase, event: { data, method, section } }) => {
            if (section == "identity" && method == "NewClaims") {
              new_claim_ok = true;
              completeBar.increment();
            }
          });

          if (!new_claim_ok) {
            fail_count++;
            completeBar.increment();
            fail_type["ADD CLAIMS"]++;
          }
          unsub();
        }
      });
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    submitBar.increment();
  }  
}

async function blockTillPoolEmpty(api, expected_tx_count) {
  let prev_block_pending = 0;
  let done_something = false;
  let done = false;
  const unsub = await api.rpc.chain.subscribeNewHeads(async header => {
    let pool = await api.rpc.author.pendingExtrinsics();
    if (pool.length > 0) {
      done_something = true;
    }
    if (done_something && pool.length == 0) {
      unsub();
      done = true;
    }
  });
  while (!done) {
    await new Promise(resolve => setTimeout(resolve, 1000));
  }
  await new Promise(resolve => setTimeout(resolve, 3000));
  
}

// Use the `du` command to obtain recursive directory size
function duDirSize(dir) {
  let cmd = `du -s ${dir}`;
  let re = /(\d+)/;
  let output = childProc.execSync(cmd).toString();

  let results = output.match(re);
  return new Number(results[1]);
}

function updateStorageSize(dir, n_txs) {
  n_txs = n_txs > 0 ? n_txs : 1;
  let new_storage_size = duDirSize(STORAGE_DIR);
  let storage_delta = new_storage_size - current_storage_size;

  current_storage_size = new_storage_size;
}

main().catch(console.error);
