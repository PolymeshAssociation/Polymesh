// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

const {
  path,
  fs,
  ApiPromise,
  WsProvider,
  createIdentities,
  initMain,
  colors,
  cliProg,
  updateStorageSize,
  blockTillPoolEmpty
} = require("../util/init.js");

let {
  STORAGE_DIR,
  nonces,
  synced_block,
  synced_block_ts,
  current_storage_size,
  transfer_amount,
  n_accounts,
  n_claim_accounts,
  master_keys,
  signing_keys,
  claim_keys,
  fail_type,
  fail_count,
  block_sizes,
  block_times,
  entities,
  initial_storage_size
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

  initMain(api);

  const testEntities = entities.slice(0, 2);

  // Execute each stats collection stage
  const init_tasks = {
    "Submit  : DISTRIBUTE POLY                ":
      n_claim_accounts + n_accounts * 2,
    "Complete: DISTRIBUTE POLY                ":
      n_claim_accounts + n_accounts * 2
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
  
  await createIdentities(api, testEntities, true);

  await distributePoly(
    api,
    master_keys.concat(signing_keys).concat(claim_keys),
    transfer_amount,
    init_bars[0],
    init_bars[1]
  );

  await blockTillPoolEmpty(api);

  await createIdentities(api, master_keys, true);

  await new Promise(resolve => setTimeout(resolve, 3000));
  init_multibar.stop();

  if (fail_count > 0) {
    console.log("Test Failed");
    process.exit();
  }
  
  console.log("DONE");

  process.exit();
}

// Sends transfer_amount to accounts[] from alice
async function distributePoly(
  api,
  accounts,
  transfer_amount,
  submitBar,
  completeBar
) {
  fail_type["DISTRIBUTE POLY"] = 0;
  let signingEntity = entities[0];
  // Perform the transfers
  for (let i = 0; i < accounts.length; i++) {
    const unsub = await api.tx.balances
      .transfer(accounts[i].address, transfer_amount)
      .signAndSend(
        signingEntity,
        { nonce: nonces.get(signingEntity.address) },
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

    nonces.set(
      signingEntity.address,
      nonces.get(signingEntity.address).addn(1)
    );
    submitBar.increment();
  }
}

main().catch(console.error);
