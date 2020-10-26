// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {

  const api = await reqImports.createApi();

  const testEntities = await reqImports.initMain(api);

  let primary_keys = await reqImports.generateKeys(api, 2, "primary4");

  let secondary_keys = await reqImports.generateKeys(api, 2, "secondary4");

  let issuer_dids = await reqImports.createIdentities(api, primary_keys, testEntities[0]);

  await reqImports.distributePolyBatch( api, primary_keys, reqImports.transfer_amount, testEntities[0] );

  await reqImports.addSecondaryKeys( api, primary_keys, issuer_dids, secondary_keys );

  await reqImports.authorizeJoinToIdentities( api, primary_keys, issuer_dids, secondary_keys);

  await setPermissionToSigner(api, primary_keys, secondary_keys);

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function setPermissionToSigner(api, accounts, secondary_accounts) {
  for (let i = 0; i < accounts.length; i++) {
    let signer = { Account: secondary_accounts[i].publicKey };
    let transaction = api.tx.identity.setPermissionToSigner(signer, reqImports.total_permissions);
    let tx = await reqImports.sendTx(accounts[i], transaction);
    if(tx !== -1) reqImports.fail_count--;
  }
}

main().catch(console.error);
