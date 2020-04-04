// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {
 
  const api = await reqImports.createApi();

  const testEntities = await reqImports.initMain(api);

  let master_keys = await reqImports.generateKeys(api,5, "master");

  let signing_keys = await reqImports.generateKeys(api, 5, "signing");

  let issuer_dids = await reqImports["createIdentities"](api, master_keys, testEntities[0]);

  await reqImports["distributePoly"]( api, master_keys, reqImports["transfer_amount"], testEntities[0] );

  await reqImports["blockTillPoolEmpty"](api);

  await addSigningKeys( api, master_keys, issuer_dids, signing_keys );

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

// Attach a signing key to each DID
async function addSigningKeys( api, accounts, dids, signing_accounts ) {

  for (let i = 0; i < accounts.length; i++) {


    // 1. Add Signing Item to identity.
    let nonceObj = {nonce: reqImports.nonces.get(accounts[i].address)};
    const transaction = api.tx.identity.addAuthorizationAsKey({AccountKey: signing_accounts[i].publicKey}, {JoinIdentity: dids[i]}, null);
    const result = await reqImports.sendTransaction(transaction, accounts[i], nonceObj);  
    const passed = result.findRecord('system', 'ExtrinsicSuccess');
    if (passed) reqImports.fail_count--;

    reqImports.nonces.set(accounts[i].address, reqImports.nonces.get(accounts[i].address).addn(1));
  }
}

main().catch(console.error);
