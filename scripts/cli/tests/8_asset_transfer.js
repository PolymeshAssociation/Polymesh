// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {

  const api = await reqImports.createApi();
  const ticker = await reqImports.generateRandomTicker(api);
  const asset_did = reqImports.tickerToDid(ticker);
  let primary_dev_seed = await reqImports.generateRandomKey(api);

  const testEntities = await reqImports.initMain(api);

  let primary_keys = await reqImports.generateKeys( api, 3, primary_dev_seed );

  let issuer_dids = await reqImports.createIdentities( api, primary_keys, testEntities[0] );

  await reqImports.distributePolyBatch( api, primary_keys, reqImports.transfer_amount, testEntities[0] );

  await reqImports.issueTokenPerDid( api, primary_keys, ticker, 1000000, null);

  // receiverRules Claim
  await reqImports.addClaimsToDids( api, primary_keys, issuer_dids[2], "Exempted", { "Ticker": ticker }, null );

  // senderRules Claim
  await reqImports.addClaimsToDids( api, primary_keys, issuer_dids[1], "Exempted", { "Ticker": ticker }, null );

  // issuer Claim
  await reqImports.addClaimsToDids( api, primary_keys, issuer_dids[0], "Exempted", { "Ticker": ticker }, null );

  await reqImports.createClaimCompliance( api, primary_keys, issuer_dids, ticker );

  await mintingAsset( api, primary_keys[0], ticker );

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function mintingAsset(api, minter, ticker) {
  
  const transaction = await api.tx.asset.issue(ticker, 100);
  let tx = await reqImports.sendTx(minter, transaction);
  if(tx !== -1) reqImports.fail_count--;

}

main().catch(console.error);
