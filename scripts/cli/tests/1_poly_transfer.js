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

  let keys = await reqImports["generateKeys"](api,5, "master");

  await reqImports["createIdentities"](api, testEntities, testEntities[0]);

  await reqImports["createIdentities"](api, keys, testEntities[0]);

  await distributePoly( api, keys, reqImports["transfer_amount"], testEntities[0] );

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

// Sends transfer_amount to accounts[] from alice
async function distributePoly( api, accounts, transfer_amount, signingEntity ) {

  // Perform the transfers
  for (let i = 0; i < accounts.length; i++) {
    const unsub = await api.tx.balances
      .transfer(accounts[i].address, transfer_amount)
      .signAndSend(
        signingEntity,
        { nonce: reqImports["nonces"].get(signingEntity.address) },
        ({ events = [], status }) => {
          if (status.isFinalized) {
            reqImports["fail_count"] = reqImports["callback"](status, events, "balances", "Transfer", reqImports["fail_count"]);
            unsub();
          }
        }
      );

    reqImports["nonces"].set(
      signingEntity.address,
      reqImports["nonces"].get(signingEntity.address).addn(1)
    );
  }
}

main().catch(console.error);
