// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {

  const api = await reqImports.createApi();

  let primary_dev_seed = await reqImports.generateRandomKey(api);
  
  let secondary_dev_seed = await reqImports.generateRandomKey(api);
  

  const testEntities = await reqImports.initMain(api);

  let primary_keys = await reqImports.generateKeys(api,2, primary_dev_seed );
  let claim_keys = await reqImports.generateKeys(api, 2, secondary_dev_seed );

  let claim_issuer_dids = await reqImports.createIdentities(api, claim_keys, testEntities[0]);

  let issuer_dids = await reqImports.createIdentities(api, primary_keys, testEntities[0]);

  await reqImports.distributePolyBatch( api, primary_keys.concat(claim_keys), reqImports.transfer_amount, testEntities[0] );

  await addClaimsToDids(api, claim_keys, issuer_dids, claim_issuer_dids);

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

// Adds claim to DID  origin,
async function addClaimsToDids(api, accounts, dids, claim_dids) {
    //accounts should have the same length as claim_dids
    for (let i = 0; i < dids.length; i++) {

        const transaction = await api.tx.identity.addClaim(dids[i], 0, 0);
        let tx = await reqImports.sendTx(accounts[i%claim_dids.length], transaction);
        if(tx !== -1) reqImports.fail_count--;

    }
  }

main().catch(console.error);
