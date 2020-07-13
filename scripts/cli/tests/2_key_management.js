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

  let master_keys = await reqImports.generateKeys(api, 2, "master2");

  let signing_keys = await reqImports.generateKeys(api, 2, "signing2");

  let issuer_dids = await reqImports.createIdentities(api, master_keys, alice);

  await reqImports.distributePolyBatch( api, master_keys, reqImports.transfer_amount, alice );

  await addSigningKeys( api, master_keys, issuer_dids, signing_keys );

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

// Attach a signing key to each DID
async function addSigningKeys( api, accounts, dids, signing_accounts ) {

  for (let i = 0; i < accounts.length; i++) {


    // 1. Add Signing Item to identity.
    let nonceObj = {nonce: reqImports.nonces.get(accounts[i].address)};
    const transaction = api.tx.identity.addAuthorization({Account: signing_accounts[i].publicKey}, {JoinIdentity: []}, null);
    const result = await reqImports.sendTransaction(transaction, accounts[i], nonceObj);
    const passed = result.findRecord('system', 'ExtrinsicSuccess');
    if (passed) reqImports.fail_count--;

    reqImports.nonces.set(accounts[i].address, reqImports.nonces.get(accounts[i].address).addn(1));
  }
}

// Creates a multiSig Key
async function createMultiSig( api, alice, dids, numOfSigners ) {

    let nonceObj = {nonce: reqImports.nonces.get(alice.address)};
    const transaction = api.tx.multiSig.createMultisig(dids, numOfSigners);
    const result = await reqImports.sendTransaction(transaction, alice, nonceObj);
    const passed = result.findRecord('system', 'ExtrinsicSuccess');
    if (passed) reqImports.fail_count--;

    if (!passed) console.log('multiSig Failed');
    reqImports.nonces.set(alice.address, reqImports.nonces.get(alice.address).addn(1));

}



main().catch(console.error);
