// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module/*, options*/);
module.exports = require("../util/init.js");

const { path, fs, ApiPromise, WsProvider, Keyring, createIdentities,
        BN, colors, duDirSize, cliProg, updateStorageSize, blockTillPoolEmpty} = require("../util/init.js");

let { STORAGE_DIR, nonces, synced_block, synced_block_ts, current_storage_size,
      master_keys, signing_keys, claim_keys, fail_type, fail_count, block_sizes, block_times } = require("../util/init.js");


async function main() {

  // Parse CLI args and compute tx count
  const opts = {
      "accounts": 5,
      "claim_accounts": 5,
      "claims": 5,
      "prepend": "demo",
      "fast": false,
      "dir": "/tmp/pmesh-primary-node"
  };
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
    __dirname + "/../../../polymesh_schema.json"
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
  let signing_key = signing_keys[0];


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
    'Submit  : DISTRIBUTE POLY                ': n_claim_accounts + (n_accounts * 2),
    'Complete: DISTRIBUTE POLY                ': n_claim_accounts + (n_accounts * 2),
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

  
    await createIdentities(api, [alice, bob], true);
    await distributePoly(api, keyring, master_keys.concat(signing_keys).concat(claim_keys), transfer_amount, init_bars[0], init_bars[1], fast);
    await blockTillPoolEmpty(api);

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
  // console.log(`Transactions processed:`);
  // for (let block_number in block_sizes) {
  //   console.log(`\tBlock Number: ` + block_number + "\t\tProcessed: " + block_sizes[block_number] + "\tTime (ms): " + block_times[block_number]);
  // }
  console.log("DONE");

  process.exit();
    
}



// Sends transfer_amount to accounts[] from alice
async function distributePoly(api, keyring, accounts, transfer_amount, submitBar, completeBar, fast) {
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


  main().catch(console.error);