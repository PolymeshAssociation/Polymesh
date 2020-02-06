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
  addSigningKeys,
  authorizeJoinToIdentities
} = require("../util/init.js");

let {
  STORAGE_DIR,
  nonces,
  synced_block,
  synced_block_ts,
  current_storage_size,
  transfer_amount,
  n_accounts,
  sk_roles,
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
    'Submit  : MAKE CLAIMS                    ': n_accounts,
    'Complete: MAKE CLAIMS                    ': n_accounts
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

  await authorizeJoinToIdentities( api, master_keys, issuer_dids, signing_keys, true);

  await blockTillPoolEmpty(api);

  let claim_issuer_dids = await createIdentities(api, claim_keys, true);

  await addClaimsToDids(api, claim_keys, issuer_dids, claim_issuer_dids, init_bars[0], init_bars[1]);

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

// Adds claim to DID  origin,
            // did: IdentityId,
            // claim_key: Vec<u8>,
            // did_issuer: IdentityId,
            // expiry: <T as pallet_timestamp::Trait>::Moment,
            // claim_value: ClaimValue
async function addClaimsToDids(api, accounts, dids, claim_dids, submitBar, completeBar) {
    //accounts should have the same length as claim_dids
    fail_type["MAKE CLAIMS"] = 0;
    for (let i = 0; i < dids.length; i++) {
      
      let claim_value = {data_type: 0, value: "0"};
  
        const unsub = await api.tx.identity
        .addClaim(dids[i], 0, claim_dids[i%claim_dids.length], 0, claim_value)
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
    
      nonces.set(accounts[i%claim_dids.length].address, nonces.get(accounts[i%claim_dids.length].address).addn(1));
      submitBar.increment();
    }
  }

main().catch(console.error);
