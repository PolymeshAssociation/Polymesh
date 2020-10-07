// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {

  const api = await reqImports.createApi();

  const testEntities = await reqImports.initMain(api);

  let primary_dev_seed = await reqImports.generateRandomKey(api);

  let keys = await reqImports.generateKeys(api, 2, primary_dev_seed );

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
        let account_did = await reqImports.keyToIdentityIds(api, accounts[i].publicKey);
   
        if(account_did == 0) {

            let nonceObj = {nonce: reqImports.nonces.get(alice.address)};
            console.log( `>>>> [Register CDD Claim] acc: ${accounts[i].address}`);
            const transaction = api.tx.identity.cddRegisterDid(accounts[i].address, []);
            const result = await reqImports.sendTransaction(transaction, alice, nonceObj);
            const passed = result.findRecord('system', 'ExtrinsicSuccess');
            if (passed) reqImports.fail_count--;

            reqImports.nonces.set(alice.address, reqImports.nonces.get(alice.address).addn(1));
        }
        else {
          console.log('Identity Already Linked.');
      }
      }

      for (let i = 0; i < accounts.length; i++) {
        const d = await api.query.identity.keyToIdentityIds(accounts[i].publicKey);
        dids.push(d.toHuman());
        console.log( `>>>> [Get DID ] acc: ${accounts[i].address} did: ${dids[i]}` );
      }

      // Add CDD Claim with CDD_ID
      for (let i = 0; i < dids.length; i++) {

        const cdd_id_byte = (i+1).toString(16).padStart(2,'0');
        const claim = { CustomerDueDiligence: `0x00000000000000000000000000000000000000000000000000000000000000${cdd_id_byte}`};

        console.log( `>>>> [add CDD Claim] did: ${dids[i]}, claim: ${JSON.stringify( claim)}`);
        const transaction = api.tx.identity.addClaim(dids[i], claim, null);
        let tx = await reqImports.sendTx(alice, transaction);
        if(tx !== -1) reqImports.fail_count--;

      }
      await reqImports.blockTillPoolEmpty(api);

      return dids;

}

main().catch(console.error);
