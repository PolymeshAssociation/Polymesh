// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {

  const api = await reqImports.createApi();

  const testEntities = await reqImports.initMain(api);

  let keys = await reqImports.generateKeys(api, 2, "primary1");

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
async function distributePoly( api, accounts, transfer_amount, secondaryEntity ) {

  // Perform the transfers
  for (let i = 0; i < accounts.length; i++) {

    const transaction = api.tx.balances.transfer(accounts[i].address, transfer_amount);
    let tx = await reqImports.sendTx(secondaryEntity, transaction);
    if(tx !== -1) reqImports.fail_count--;

  }
}

main().catch(console.error);
