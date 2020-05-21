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

  let bob_signatory = await reqImports.signatory(api, bob, alice);
  let charlie_signatory = await reqImports.signatory(api, charlie, alice);
  let dave_signatory = await reqImports.signatory(api, dave, alice);
  
  let signatory_array = [bob_signatory, charlie_signatory, dave_signatory];

  await reqImports.distributePolyBatch( api, bob, reqImports.transfer_amount, alice );

  await reqImports.createMultiSig( api, alice, signatory_array, 1 );

  await bridgeTransfer(api, alice, bob);
 

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

// Proposing a Bridge Transaction
async function bridgeTransfer( api, signer, bob ) {

  let amount = 1_000_000_000_000_000_000_000;
  let bridge_tx = {
        nonce: 1,
        recipient: bob.publicKey,
        amount,
        tx_hash: reqImports.u8aToHex(1,256),
    }

    let nonceObjTwo = {nonce: reqImports.nonces.get(signer.address)};
    const transactionTwo = api.tx.bridge.proposeBridgeTx(bridge_tx);
    const resultTwo = await reqImports.sendTransaction(transactionTwo, signer, nonceObjTwo);  
    const passedTwo = resultTwo.findRecord('system', 'ExtrinsicSuccess');
    if (passedTwo) reqImports.fail_count--;

    reqImports.nonces.set(signer.address, reqImports.nonces.get(signer.address).addn(1));

}

main().catch(console.error);
