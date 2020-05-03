// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {

  const api = await reqImports.createApi();

  const testEntities = await reqImports.initMain(api);

  let keys = await reqImports.generateKeys(api,5, "master");

  await reqImports.createIdentities(api, keys, testEntities[0]);
  
  await distributePoly( api, keys, reqImports.transfer_amount, testEntities[0] );
 
  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

// Sends transfer_amount to accounts[] from alice
async function distributePoly( api, accounts, transfer_amount, signingEntity ) {

  // Perform the transfers
  for (let i = 0; i < accounts.length; i++) {
   
    let nonceObj = {nonce: reqImports.nonces.get(signingEntity.address)};
    //console.log(accounts[i].address, transfer_amount.toString());
    const transaction = api.tx.balances.transfer(accounts[i].address, transfer_amount);
    const result = await reqImports.sendTransaction(transaction, signingEntity, nonceObj);  
    const passed = result.findRecord('system', 'ExtrinsicSuccess');
    if (passed) reqImports.fail_count--;

    reqImports.nonces.set( signingEntity.address, reqImports.nonces.get(signingEntity.address).addn(1));

  }
}

main().catch(console.error);
