// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {
  
  const api = await reqImports.createApi();

  const asset_did = reqImports.tickerToDid(reqImports.ticker);

  const testEntities = await reqImports.initMain(api);

  let master_keys = await reqImports.generateKeys( api, 3, "master" );

  let signing_keys = await reqImports.generateKeys( api, 3, "signing" );

  let issuer_dids = await reqImports.createIdentities( api, master_keys, testEntities[0] );

  await reqImports.distributePolyBatch( api, master_keys, reqImports.transfer_amount, testEntities[0] );

  await reqImports.addSigningKeys( api, master_keys, issuer_dids, signing_keys );

  await reqImports.authorizeJoinToIdentities( api, master_keys, issuer_dids, signing_keys );

  await reqImports.issueTokenPerDid( api, master_keys );

  // receiverRules Claim
  await reqImports.addClaimsToDids( api, master_keys, issuer_dids[2], "Exempted", asset_did, null );

  // senderRules Claim
  await reqImports.addClaimsToDids( api, master_keys, issuer_dids[0], "Exempted", asset_did, null );

  await reqImports.createClaimRules( api, master_keys, issuer_dids );

  await mintingAsset( api, master_keys, issuer_dids );

  await assetTransfer( api, master_keys, issuer_dids );

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function mintingAsset(api, accounts, dids) {

    
    let nonceObj = {nonce: reqImports.nonces.get(accounts[0].address)};
    const transaction = await api.tx.asset.issue(reqImports.ticker, dids[2], 100, "");
    const result = await reqImports.sendTransaction(transaction, accounts[0], nonceObj);  
    const passed = result.findRecord('system', 'ExtrinsicSuccess');
    if (passed) reqImports.fail_count--;

reqImports.nonces.set(accounts[0].address, reqImports.nonces.get(accounts[0].address).addn(1));
}

async function assetTransfer(api, accounts, dids) {

    let nonceObj = {nonce: reqImports.nonces.get(accounts[0].address)};
    const transaction = await api.tx.asset.transfer(reqImports.ticker, dids[2], 100);
    const result = await reqImports.sendTransaction(transaction, accounts[0], nonceObj);  
    const passed = result.findRecord('system', 'ExtrinsicSuccess');
    if (passed) reqImports.fail_count--;

    reqImports.nonces.set( accounts[0].address, reqImports.nonces.get(accounts[0].address).addn(1));
  
}

main().catch(console.error);
