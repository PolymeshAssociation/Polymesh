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
  const tickerHex = reqImports.stringToHex(`${prepend}0`); 
  const ticker2Hex = reqImports.stringToHex(`${prepend2}0`); 
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
  await createConfidentialAsset(api, tickerHex, alice);
  await createConfidentialAsset(api, ticker2Hex, alice);

  // Alice and Bob create their Mercat account locally and submit the proof to the chain
  const execSync = require('child_process').execSync;
  let output = execSync(
      `mercat-interactive create-user-account --user bob --db-dir chain_dir --ticker ${tickerHex.substr(2)} --valid-ticker-names ${tickerHex.substr(2)} ${ticker2Hex.substr(2)}`,
       );  


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