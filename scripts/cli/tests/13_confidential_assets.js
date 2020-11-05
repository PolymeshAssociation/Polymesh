// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");
// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

const prepend = "EUR";
const prepend2 = "USD";


async function main() {
  const api = await reqImports.createApi();
  const ticker = `token${prepend}0`.toUpperCase();
  const ticker2 = `token${prepend2}0`.toUpperCase();
  const testEntities = await reqImports.initMain(api);

  let alice = testEntities[0];
  let bob = testEntities[1];
  let charlie = testEntities[2];


  let alice_did = await reqImports.keyToIdentityIds(api, alice.publicKey);

  let bob_did = await reqImports.createIdentities(api, [bob], alice);
  bob_did = bob_did[0];

  let charlie_did = await reqImports.createIdentities(api, [charlie], alice);
  charlie_did = charlie_did[0];

  await reqImports.distributePolyBatch(
    api,
    [bob, charlie],
    reqImports.transfer_amount,
    alice
  );

  // Alice creates Confidential Assets 
  await createConfidentialAsset(api, ticker, alice);
  await createConfidentialAsset(api, ticker2, alice);

  // Alice and Bob create their Mercat account locally and submit the proof to the chain

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function createConfidentialAsset(api, ticker, signer) {

    const transaction = await api.tx.confidentialAsset.createConfidentialAsset(
        ticker,
        ticker,
        true,
        0,
        [],
        null
      );
    
      let tx = await reqImports.sendTx(signer, transaction);
      if(tx !== -1) reqImports.fail_count--;
}

main().catch(console.error);