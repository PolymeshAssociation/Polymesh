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
    defaultValue: 30
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

  const filePath = path.join(
    __dirname + "/../../../Polymesh/polymesh_schema.json"
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

  // Create `n_accounts` master key accounts

  console.log("Generating Master Keys");
  for (let i = 0; i < n_accounts; i++) {
    master_keys.push(
      keyring.addFromUri("//IssuerMK" + prepend + i.toString(), { name: i.toString() })
    );
    let accountRawNonce = await api.query.system.accountNonce(master_keys[i].address);
    let account_nonce = new BN(accountRawNonce.toString());
    nonces.set(master_keys[i].address, account_nonce);  
  }

  // Create `n_accounts` signing key accounts
  console.log("Generating Signing Keys");
  for (let i = 0; i < n_accounts; i++) {
    signing_keys.push(
      keyring.addFromUri("//IssuerSK" + prepend + i.toString(), { name: i.toString() })
    );
    let accountRawNonce = await api.query.system.accountNonce(signing_keys[i].address);
    let account_nonce = new BN(accountRawNonce.toString());
    nonces.set(signing_keys[i].address, account_nonce);  
  }

  // Create `n_accounts` claim key accounts
  console.log("Generating Claim Keys");
  for (let i = 0; i < n_claim_accounts; i++) {
    claim_keys.push(
      keyring.addFromUri("//ClaimIssuerMK" + prepend + i.toString(), { name: i.toString() })
    );
    let claimIssuerRawNonce = await api.query.system.accountNonce(claim_keys[i].address);
    let account_nonce = new BN(claimIssuerRawNonce.toString());
    nonces.set(claim_keys[i].address, account_nonce);  
  }
  // Amount to seed each key with
  let transfer_amount = 10 * 10**12;
  updateStorageSize(STORAGE_DIR);

  // Execute each stats collection stage
  const init_tasks = {
    'Submit  : TPS                            ': n_accounts,
    'Complete: TPS                            ': n_accounts,
    'Submit  : DISTRIBUTE POLY                ': n_claim_accounts + (n_accounts * 2),
    'Complete: DISTRIBUTE POLY                ': n_claim_accounts + (n_accounts * 2),
    'Submit  : CREATE ISSUER IDENTITIES       ': n_accounts,
    'Complete: CREATE ISSUER IDENTITIES       ': n_accounts,
    'Submit  : ADD SIGNING KEYS               ': n_accounts,
    'Complete: ADD SIGNING KEYS               ': n_accounts,
    'Submit  : SET SIGNING KEY ROLES          ': n_accounts,
    'Complete: SET SIGNING KEY ROLES          ': n_accounts,
    'Submit  : ISSUE SECURITY TOKEN           ': n_accounts,
    'Complete: ISSUE SECURITY TOKEN           ': n_accounts,
    'Submit  : CREATE CLAIM ISSUER IDENTITIES ': n_claim_accounts,
    'Complete: CREATE CLAIM ISSUER IDENTITIES ': n_claim_accounts,
    'Submit  : ADD CLAIM ISSUERS              ': n_accounts,
    'Complete: ADD CLAIM ISSUERS              ': n_accounts,
    'Submit  : MAKE CLAIMS                    ': n_accounts,
    'Complete: MAKE CLAIMS                    ': n_accounts,
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

  await tps(api, keyring, n_accounts, init_bars[0], init_bars[1], fast); // base currency transfer sanity-check
  await distributePoly(api, keyring, master_keys.concat(signing_keys).concat(claim_keys), transfer_amount, init_bars[2], init_bars[3], fast);
  // Need to wait until POLY has been distributed to pay for the next set of transactions
  await blockTillPoolEmpty(api, n_accounts);

  let issuer_dids = await createIdentities(api, master_keys, "issuer", prepend, init_bars[4], init_bars[5], fast);
  await addSigningKeys(api, master_keys, issuer_dids, signing_keys, init_bars[6], init_bars[7], fast);
  await addSigningKeyRoles(api, master_keys, issuer_dids, signing_keys, init_bars[8], init_bars[9], fast);
  await issueTokenPerDid(api, master_keys, issuer_dids, prepend, init_bars[10], init_bars[11], fast);
  let claim_issuer_dids = await createIdentities(api, claim_keys, "claim_issuer", prepend, init_bars[12], init_bars[13], fast);
  // Need to wait until identites have been created before we use them
  await blockTillPoolEmpty(api, n_accounts);

  await addClaimIssuersToDids(api, master_keys, issuer_dids, claim_issuer_dids, init_bars[14], init_bars[15], fast);
  // Need to wait until identites have been added as claim issuers
  await blockTillPoolEmpty(api, n_accounts);

  await addClaimsToDids(api, claim_keys, issuer_dids, claim_issuer_dids, n_claims, init_bars[16], init_bars[17], fast);
  // All transactions subitted, wait for queue to empty
  await blockTillPoolEmpty(api, n_accounts);
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
  process.exit();
}

// Spams the network with `n_accounts` transfer transactions in an attempt to measure base
// currency TPS.
async function tps(api, keyring, n_accounts, submitBar, completeBar, fast) {
  fail_type["TPS"] = 0;
  // Send one half from Alice to Bob
  for (let j = 0; j < Math.floor(n_accounts / 2); j++) {

    if (fast) {
      await api.tx.balances
      .transfer(bob.address, 10)
      .signAndSend(
        alice,
        { nonce: nonces.get(alice.address) });
    } else {
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
    }
    
    nonces.set(alice.address, nonces.get(alice.address).addn(1));
    submitBar.increment();
  }

  // Send the other half from Bob to Alice to leave balances unaltered
  for (let j = Math.floor(n_accounts / 2); j < n_accounts; j++) {
    if (fast) {
      const unsub = await api.tx.balances
      .transfer(alice.address, 10)
      .signAndSend(
        bob,
        { nonce: nonces.get(bob.address) });
    } else {
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
    }

    nonces.set(bob.address, nonces.get(bob.address).addn(1));
    submitBar.increment();
  }

}

// Sends transfer_amount to accounts[] from alice
async function distributePoly(api, keyring, accounts, transfer_amount, submitBar, completeBar, fast) {
  fail_type["DISTRIBUTE POLY"] = 0;

  // Perform the transfers
  for (let i = 0; i < accounts.length; i++) {
    if (fast) {
      const unsub = await api.tx.balances
      .transfer(accounts[i].address, transfer_amount)
      .signAndSend(
        alice,
        { nonce: nonces.get(alice.address) });
    } else {
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
    }

    nonces.set(alice.address, nonces.get(alice.address).addn(1));
    submitBar.increment();
  }

}

// Create a new DID for each of accounts[]
async function createIdentities(api, accounts, identity_type, prepend, submitBar, completeBar, fast) {
  let dids = [];
  if (!("CREATE IDENTITIES" in fail_type)) {
    fail_type["CREATE IDENTITIES"] = 0;
  }
  for (let i = 0; i < accounts.length; i++) {
    const did = "did:poly:" + identity_type + prepend + i;
    dids.push(did);
    if (fast) {
      await api.tx.identity
        .registerDid(did, [])
        .signAndSend(accounts[i],
          { nonce: nonces.get(accounts[i].address) });
    } else {
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
    }
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    submitBar.increment();
  }

  return dids;
}

// Attach a signing key to each DID
async function addSigningKeys(api, accounts, dids, signing_accounts, submitBar, completeBar, fast) {
  fail_type["ADD SIGNING KEY"] = 0;
  for (let i = 0; i < accounts.length; i++) {
    let signing_key = {
      key: signing_accounts[i].address,
      key_type: 0,
      roles: []
    }
    if (fast) {
      const unsub = await api.tx.identity
      .addSigningKeys(dids[i], [signing_key])
      .signAndSend(accounts[i],
        { nonce: nonces.get(accounts[i].address) });
    } else {    
      const unsub = await api.tx.identity
        .addSigningKeys(dids[i], [signing_key])
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
    }
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    submitBar.increment();
  }

  return dids;
}

// Attach a signing key to each DID
async function addSigningKeyRoles(api, accounts, dids, signing_accounts, submitBar, completeBar, fast) {
  fail_type["SET SIGNING KEY ROLES"] = 0;
  for (let i = 0; i < accounts.length; i++) {
    if (fast) {
      const unsub = await api.tx.identity
      .setRoleToSigningKey(dids[i], signing_accounts[i].address, sk_roles[i%sk_roles.length])
      .signAndSend(accounts[i],
        { nonce: nonces.get(accounts[i].address) });
    } else {
      const unsub = await api.tx.identity
      .setRoleToSigningKey(dids[i], signing_accounts[i].address, [])
      .signAndSend(accounts[i],
        { nonce: nonces.get(accounts[i].address) },
        ({ events = [], status }) => {
        if (status.isFinalized) {
          let tx_ok = false;
          events.forEach(({ phase, event: { data, method, section } }) => {
            //TODO: Capture event correctly
            // if (section == "identity" && method == "SigningKeyRolesUpdated") {
              tx_ok = true;
              completeBar.increment();
            // }
          });

          if (!tx_ok) {
            fail_count++;
            completeBar.increment();
            fail_type["SET SIGNING KEY ROLES"]++;
          }
          unsub();
        }
      });
    }

    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    submitBar.increment();
  }

  return dids;
}

async function issueTokenPerDid(api, accounts, dids, prepend, submitBar, completeBar, fast) {
  fail_type["ISSUE SECURITY TOKEN"] = 0;
  for (let i = 0; i < dids.length; i++) {

    const ticker = `token${prepend}${i}`;
    if (fast) {
      const unsub = await api.tx.asset
      .createToken(dids[i], ticker, ticker, 1000000, true)
      .signAndSend(accounts[i],
        { nonce: nonces.get(accounts[i].address) });
    } else {
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
    }
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    submitBar.increment();
  }
}

async function addClaimIssuersToDids(api, accounts, dids, claim_dids, submitBar, completeBar, fast) {
  fail_type["ADD CLAIM ISSUERS"] = 0;
  for (let i = 0; i < dids.length; i++) {
    if (fast) {
      const unsub = await api.tx.identity
      .addClaimIssuer(dids[i], claim_dids[i%claim_dids.length])
      .signAndSend(accounts[i],
        { nonce: nonces.get(accounts[i].address) });
    } else {
      const unsub = await api.tx.identity
      .addClaimIssuer(dids[i], claim_dids[i%claim_dids.length])
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
    }
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    submitBar.increment();

  }
}

async function addClaimsToDids(api, accounts, dids, claim_dids, n_claims, submitBar, completeBar, fast) {
  //accounts should have the same length as claim_dids
  fail_type["MAKE CLAIMS"] = 0;
  for (let i = 0; i < dids.length; i++) {
    let claims = [];
    for (let j = 0; j < n_claims; j++) {
      claims.push({
        topic: 0,
        schema: 0,
        bytes: `claim${i}-${j}`
      });
    }
    if (fast) {
      const unsub = await api.tx.identity
      .addClaim(dids[i], claim_dids[i%claim_dids.length], claims)
      .signAndSend(accounts[i%claim_dids.length],
        { nonce: nonces.get(accounts[i%claim_dids.length].address) });
    } else {

      const unsub = await api.tx.identity
      .addClaim(dids[i], claim_dids[i%claim_dids.length], claims)
      .signAndSend(accounts[i%claim_dids.length],
        { nonce: nonces.get(accounts[i%claim_dids.length].address) },        
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
            fail_type["MAKE CLAIMS"]++;
          }
          unsub();
        }
      });
    }
    nonces.set(accounts[i%claim_dids.length].address, nonces.get(accounts[i%claim_dids.length].address).addn(1));
    submitBar.increment();
  }  
}

async function blockTillPoolEmpty(api, expected_tx_count) {
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
