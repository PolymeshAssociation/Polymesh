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
  
  await reqImports.createIdentities(api, testEntities, testEntities[0]);
  
  await distributePoly( api, keys, reqImports.transfer_amount, testEntities[0] );

  await reqImports.blockTillPoolEmpty(api);

  await reqImports.createIdentities(api, keys, testEntities[0]);

  await new Promise(resolve => setTimeout(resolve, 3000));
 
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
    const unsub = await api.tx.balances
      .transfer(accounts[i].address, transfer_amount)
      .signAndSend(
        signingEntity,
        { nonce: reqImports.nonces.get(signingEntity.address) },
        ({ events = [], status }) => {

          if (status.isFinalized) {
            // Loop through Vec<EventRecord> to display all events
            events.forEach(({ phase, event: { data, method, section } }) => {
              if ( section === "system" && method === "ExtrinsicSuccess" )  reqImports.fail_count--;
            });
            unsub();
          }
        }

      );

    reqImports.nonces.set( signingEntity.address, reqImports.nonces.get(signingEntity.address).addn(1));

  }
}

main().catch(console.error);
