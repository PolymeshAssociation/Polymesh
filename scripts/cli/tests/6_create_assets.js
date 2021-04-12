// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
const assert = require('assert');
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {

  const api = await reqImports.createApi();

  let primary_dev_seed = await reqImports.generateRandomKey(api);

  const testEntities = await reqImports.initMain(api);

  let primary_keys = await reqImports.generateKeys(api, 2, primary_dev_seed);

  let issuer_dids = await reqImports.createIdentities(api, primary_keys, testEntities[0]);

  await reqImports.distributePolyBatch( api, primary_keys, reqImports.transfer_amount, testEntities[0] );

  await issueTokenPerDid(api, primary_keys, issuer_dids);

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function issueTokenPerDid(api, accounts, dids) {

  for (let i = 0; i < dids.length; i++) {
    const ticker = await reqImports.generateRandomTicker(api);
    assert( ticker.length <= 12, "Ticker cannot be longer than 12 characters");

    const transaction = api.tx.asset.createAssetAndMint(
      ticker, ticker, 1000000, true, 0, [], "abc"
    );
    let tx = await reqImports.sendTx(accounts[i], transaction);
    if(tx !== -1) reqImports.fail_count--;
   
  }
}

main().catch(console.error);
