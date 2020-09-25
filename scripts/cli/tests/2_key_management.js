// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {

  const api = await reqImports.createApi();

  const testEntities = await reqImports.initMain(api);

  let alice = testEntities[0];
  let bob = testEntities[1];
  let charlie = testEntities[2];
  let dave = testEntities[3];

  let primary_keys = await reqImports.generateKeys(api, 2, "primary2");

  let secondary_keys = await reqImports.generateKeys(api, 2, "secondary2");

  let issuer_dids = await reqImports.createIdentities(api, primary_keys, alice);

  await reqImports.distributePolyBatch( api, primary_keys, reqImports.transfer_amount, alice );

  await addSecondaryKeys( api, primary_keys, issuer_dids, secondary_keys );

  let bob_signatory = await reqImports.signatory(api, bob, alice);
  let charlie_signatory = await reqImports.signatory(api, charlie, alice);
  let dave_signatory = await reqImports.signatory(api, dave, alice);

  let signatory_array = [bob_signatory, charlie_signatory, dave_signatory];

  await createMultiSig( api, alice, signatory_array, 2 );

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

// Attach a secondary key to each DID
async function addSecondaryKeys( api, accounts, dids, secondary_accounts ) {

  for (let i = 0; i < accounts.length; i++) {

    // 1. Add Secondary Item to identity.
    const transaction = api.tx.identity.addAuthorization({Account: secondary_accounts[i].publicKey}, {JoinIdentity: reqImports.totalPermissions}, null);
    let tx = await reqImports.sendTx(accounts[i], transaction);
    if(tx !== -1) reqImports.fail_count--;

  }
}

// Creates a multiSig Key
async function createMultiSig( api, alice, dids, numOfSigners ) {

    const transaction = api.tx.multiSig.createMultisig(dids, numOfSigners);
    let tx = await reqImports.sendTx(alice, transaction);
    if(tx !== -1) reqImports.fail_count--;

}



main().catch(console.error);
