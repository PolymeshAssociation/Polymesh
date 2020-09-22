// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

const prepend = "DEMOAT";

async function main() {

  const api = await reqImports.createApi();
  const ticker = `token${prepend}0`.toUpperCase();
  const asset_did = reqImports.tickerToDid(ticker);

  const testEntities = await reqImports.initMain(api);

  let primary_keys = await reqImports.generateKeys( api, 3, "primary8" );

  let issuer_dids = await reqImports.createIdentities( api, primary_keys, testEntities[0] );

  await reqImports.distributePolyBatch( api, primary_keys, reqImports.transfer_amount, testEntities[0] );

  await reqImports.issueTokenPerDid( api, primary_keys, prepend);

  // receiverRules Claim
  await reqImports.addClaimsToDids( api, primary_keys, issuer_dids[2], "Exempted", { "Ticker": ticker }, null );

  // senderRules Claim
  await reqImports.addClaimsToDids( api, primary_keys, issuer_dids[1], "Exempted", { "Ticker": ticker }, null );

  // issuer Claim
  await reqImports.addClaimsToDids( api, primary_keys, issuer_dids[0], "Exempted", { "Ticker": ticker }, null );

  await reqImports.createClaimCompliance( api, primary_keys, issuer_dids, prepend );

  await mintingAsset( api, primary_keys[0], prepend );

  // TODO: Use settlement module
  // await assetTransfer( api, primary_keys[0], issuer_dids[2], prepend );

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function mintingAsset(api, minter, prepend) {
  const ticker = `token${prepend}0`.toUpperCase();
  const transaction = await api.tx.asset.issue(ticker, 100);
  let tx = await reqImports.sendTx(minter, transaction);
  if(tx !== -1) reqImports.fail_count--;

}

// TODO: Use settlement module
// async function assetTransfer(api, from_account, did, prepend) {
//     const ticker = `token${prepend}0`.toUpperCase();
//     let nonceObj = {nonce: reqImports.nonces.get(from_account)};
//     const transaction = await api.tx.asset.transfer(ticker, did, 100);
//     const result = await reqImports.sendTransaction(transaction, from_account, nonceObj);
//     const passed = result.findRecord('system', 'ExtrinsicSuccess');
//     if (passed) reqImports.fail_count--;

//     reqImports.nonces.set( from_account.address, reqImports.nonces.get(from_account.address).addn(1));

// }

main().catch(console.error);
