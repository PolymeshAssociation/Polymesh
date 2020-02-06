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
  blockTillPoolEmpty,
  distributePoly,
  addSigningKeys
} = require("../util/init.js");

let {
  STORAGE_DIR,
  nonces,
  synced_block,
  synced_block_ts,
  current_storage_size,
  transfer_amount,
  n_accounts,
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

  const testEntities = entities.slice(0,2);

  // Execute each stats collection stage
  const init_tasks = {
    'Submit  : AUTH JOIN TO IDENTITIES        ': n_accounts,
    'Complete: AUTH JOIN TO IDENTITIES        ': n_accounts
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
    true
  );

  await blockTillPoolEmpty(api);

  await createIdentities(api, master_keys, true);

  let issuer_dids = await createIdentities(api, master_keys, true);

  await addSigningKeys(
    api,
    master_keys,
    issuer_dids,
    signing_keys,
    true
  );

  await authorizeJoinToIdentities( api, master_keys, issuer_dids, signing_keys, init_bars[0], init_bars[1]);

  await blockTillPoolEmpty(api);

  await new Promise(resolve => setTimeout(resolve, 3000));
  init_multibar.stop();

  if (fail_count > 0) {
    console.log("Test Failed");
    process.exit();
  }
  
  console.log("DONE");

  process.exit();
}

// Authorizes the join of signing keys to a DID
async function authorizeJoinToIdentities(api, accounts, dids, signing_accounts, submitBar, completeBar) {
    fail_type["AUTH SIGNING KEY"] = 0;
    for (let i = 0; i < accounts.length; i++) {
      // 1. Authorize
      
          const unsub = await api.tx.identity
          .authorizeJoinToIdentity(dids[i])
          .signAndSend(signing_accounts[i],
            { nonce: nonces.get(signing_accounts[i].address) },
            ({ events = [], status }) => {
            if (status.isFinalized) {
              let tx_ok = false;
              events.forEach(({ phase, event: { data, method, section } }) => {
                if (section == "identity" && method == "SignerJoinedToIdentityApproved") {
                  tx_ok = true;
                  completeBar.increment();
                }
              });
  
              if (!tx_ok) {
                fail_count++;
                completeBar.increment();
                fail_type["AUTH SIGNING KEY"]++;
              }
              unsub();
            }
          });
      
  
      submitBar.increment();
    }
  
    return dids;
  }

main().catch(console.error);
