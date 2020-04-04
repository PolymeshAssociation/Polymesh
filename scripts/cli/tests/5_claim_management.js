// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {
  
  const api = await reqImports.createApi();

  const testEntities = await reqImports.initMain(api);

  let master_keys = await reqImports.generateKeys(api,5, "master");

  let signing_keys = await reqImports.generateKeys(api, 5, "signing");

  let claim_keys = await reqImports.generateKeys(api, 5, "claim");

  let claim_issuer_dids = await reqImports.createIdentities(api, claim_keys, testEntities[0]);

  let issuer_dids = await reqImports.createIdentities(api, master_keys, testEntities[0]);

  await reqImports.distributePoly( api, master_keys.concat(signing_keys).concat(claim_keys), reqImports.transfer_amount, testEntities[0] );

  await reqImports.blockTillPoolEmpty(api);

  await reqImports.addSigningKeys( api, master_keys, issuer_dids, signing_keys );

  await reqImports.blockTillPoolEmpty(api);

  await reqImports.authorizeJoinToIdentities( api, master_keys, issuer_dids, signing_keys);

  await reqImports.blockTillPoolEmpty(api);

  await addClaimsToDids(api, claim_keys, issuer_dids, claim_issuer_dids);

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

// Adds claim to DID  origin,
async function addClaimsToDids(api, accounts, dids, claim_dids) {
    //accounts should have the same length as claim_dids
    for (let i = 0; i < dids.length; i++) {
        
        let nonceObj = {nonce: reqImports.nonces.get(accounts[i%claim_dids.length].address)};
        const transaction = await api.tx.identity.addClaim(dids[i], 0, 0);
        const result = await reqImports.sendTransaction(transaction, accounts[i%claim_dids.length], nonceObj);  
        const passed = result.findRecord('system', 'ExtrinsicSuccess');
        if (passed) reqImports.fail_count--;

      reqImports.nonces.set(accounts[i%claim_dids.length].address, reqImports.nonces.get(accounts[i%claim_dids.length].address).addn(1));
    }
  }

main().catch(console.error);
