// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
const assert = require('assert');
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {

  const api = await reqImports.createApi();

  const testEntities = await reqImports.initMain(api);

  let primary_keys = await reqImports.generateKeys(api, 2, "primary6");

  let issuer_dids = await reqImports.createIdentities(api, primary_keys, testEntities[0]);

  await reqImports.distributePolyBatch( api, primary_keys, reqImports.transfer_amount, testEntities[0] );

  await issueTokenPerDid(api, primary_keys, issuer_dids, "DEMOCA");

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function issueTokenPerDid(api, accounts, dids, prepend) {

  for (let i = 0; i < dids.length; i++) {
    const ticker = `token${prepend}${i}`.toUpperCase();
    assert( ticker.length <= 12, "Ticker cannot be longer than 12 characters");

    let nonceObj = {nonce: reqImports.nonces.get(accounts[i].address)};
    const transaction = api.tx.asset.createAsset(
      ticker, ticker, 1000000, true, 0, [], "abc"
    );
    const result = await reqImports.sendTransaction(transaction, accounts[i], nonceObj);
    const passed = result.findRecord('system', 'ExtrinsicSuccess');
    if (passed) reqImports.fail_count--;

    reqImports.nonces.set(accounts[i].address, reqImports.nonces.get(accounts[i].address).addn(1));
  }
}

main().catch(console.error);
