// Use asset.transfer(icker, to_did, value)
// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
const assert = require("assert");
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

async function main() {
  // Schema path
  const filePath = reqImports["path"].join(
    __dirname + "/../../../polymesh_schema.json"
  );
  const customTypes = JSON.parse(
    reqImports["fs"].readFileSync(filePath, "utf8")
  );

  // Start node instance
  const ws_provider = new reqImports["WsProvider"]("ws://127.0.0.1:9944/");
  const api = await reqImports["ApiPromise"].create({
    types: customTypes,
    provider: ws_provider
  });

  const testEntities = await reqImports["initMain"](api);

  let master_keys = await reqImports["generateKeys"]( api, 3, "master" );

  let signing_keys = await reqImports["generateKeys"]( api, 3, "signing" );

  await reqImports["createIdentities"]( api, testEntities );

  await reqImports["distributePoly"]( api, master_keys.concat(signing_keys), reqImports["transfer_amount"], testEntities[0] );

  await reqImports["blockTillPoolEmpty"](api);

  let issuer_dids = await reqImports["createIdentities"]( api, master_keys );

  //await reqImports["distributePoly"]( api, issuer_dids, reqImports["transfer_amount"], testEntities[0] );

  await reqImports["addSigningKeys"]( api, master_keys, issuer_dids, signing_keys );

  await reqImports["authorizeJoinToIdentities"]( api, master_keys, issuer_dids, signing_keys );

  await reqImports["blockTillPoolEmpty"](api);

  await reqImports["issueTokenPerDid"]( api, master_keys, issuer_dids, reqImports["prepend"] );

   await reqImports["blockTillPoolEmpty"](api);

  await reqImports["addClaimsToDids"](api, master_keys, issuer_dids);

  await reqImports["blockTillPoolEmpty"](api);

  await reqImports["createClaimRules"]( api, master_keys, issuer_dids );

  await reqImports["blockTillPoolEmpty"](api);

  await mintingAsset(api, master_keys, issuer_dids, reqImports["prepend"]);

  await reqImports["blockTillPoolEmpty"](api);

  await assetTransfer( api, master_keys, issuer_dids, reqImports["prepend"] );

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

async function mintingAsset(api, accounts, dids, prepend) {

const ticker = `token${prepend}0`.toUpperCase();
assert( ticker.length <= 12, "Ticker cannot be longer than 12 characters");

const unsub = await api.tx.asset
  .issue(ticker, dids[2], 100, "")
  .signAndSend(
    accounts[0],
    { nonce: reqImports["nonces"].get(accounts[0].address) },
    ({ events = [], status }) => {
      console.log(`events is ${events} and status is ${status}`);
      if (status.isFinalized) {
        console.log(`events is ${events} and status is ${status}`);
        reqImports["fail_count"] = reqImports["callback"](
          status,
          events,
          "",
          "",
          reqImports["fail_count"]
        );
        unsub();
      }
    }
  );

reqImports["nonces"].set(
  accounts[0].address,
  reqImports["nonces"].get(accounts[0].address).addn(1)
);
}

async function assetTransfer(api, accounts, dids, prepend) {
    
    const ticker = `token${prepend}0`.toUpperCase();
    assert( ticker.length <= 12, "Ticker cannot be longer than 12 characters");

    const unsub = await api.tx.asset
      .transfer(ticker, dids[2], 100)
      .signAndSend(
        accounts[0],
        { nonce: reqImports["nonces"].get(accounts[0].address) },
        ({ events = [], status }) => {
          console.log(`events is ${events} and status is ${status}`);
          if (status.isFinalized) {
            console.log(`events is ${events} and status is ${status}`);
            reqImports["fail_count"] = reqImports["callback"](
              status,
              events,
              "",
              "",
              reqImports["fail_count"]
            );
            unsub();
          }
        }
      );

    reqImports["nonces"].set(
      accounts[0].address,
      reqImports["nonces"].get(accounts[0].address).addn(1)
    );
  
}

main().catch(console.error);
