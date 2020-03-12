// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

async function main() {
  // Schema path
  const filePath = reqImports["path"].join(__dirname + "/../../../polymesh_schema.json");
  const customTypes = JSON.parse(reqImports["fs"].readFileSync(filePath, "utf8"));

  // Start node instance
  const ws_provider = new reqImports["WsProvider"]("ws://127.0.0.1:9944/");
  const api = await reqImports["ApiPromise"].create({
    types: customTypes,
    provider: ws_provider
  });

  const testEntities = await reqImports["initMain"](api);

  let master_keys = await reqImports["generateKeys"](api,5, "master");

  let signing_keys = await reqImports["generateKeys"](api, 5, "signing");

  await reqImports["createIdentities"](api, testEntities);

  await reqImports["distributePoly"]( api, master_keys.concat(signing_keys), reqImports["transfer_amount"], testEntities[0] );

  await reqImports["blockTillPoolEmpty"](api);

  let issuer_dids = await reqImports["createIdentities"](api, master_keys);

  await addSigningKeys( api, master_keys, issuer_dids, signing_keys );

  await reqImports["blockTillPoolEmpty"](api);

  await new Promise(resolve => setTimeout(resolve, 3000));

  if (reqImports["fail_count"] > 0) {
    console.log("Failed");
    process.exitCode = 1;
  } else {
    console.log("Passed");
  }

  process.exit();
}

// Attach a signing key to each DID
async function addSigningKeys( api, accounts, dids, signing_accounts ) {

  for (let i = 0; i < accounts.length; i++) {
    // 1. Add Signing Item to identity.
    const unsub = await api.tx.identity
      .addAuthorizationAsKey({AccountKey: signing_accounts[i].publicKey}, {JoinIdentity: dids[i]}, 0)
      .signAndSend(
        accounts[i],
        { nonce: reqImports["nonces"].get(accounts[i].address) },
        ({ events = [], status }) => {
          if (status.isFinalized) {
            reqImports["fail_count"] = reqImports["callback"](status, events, "identity", "NewAuthorization", reqImports["fail_count"]);
            unsub();
          }
        }
      );

    reqImports["nonces"].set(accounts[i].address, reqImports["nonces"].get(accounts[i].address).addn(1));
  }
}

main().catch(console.error);
