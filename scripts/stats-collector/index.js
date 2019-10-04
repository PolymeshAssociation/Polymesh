const { ApiPromise, WsProvider } = require("@polkadot/api");
const { Keyring } = require("@polkadot/keyring");
const { stringToU8a, u8aToHex } = require("@polkadot/util");
const BN = require("bn.js");
const cli = require("command-line-args");
const cliProg = require("cli-progress");
const getSize = require("get-folder-size");

const fs = require("fs");
const path = require("path");

const BYTES_IN_MB = 1024 * 1024;

const BOB = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";

// Helps track the size delta for
let current_storage_size = 0;
// Updated by the CLI option
let STORAGE_DIR;

const cli_opts = [
  {
    name: "order", // Order of magnitude, results in 10^order transactions/parties involved per stage
    alias: "o",
    type: Number,
    defaultValue: 3
  },
  { name: "tps", alias: "t", type: Boolean, defaultValue: false },
  {
    name: "claims", // How many claims to add to the 10^order DIDs
    alias: "c",
    type: Number,
    defaultValue: 1
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
  let n_txes = 10 ** opts.order;
  let n_claims = 10 ** opts.claims;
  STORAGE_DIR = opts.dir;

  console.log(
    `Welcome to Polymesh Stats Collector. Performing ${n_txes} txes per stat,` +
      `${n_claims} claims per DID.`
  );

  const filePath = path.join(
    __dirname + "/../../../polymesh/polymesh_substrate/substrateui_dev.json"
  );
  const customTypes = JSON.parse(fs.readFileSync(filePath, "utf8"));

  const ws_provider = new WsProvider("ws://127.0.0.1:9944");
  const api = await ApiPromise.create({
    types: customTypes,
    provider: ws_provider
  });
  const keyring = new Keyring({ type: "sr25519" });

  // Set initial storage size
  getSize(STORAGE_DIR, (err, size) => {
    if (err) {
      throw err;
    }

    current_storage_size = size;
  });

  console.log(`Initial storage size: ${current_storage_size}MB`);

  // Execute each stats collection stage

  // Optional base currency TPS test
  if (opts.tps) {
    console.log("=== TPS ===");
    await tps(api, keyring, n_txes); // base currency transfer sanity-check
  }

  console.log("=== DISTRIBUTE POLY === ");

  let alice = keyring.addFromUri("//Alice", { name: "Alice" });
  let id_module_owner = keyring.addFromUri("//Dave", { name: "Dave" });
  let accounts = [];

  // Create `n_txes` accounts
  for (let i = 0; i < n_txes; i++) {
    accounts.push(
      keyring.addFromUri("//" + i.toString(), { name: i.toString() })
    );
  }

  let transfer_amount = 5000;
  let alice_balance = api.query.balances.freeBalance(alice.address);

  // Check that Alice can afford distributing the funds
  if (transfer_amount * accounts.length > alice_balance) {
    console.error(
      "Alice has insufficient balance (need " +
        transfer_amount * accounts.length +
        ", got " +
        alice_balance +
        ")"
    );
    process.exit();
  }

  await distributePoly(api, keyring, alice, accounts, transfer_amount);

  console.log("=== CREATE IDENTITIES ===");
  let dids = await createIdentities(api, accounts);

  console.log("=== TURN NEW DIDS INTO ISSUERS ===");
  await makeDidsIssuers(api, dids, id_module_owner);

  console.log("=== ISSUE ONE SECURITY TOKEN PER DID ===");
  await issueTokenPerDid(api, accounts, dids);

  console.log("=== ADD CLAIM ISSUERS TO DIDS ===");
  await addClaimIssuersToDids(api, accounts, dids);

  console.log("=== ADD CLAIMS TO DIDS ===");
  await addNClaimsToDids(api, accounts, dids, n_claims);

  console.log("DONE");
  process.exit();
}

// Spams the network with `n_txes` transfer transactions in an attempt to measure base
// currency TPS.
async function tps(api, keyring, n_txes) {
  let alice = keyring.addFromUri("//Alice", { name: "Alice" });
  let bob = keyring.addFromUri("//Bob", { name: "Bob" });

  // Send one half from Alice to Bob
  console.time("Transactions sent to the node in");
  let aliceRawNonce = await api.query.system.accountNonce(alice.address);
  let aliceNonce = new BN(aliceRawNonce.toString());
  for (let j = 0; j < n_txes / 2; j++) {
    const txHash = await api.tx.balances
      .transfer(bob.address, 10)
      .signAndSend(
        alice,
        { nonce: aliceNonce },
        ({ events = [], status }) => {}
      );
    aliceNonce = aliceNonce.addn(1);
  }
  console.log("Alice -> Bob scheduled");

  // Send the other half from Bob to Alice to leave balances unaltered
  let bobRawNonce = await api.query.system.accountNonce(bob.address);
  let bobNonce = new BN(bobRawNonce.toString());
  for (let j = 0; j < n_txes / 2; j++) {
    const txHash = await api.tx.balances
      .transfer(alice.address, 10)
      .signAndSend(bob, { nonce: bobNonce });
    bobNonce = bobNonce.addn(1);
  }
  console.log("Bob -> Alice scheduled");

  await blockTillPoolEmpty(api, n_txes);
}

// Sends transfer_amount to accounts[] from alice
async function distributePoly(api, keyring, alice, accounts, transfer_amount) {
  let aliceRawNonce = await api.query.system.accountNonce(alice.address);
  let aliceNonce = new BN(aliceRawNonce.toString());

  let sched_prog = new cliProg.SingleBar({}, cliProg.Presets.shades_classic);

  console.log("Scheduling");
  sched_prog.start(accounts.length);

  // Perform the transfers
  for (let i = 0; i < accounts.length; i++) {
    const unsub = await api.tx.balances
      .transfer(accounts[i].address, transfer_amount)
      .signAndSend(
        alice,
        { nonce: aliceNonce.addn(accounts.length - i - 1) },
        ({ events = [], status }) => {
          if (status.isFinalized) {
            console.group();
            console.log(
              `Distribution Tx ${i} included at ${status.asFinalized}`
            );
            transfer_ok = false;
            events.forEach(({ phase, event: { data, method, section } }) => {
              if (section == "balances" && method == "Transfer") {
                transfer_ok = true;
                console.log(`New transfer to ${i}: Transfer ${data}`);
              }
            });

            if (!transfer_ok) {
              console.error(`Transfer event ${i} not received`);
            }

            unsub();
            console.groupEnd();
          }
        }
      );
    sched_prog.increment();
  }

  sched_prog.stop();

  await blockTillPoolEmpty(api, accounts.length);
}

// Create a new DID for each of accounts[]
async function createIdentities(api, accounts) {
  let dids = [];
  for (let i = 0; i < accounts.length; i++) {
    console.log(
      "Creating DID for account " + i + " (address " + accounts[i].address + ")"
    );

    const did = "did:poly:" + i;
    dids.push(did);

    const unsub = await api.tx.identity
      .registerDid(did, [])
      .signAndSend(accounts[i], ({ events = [], status }) => {
        if (status.isFinalized) {
          let new_did_ok = false;
          events.forEach(({ phase, event: { data, method, section } }) => {
            if (section == "identity" && method == "NewDid") {
              new_did_ok = true;
              console.log(`Account ${i} now has a DID: NewDid ${data}`);
            }
          });

          if (!new_did_ok) {
            console.error(`NewDid event ${i} not received.`);
          }
          unsub();
        }
      });
  }

  await blockTillPoolEmpty(api, accounts.length);

  return dids;
}

async function makeDidsIssuers(api, dids, id_module_owner) {
  let nonce = new BN(
    (await api.query.system.accountNonce(id_module_owner.address)).toString()
  );

  for (let i = 0; i < dids.length; i++) {
    console.log(`Making DID ${i} an issuer`);

    const unsub = await api.tx.identity
      .createIssuer(dids[i])
      .signAndSend(id_module_owner, { nonce }, ({ events = [], status }) => {
        if (status.isFinalized) {
          let new_issuer_ok = false;
          events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(`event ${section}: ${data}`);
            if (section == "identity" && method == "NewIssuer") {
              new_issuer_ok = true;
              console.log(`DID ${i} is now an issuer: NewIssuer ${data}`);
            }
          });

          if (!new_issuer_ok) {
            console.error(`DID ${i}: NewIssuer event not detected`);
          }
          unsub();
        }
      });
    nonce = nonce.addn(1);
  }

  await blockTillPoolEmpty(api, dids.length);
}

async function issueTokenPerDid(api, accounts, dids) {
  for (let i = 0; i < dids.length; i++) {
    console.log(`Creating token for DID ${i} ${accounts[i].address}`);

    const ticker = `token${i}`;

    const unsub = await api.tx.asset
      .issueToken(dids[i], ticker, ticker, 1000000, true)
      .signAndSend(accounts[i], ({ events = [], status }) => {
        if (status.isFinalized) {
          let new_token_ok = false;
          events.forEach(({ phase, event: { data, method, section } }) => {
            if (section == "asset" && method == "IssuedToken") {
              new_token_ok = true;
              console.log(`Successfully issued token for DID ${i}: ${data}`);
            }
          });

          if (!new_token_ok) {
            console.error(`Could not issue token for DID ${i}`);
          }
          unsub();
        }
      });
  }

  await blockTillPoolEmpty(api, dids.length);
}

async function issueTokenPerDid(api, accounts, dids) {
  for (let i = 0; i < dids.length; i++) {
    console.log(`Creating token for DID ${i} ${accounts[i].address}`);

    const ticker = `token${i}`;

    const unsub = await api.tx.asset
      .issueToken(dids[i], ticker, ticker, 1000000, true)
      .signAndSend(accounts[i], ({ events = [], status }) => {
        if (status.isFinalized) {
          let new_token_ok = false;
          events.forEach(({ phase, event: { data, method, section } }) => {
            if (section == "asset" && method == "IssuedToken") {
              new_token_ok = true;
              console.log(`Successfully issued token for DID ${i}: ${data}`);
            }
          });

          if (!new_token_ok) {
            console.error(`Could not issue token for DID ${i}`);
          }
          unsub();
        }
      });
  }

  await blockTillPoolEmpty(api, dids.length);
}

async function addClaimIssuersToDids(api, accounts, dids) {
  for (let i = 0; i < dids.length; i++) {
    console.log(`Adding claim issuer for DID ${i} ${accounts[i].address}`);

    const unsub = await api.tx.identity
      .addClaimIssuer(dids[i], dids[i])
      .signAndSend(accounts[i], ({ events = [], status }) => {
        if (status.isFinalized) {
          let new_issuer_ok = false;
          events.forEach(({ phase, event: { data, method, section } }) => {
            if (section == "identity" && method == "NewClaimIssuer") {
              new_issuer_ok = true;
              console.log(
                `Successfully added a claim issuer for DID ${i}: ${data}`
              );
            }
          });

          if (!new_issuer_ok) {
            console.error(`Could not add claim issuer for DID ${i}`);
          }
          unsub();
        }
      });
  }

  await blockTillPoolEmpty(api, dids.length);
}

async function addNClaimsToDids(api, accounts, dids, n_claims) {
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
      .signAndSend(accounts[i], ({ events = [], status }) => {
        if (status.isFinalized) {
          let new_claim_ok = false;
          events.forEach(({ phase, event: { data, method, section } }) => {
            if (section == "identity" && method == "NewClaims") {
              new_claim_ok = true;
              console.log(`Successfully added claims for DID ${i}: ${data}`);
            }
          });

          if (!new_claim_ok) {
            console.error(`Could not add claims for DID ${i}`);
          }
          unsub();
        }
      });
  }

  await blockTillPoolEmpty(api, dids.length);
}

async function blockTillPoolEmpty(api, expected_tx_count) {
  const unsub = await api.rpc.chain.subscribeNewHeads(header => {
    console.log("Block " + header.number + " Mined. Hash: " + header.hash);
  });
  let elapsed_total = 0; // How many seconds since first tx pool subscription
  let elapsed_this_batch = 0; // How many seconds since last batch popped from the pool
  let prev_pending_tx_count = 0; // How many txes on last loop run
  let still_polling = true;
  let tps_sum = 0;
  while (still_polling) {
    await api.rpc.author.pendingExtrinsics(pool => {
      elapsed_total++;
      elapsed_this_batch++;
      if (pool.length < prev_pending_tx_count) {
        let batch_len = prev_pending_tx_count - pool.length;
        console.log(
          `Current batch (${batch_len} txs) processed in ${elapsed_this_batch}s`
        );
        console.log("Current batch TPS:", batch_len / elapsed_this_batch);
        elapsed_this_batch = 0;
      }
      if (pool.length === 0) {
        unsub();
        still_polling = false;
      }
      prev_pending_tx_count = pool.length;
    });

    // Wait one second
    await new Promise(resolve => setTimeout(resolve, 1000));
  }

  console.log(`Tx pool cleared in ${elapsed_total}s`);
  console.log(`TPS: ${expected_tx_count / elapsed_total}`);
}

main().catch(console.error);
