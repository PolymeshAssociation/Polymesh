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

  let primary_keys = await reqImports.generateKeys(api, 2, primary_dev_seed );

  let secondary_keys = await reqImports.generateKeys(api, 2, secondary_dev_seed );

  let issuer_dids = await reqImports.createIdentities(api, primary_keys, testEntities[0]);

  await reqImports.distributePolyBatch( api, primary_keys, reqImports.transfer_amount, testEntities[0] );

  await reqImports.addSecondaryKeys( api, primary_keys, issuer_dids, secondary_keys );

  await authorizeJoinToIdentities( api, primary_keys, issuer_dids, secondary_keys);

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

// Authorizes the join of secondary keys to a DID
async function authorizeJoinToIdentities(api, accounts, dids, secondary_accounts) {
  for (let i = 0; i < accounts.length; i++) {
    // 1. Authorize
    const auths = await api.query.identity.authorizations.entries({Account: secondary_accounts[i].publicKey});
    let last_auth_id = 0;
    for (let i = 0; i < auths.length; i++) {
      if (auths[i][1].auth_id.toNumber() > last_auth_id) {
        last_auth_id = auths[i][1].auth_id.toNumber()
      }
    }
    
    const transaction = api.tx.identity.joinIdentityAsKey(last_auth_id);
    let tx = await reqImports.sendTx(secondary_accounts[i], transaction);
    if(tx !== -1) reqImports.fail_count--;
  }

  return dids;
}

main().catch(console.error);
