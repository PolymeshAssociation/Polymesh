// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {
  
  const api = await reqImports.createApi();

  const asset_did = reqImports.tickerToDid(reqImports.ticker);

  const testEntities = await reqImports.initMain(api);

  let master_keys = await reqImports.generateKeys( api, 3, "master" );

  let signing_keys = await reqImports.generateKeys( api, 3, "signing" );

  await reqImports.createIdentities( api, testEntities, testEntities[0] );

  await reqImports.distributePoly( api, master_keys.concat(signing_keys), reqImports.transfer_amount, testEntities[0] );

  await reqImports.blockTillPoolEmpty(api);

  let issuer_dids = await reqImports.createIdentities( api, master_keys, testEntities[0] );

  await reqImports.addSigningKeys( api, master_keys, issuer_dids, signing_keys );

  await reqImports.authorizeJoinToIdentities( api, master_keys, issuer_dids, signing_keys );

  await reqImports.blockTillPoolEmpty(api);

  await reqImports.issueTokenPerDid( api, master_keys );

  // receiverRules Claim
  await reqImports.addClaimsToDids( api, master_keys, issuer_dids[2], "Whitelisted", asset_did );

  // senderRules Claim
  await reqImports.addClaimsToDids( api, master_keys, issuer_dids[0], "Whitelisted", asset_did );

  await reqImports.createClaimRules( api, master_keys, issuer_dids );

  await mintingAsset( api, master_keys, issuer_dids );

  await assetTransfer( api, master_keys, issuer_dids );

  await reqImports.blockTillPoolEmpty(api);

  await new Promise(resolve => setTimeout(resolve, 3000));

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function mintingAsset(api, accounts, dids) {

const unsub = await api.tx.asset
  .issue(reqImports.ticker, dids[2], 100, "")
  .signAndSend(
    accounts[0],
    { nonce: reqImports.nonces.get(accounts[0].address) },
    ({ events = [], status }) => {

    if (status.isFinalized) {

      // Loop through Vec<EventRecord> to display all events
      events.forEach(({ phase, event: { data, method, section } }) => {
       if ( section === "system" && method === "ExtrinsicSuccess" )  reqImports.fail_count--;
      });

      unsub();
    }
  });

reqImports.nonces.set(accounts[0].address, reqImports.nonces.get(accounts[0].address).addn(1));
}

async function assetTransfer(api, accounts, dids) {

    const unsub = await api.tx.asset
      .transfer(reqImports.ticker, dids[2], 100)
      .signAndSend(
        accounts[0],
        { nonce: reqImports.nonces.get(accounts[0].address) },
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

    reqImports.nonces.set( accounts[0].address, reqImports.nonces.get(accounts[0].address).addn(1));
  
}

main().catch(console.error);
