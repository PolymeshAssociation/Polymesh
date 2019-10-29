const { ApiPromise, WsProvider } = require("@polkadot/api");
const { Keyring } = require("@polkadot/keyring");
const { stringToU8a, u8aToHex } = require("@polkadot/util");
const BN = require("bn.js");
const cli = require("command-line-args");
const cliProg = require("cli-progress");
const childProc = require("child_process");

const fs = require("fs");
const path = require("path");

const BOB = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";

// Helps track the size delta for
let current_storage_size = 0;
// Updated by the CLI option
let STORAGE_DIR;
let nonces = new Map();
let alice, bob, dave;

let success_count = 0;
let fail_count = 0;

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
    `Welcome to Polymesh Stats Collector. Performing ${n_txes} txes per stat,` +
      `${n_claims} claims per DID.`
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

  let accounts = [];

  // Create `n_txes` accounts
  for (let i = 0; i < n_txes; i++) {
    accounts.push(
      keyring.addFromUri("//" + prepend + i.toString(), { name: i.toString() })
    );
    let accountRawNonce = await api.query.system.accountNonce(accounts[i].address);
    let account_nonce = new BN(accountRawNonce.toString());
    nonces.set(accounts[i].address, account_nonce);  
  }

  let transfer_amount = 5000000000;
  printAndUpdateStorageSize(STORAGE_DIR);

  // Execute each stats collection stage

  // Base currency TPS test
  console.log("=== TPS ===");
  await tps(api, keyring, n_txes); // base currency transfer sanity-check

  console.log("=== DISTRIBUTE POLY === ");
  await distributePoly(api, keyring, accounts, transfer_amount);
  // printAndUpdateStorageSize(STORAGE_DIR, n_txes);
  // console.log("Waiting for initial POLY distribution to generated accounts");
  await blockTillPoolEmpty(api, n_txes);

  console.log("=== CREATE IDENTITIES ===");
  let dids = await createIdentities(api, accounts, prepend);
  // printAndUpdateStorageSize(STORAGE_DIR, n_txes);

  console.log("=== ISSUE ONE SECURITY TOKEN PER DID ===");
  await issueTokenPerDid(api, accounts, dids, prepend);
  // printAndUpdateStorageSize(STORAGE_DIR, n_txes);

  console.log("=== ADD CLAIM ISSUERS TO DIDS ===");
  await addClaimIssuersToDids(api, accounts, dids);
  // printAndUpdateStorageSize(STORAGE_DIR, n_txes);

  console.log("=== ADD CLAIMS TO DIDS ===");
  await addNClaimsToDids(api, accounts, dids, n_claims);
  // printAndUpdateStorageSize(STORAGE_DIR, n_txes * n_claims);

  await blockTillPoolEmpty(api, n_txes);

  printAndUpdateStorageSize(STORAGE_DIR);
  console.log(`Total storage size delta: ${current_storage_size - initial_storage_size}KB`);

  console.log("DONE");
  process.exit();
}

// Spams the network with `n_txes` transfer transactions in an attempt to measure base
// currency TPS.
async function tps(api, keyring, n_txes) {

  let sched_prog = new cliProg.SingleBar({}, cliProg.Presets.shades_classic);
  sched_prog.start(n_txes);
  // Send one half from Alice to Bob
  // console.time("Transactions sent to the node in");
  for (let j = 0; j < n_txes / 2; j++) {
    await api.tx.balances
      .transfer(bob.address, 10)
      .signAndSend(
        alice,
        { nonce: nonces.get(alice.address) },
        ({ events = [], status }) => {
          if (status.isFinalized) {
            console.group();
            // console.log(
            //   `Distribution Tx ${i} included at ${status.asFinalized}`
            // );
            transfer_ok = false;
            events.forEach(({ phase, event: { data, method, section } }) => {
              if (section == "balances" && method == "Transfer") {
                transfer_ok = true;
                // console.log(`New transfer to ${i}: Transfer ${data}`);
              }
            });

            if (!transfer_ok) {
              console.error(`transfer[TPS]: ${j} FAIL`);
            } else {
              console.log(`transfer[TPS]: ${j} SUCCESS`);
            }
            console.groupEnd();
          }
        }
      );
    nonces.set(alice.address, nonces.get(alice.address).addn(1));
    sched_prog.increment();
  }
  // console.log("Alice -> Bob scheduled");

  // Send the other half from Bob to Alice to leave balances unaltered
  for (let j = 0; j < n_txes / 2; j++) {
    const unsub = await api.tx.balances
      .transfer(alice.address, 10)
      .signAndSend(
        bob,
        { nonce: nonces.get(bob.address) },
        ({ events = [], status }) => {
          if (status.isFinalized) {
            console.group();
            // console.log(
            //   `Distribution Tx ${i} included at ${status.asFinalized}`
            // );
            transfer_ok = false;
            events.forEach(({ phase, event: { data, method, section } }) => {
              if (section == "balances" && method == "Transfer") {
                transfer_ok = true;
                // console.log(`New transfer to ${i}: Transfer ${data}`);
              }
            });

            if (!transfer_ok) {
              console.error(`transfer[TPS]: ${j} FAIL`);
            } else {
              console.log(`transfer[TPS]: ${j} SUCCESS`);
            }

            unsub();
            console.groupEnd();
          }
        }
      );
    nonces.set(bob.address, nonces.get(bob.address).addn(1));
    sched_prog.increment();

  }
  // console.log("Bob -> Alice scheduled");
  sched_prog.stop();

  // await blockTillPoolEmpty(api, n_txes);
}

// Sends transfer_amount to accounts[] from alice
async function distributePoly(api, keyring, accounts, transfer_amount) {
  let sched_prog = new cliProg.SingleBar({}, cliProg.Presets.shades_classic);

  // console.log("Scheduling");
  sched_prog.start(accounts.length);

  // Perform the transfers
  for (let i = 0; i < accounts.length; i++) {
    const unsub = await api.tx.balances
      .transfer(accounts[i].address, transfer_amount)
      .signAndSend(
        alice,
        { nonce: nonces.get(alice.address) },
        ({ events = [], status }) => {
          if (status.isFinalized) {
            console.group();
            // console.log(
            //   `Distribution Tx ${i} included at ${status.asFinalized}`
            // );
            transfer_ok = false;
            events.forEach(({ phase, event: { data, method, section } }) => {
              if (section == "balances" && method == "Transfer") {
                transfer_ok = true;
                // console.log(`New transfer to ${i}: Transfer ${data}`);
              }
            });

            if (!transfer_ok) {
              console.error(`transfer: ${i} FAIL`);
            } else {
              console.log(`transfer: ${i} SUCCESS`);
            }

            unsub();
            console.groupEnd();
          }
        }
      );
    nonces.set(alice.address, nonces.get(alice.address).addn(1));
    // console.log("Alice Nonce: " + nonces.get(alice.address));
    sched_prog.increment();
  }

  sched_prog.stop();

  // await blockTillPoolEmpty(api, accounts.length);
}

// Create a new DID for each of accounts[]
async function createIdentities(api, accounts, prepend) {
  let dids = [];

  let sched_prog = new cliProg.SingleBar({}, cliProg.Presets.shades_classic);

  // console.log("Scheduling");
  sched_prog.start(accounts.length);
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
            // console.log(method, section, data);
            if (section == "identity" && method == "NewDid") {
              new_did_ok = true;
              console.log(`registerDid: ${i} SUCCESS with DID: ${data}`);
            }
          });

          if (!new_did_ok) {
            console.error(`registerDid: ${i} FAIL`);
          }
          unsub();
        }
      });
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    sched_prog.increment();
  }

  sched_prog.stop();

  // await blockTillPoolEmpty(api, accounts.length);

  return dids;
}

async function issueTokenPerDid(api, accounts, dids, prepend) {
  let sched_prog = new cliProg.SingleBar({}, cliProg.Presets.shades_classic);
  sched_prog.start(dids.length);
  for (let i = 0; i < dids.length; i++) {
    // console.log(`Creating token for DID ${i} ${accounts[i].address}`);

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
              console.log(`createToken: ${i} SUCCESS with: ${data}`);
            }
          });

          if (!new_token_ok) {
            console.error(`createToken: ${i} FAIL`);
          }
          unsub();
        }
      });
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    sched_prog.increment();
  }
  sched_prog.stop();
  // await blockTillPoolEmpty(api, dids.length);
}

async function addClaimIssuersToDids(api, accounts, dids) {
  let sched_prog = new cliProg.SingleBar({}, cliProg.Presets.shades_classic);
  sched_prog.start(dids.length);
  for (let i = 0; i < dids.length; i++) {
    // console.log(`Adding claim issuer for DID ${i} ${accounts[i].address}`);

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
              console.log(
                `addClaimIssuersToDids: ${i} SUCCESS with: ${data}`
              );
            }
          });

          if (!new_issuer_ok) {
            console.error(`addClaimIssuersToDids: ${i} FAIL`);
          }
          unsub();
        }
      });
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    sched_prog.increment();

  }
  sched_prog.stop();

  // await blockTillPoolEmpty(api, dids.length);
}

async function addNClaimsToDids(api, accounts, dids, n_claims) {
  let sched_prog = new cliProg.SingleBar({}, cliProg.Presets.shades_classic);
  sched_prog.start(dids.length);
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
              console.log(`addClaim: ${i} SUCCESS with: ${data}`);
            }
          });

          if (!new_claim_ok) {
            console.error(`addClaim: ${i} FAIL`);
          }
          unsub();
        }
      });
    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    sched_prog.increment();

  }
  sched_prog.stop();

  // await blockTillPoolEmpty(api, dids.length);
}

async function blockTillPoolEmpty(api, expected_tx_count) {
  console.log("Processing Transactions");
  let prev_block_pending = 0;
  let done_something = false;
  const unsub = await api.rpc.chain.subscribeNewHeads(async header => {
    console.log("Block: " + header.number + " Mined with Hash: " + header.hash);
    return api.rpc.author.pendingExtrinsics(pool => {
      // console.log("Queued: " + pool.length);
      if (pool.length > 0) {
        done_something = true;
      }
      if (done_something && pool.length == 0) {
        unsub();
      }
    });
    // console.log("HEADER: " + JSON.stringify(header));
  });
  // let elapsed_total = 0; // How many seconds since first tx pool subscription
  // let elapsed_this_batch = 0; // How many seconds since last batch popped from the pool
  // let prev_pending_tx_count = 0; // How many txes on last loop run
  // let still_polling = true;
  // let tps_sum = 0;
  // while (still_polling) {
  //   await api.rpc.author.pendingExtrinsics(pool => {
  //     console.log("Pending Transactions: ", pool.length);
  //     elapsed_total++;
  //     elapsed_this_batch++;
  //     if (pool.length < prev_pending_tx_count) {
  //       let batch_len = prev_pending_tx_count - pool.length;
  //       console.log(
  //         `Current batch (${batch_len} txs) processed in ${elapsed_this_batch}s`
  //       );
  //       console.log("Current batch TPS:", batch_len / elapsed_this_batch);
  //       elapsed_this_batch = 0;
  //     }
  //     if (pool.length === 0) {
  //       unsub();
  //       still_polling = false;
  //     }
  //     prev_pending_tx_count = pool.length;
  //   });

  //   // Wait one second
  //   await new Promise(resolve => setTimeout(resolve, 1000));
  // }

  // console.log(`Tx pool cleared in ${elapsed_total}s`);
  // console.log(`TPS: ${expected_tx_count / elapsed_total}`);
}

// Use the `du` command to obtain recursive directory size
function duDirSize(dir) {
  let cmd = `du -s ${dir}`;
  let re = /(\d+)/;
  let output = childProc.execSync(cmd).toString();

  let results = output.match(re);
  return new Number(results[1]);
}

function printAndUpdateStorageSize(dir, n_txs) {
  n_txs = n_txs > 0 ? n_txs : 1;
  let new_storage_size = duDirSize(STORAGE_DIR);
  let storage_delta = new_storage_size - current_storage_size;

  // console.log(
  //   `Current storage size (${STORAGE_DIR}): ${new_storage_size /
  //     1024}MB (delta ${storage_delta}KB, ${storage_delta /
  //     n_txs}KB per tx)`
  // );
  current_storage_size = new_storage_size;
}

main().catch(console.error);
