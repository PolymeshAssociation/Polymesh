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

  await createIdentities(api, keys, testEntities[0]);

 // await reqImports.distributePoly( api, keys, reqImports.transfer_amount, testEntities[0] );
  
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
        let nonceObj = {nonce: reqImports.nonces.get(alice.address)};
        const transaction = api.tx.identity.cddRegisterDid(accounts[i].address, null, []);
        const result = await reqImports.sendTransaction(transaction, alice, nonceObj);  
        const passed = result.findRecord('system', 'ExtrinsicSuccess');
        if (passed) reqImports.fail_count--;

        reqImports.nonces.set(alice.address, reqImports.nonces.get(alice.address).addn(1));
      }
      await reqImports.blockTillPoolEmpty(api);
      for (let i = 0; i < accounts.length; i++) {
        const d = await api.query.identity.keyToIdentityIds(accounts[i].publicKey);
        dids.push(d.raw.asUnique);
      }
      let did_balance = 10 * 10**12;
      for (let i = 0; i < dids.length; i++) {
        let nonceObjTwo = {nonce: reqImports.nonces.get(alice.address)};
        const transactionTwo = api.tx.balances.topUpIdentityBalance(dids[i], did_balance);
        const result = await reqImports.sendTransaction(transactionTwo, alice, nonceObjTwo);  
        const passed = result.findRecord('system', 'ExtrinsicSuccess');
        if (passed) reqImports.fail_count--;

        reqImports.nonces.set( alice.address, reqImports.nonces.get(alice.address).addn(1));
      }
      return dids;

}

main().catch(console.error);
