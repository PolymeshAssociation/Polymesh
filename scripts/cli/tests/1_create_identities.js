// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

const {
  path,
  fs,
  ApiPromise,
  WsProvider,
  initMain,
  colors,
  cliProg,
  updateStorageSize,
  blockTillPoolEmpty,
  distributePoly
} = require("../util/init.js");

let {
  STORAGE_DIR,
  nonces,
  n_accounts,
  master_keys,
  signing_keys,
  claim_keys,
  synced_block,
  transfer_amount,
  synced_block_ts,
  current_storage_size,
  fail_type,
  fail_count,
  block_sizes,
  block_times,
  entities,
  initial_storage_size,
  generateEntity
} = require("../util/init.js");

async function main() {
  // Schema path
  const filePath = path.join(__dirname + "/../../../polymesh_schema.json");
  const customTypes = JSON.parse(fs.readFileSync(filePath, "utf8"));

  // Start node instance
  const ws_provider = new WsProvider("ws://127.0.0.1:9944/");
  const api = await ApiPromise.create({
    types: customTypes,
    provider: ws_provider
  });

    generateEntity(api, "Alice");
    generateEntity(api, "Bob");
    generateEntity(api, "Dave");

  initMain(api);

  const testEntities = entities.slice(0,2);

  // Execute each stats collection stage
  const init_tasks = {
    "Submit  : CREATE ISSUER IDENTITIES       ": n_accounts + 2,
    "Complete: CREATE ISSUER IDENTITIES       ": n_accounts + 2
  };
  const init_bars = [];

  // create new container
  console.log("=== Processing Transactions ===");
  const init_multibar = new cliProg.MultiBar(
    {
      format: colors.cyan("{bar}") + " | {task} | {value}/{total}",
      hideCursor: true,
      barCompleteChar: "\u2588",
      barIncompleteChar: "\u2591",
      clearOnComplete: false,
      stopOnComplete: true
    },
    cliProg.Presets.shades_grey
  );

  for (let task in init_tasks) {
    const size = init_tasks[task];
    init_bars.push(init_multibar.create(size, 0, { task: task }));
  }

  // Get current block
  let current_header = await api.rpc.chain.getHeader();
  synced_block = parseInt(current_header.number);
  let current_block_hash = await api.rpc.chain.getBlockHash(synced_block);
  let current_block = await api.rpc.chain.getBlock(current_block_hash);
  let timestamp_extrinsic = current_block["block"]["extrinsics"][0];
  synced_block_ts = parseInt(
    JSON.stringify(timestamp_extrinsic.raw["method"].args[0].raw)
  );

  await createIdentities(api, testEntities, init_bars[0], init_bars[1]);

  await distributePoly(api, master_keys.concat(signing_keys).concat(claim_keys), transfer_amount, true);

  await blockTillPoolEmpty(api);

  await createIdentities(api, master_keys, init_bars[0], init_bars[1]);

  await new Promise(resolve => setTimeout(resolve, 3000));
  init_multibar.stop();

  if (fail_count > 0) {
    console.log("Test Failed");
    process.exit();
  }
  
  console.log("DONE");

  process.exit();
}

// Create a new DID for each of accounts[]
async function createIdentities(api, accounts, submitBar, completeBar) {
  let dids = [];
  if (!("CREATE IDENTITIES" in fail_type)) {
    fail_type["CREATE IDENTITIES"] = 0;
  }
  for (let i = 0; i < accounts.length; i++) {
    const unsub = await api.tx.identity
      .registerDid([])
      .signAndSend(
        accounts[i],
        { nonce: nonces.get(accounts[i].address) },
        ({ events = [], status }) => {
          if (status.isFinalized) {
            let new_did_ok = false;
            events.forEach(({ phase, event: { data, method, section } }) => {
              console.log(`Section: ${section} and Method: ${method}`);
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
        }
      );

    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
    submitBar.increment();
  }
  await blockTillPoolEmpty(api);
  for (let i = 0; i < accounts.length; i++) {
    const d = await api.query.identity.keyToIdentityIds(accounts[i].publicKey);
    dids.push(d.raw.asUnique);
  }
  return dids;
}

main().catch(console.error);
