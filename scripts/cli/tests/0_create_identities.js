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
  
  await createIdentities(api, testEntities, testEntities[0]);
 
  await reqImports.distributePoly( api, keys, reqImports.transfer_amount, testEntities[0] );
 
  await reqImports.blockTillPoolEmpty(api);
 
  await createIdentities(api, keys, testEntities[0]);
 
  await new Promise(resolve => setTimeout(resolve, 3000));
  
  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

// Create a new DID for each of accounts[]
async function createIdentities(api, accounts, alice) {

    let dids = [];
      for (let i = 0; i < accounts.length; i++) {
        const unsub = await api.tx.identity
          .registerDid([])
          .signAndSend(
            accounts[i],
            { nonce: reqImports.nonces.get(accounts[i].address) },
            ({ events = [], status }) => {

              if (status.isFinalized) {
                // Loop through Vec<EventRecord> to display all events
                events.forEach(({ phase, event: { data, method, section } }) => {
                  if ( section === "system" && method === "ExtrinsicSuccess" )  reqImports.fail_count--;
                  else if ( section === "system" && method === "ExtrinsicFailed" ) {
                    console.log(` ${phase}: ${section}.${method}:: ${data}`);
                  }
                });
                unsub();
              }

            }
          );

        reqImports.nonces.set(accounts[i].address, reqImports.nonces.get(accounts[i].address).addn(1));
      }
      await reqImports.blockTillPoolEmpty(api);
      for (let i = 0; i < accounts.length; i++) {
        const d = await api.query.identity.keyToIdentityIds(accounts[i].publicKey);
        dids.push(d.raw.asUnique);
      }
      let did_balance = 10 * 10**12;
      for (let i = 0; i < dids.length; i++) {
        await api.tx.balances
          .topUpIdentityBalance(dids[i], did_balance)
          .signAndSend(
            alice,
            { nonce: reqImports.nonces.get(alice.address) }
          );
        reqImports.nonces.set(
          alice.address,
          reqImports.nonces.get(alice.address).addn(1)
        );
      }
      return dids;

}

main().catch(console.error);
