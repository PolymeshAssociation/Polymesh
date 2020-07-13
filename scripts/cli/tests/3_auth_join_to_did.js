// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {

  const api = await reqImports.createApi();

  const testEntities = await reqImports.initMain(api);

  let master_keys = await reqImports.generateKeys(api, 2, "master3");

  let signing_keys = await reqImports.generateKeys(api, 2, "signing3");

  let issuer_dids = await reqImports.createIdentities(api, master_keys, testEntities[0]);

  await reqImports.distributePolyBatch( api, master_keys, reqImports.transfer_amount, testEntities[0] );

  await reqImports.addSigningKeys( api, master_keys, issuer_dids, signing_keys );

  await authorizeJoinToIdentities( api, master_keys, issuer_dids, signing_keys);

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

// Authorizes the join of signing keys to a DID
async function authorizeJoinToIdentities(api, accounts, dids, signing_accounts) {
  for (let i = 0; i < accounts.length; i++) {
    // 1. Authorize
    const auths = await api.query.identity.authorizations.entries({Account: signing_accounts[i].publicKey});
    let last_auth_id = 0;
    for (let i = 0; i < auths.length; i++) {
      if (auths[i][1].auth_id.toNumber() > last_auth_id) {
        last_auth_id = auths[i][1].auth_id.toNumber()
      }
    }
    let nonceObj = {nonce: reqImports.nonces.get(signing_accounts[i].address)};
    const transaction = api.tx.identity.joinIdentityAsKey(last_auth_id);
    const result = await reqImports.sendTransaction(transaction, signing_accounts[i], nonceObj);
    const passed = result.findRecord('system', 'ExtrinsicSuccess');
    if (passed) reqImports.fail_count--;
  }

  return dids;
}

main().catch(console.error);
