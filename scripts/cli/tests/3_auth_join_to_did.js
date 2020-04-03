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

  await reqImports.distributePoly( api, master_keys.concat(signing_keys), reqImports.transfer_amount, testEntities[0] );

  await reqImports.blockTillPoolEmpty(api);

  let issuer_dids = await reqImports.createIdentities(api, master_keys, testEntities[0]);

  await reqImports.addSigningKeys( api, master_keys, issuer_dids, signing_keys );

  await authorizeJoinToIdentities( api, master_keys, issuer_dids, signing_keys);

  await reqImports.blockTillPoolEmpty(api);

  await new Promise(resolve => setTimeout(resolve, 7000));

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

// Authorizes the join of signing keys to a DID
async function authorizeJoinToIdentities(api, accounts, dids, signing_accounts) {

  for (let i = 0; i < accounts.length; i++) {
    // 1. Authorize
    const auths = await api.query.identity.authorizations.entries({AccountKey: signing_accounts[i].publicKey});
    let last_auth_id = 0;
    for (let i = 0; i < auths.length; i++) {
      if (auths[i][1].auth_id.toNumber() > last_auth_id) {
        last_auth_id = auths[i][1].auth_id.toNumber()
      }
    }
    const unsub = await api.tx.identity
    .joinIdentityAsKey(last_auth_id)
    .signAndSend(signing_accounts[i],
      { nonce: reqImports.nonces.get(signing_accounts[i].address) },
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

    });
  }

  return dids;
}

main().catch(console.error);
