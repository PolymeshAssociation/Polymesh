// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

/*
 * This test is for checking the ability to distribute ssets to a group of investors
 * without payment.
 */



const prepend = "ACME";
async function main() {
  const api = await reqImports.createApi();
  const ticker = `token${prepend}0`.toUpperCase();
  const testEntities = await reqImports.initMain(api);

  let alice = testEntities[0];
  let bob = testEntities[1];
  let charlie = testEntities[2];
  let dave = testEntities[3];

  let dave_did = await reqImports.createIdentities(api, [dave], alice);
  dave_did = dave_did[0];

  let charlie_did = await reqImports.createIdentities(api, [charlie], alice);
  charlie_did = charlie_did[0];

  let bob_did = await reqImports.createIdentities(api, [bob], alice);
  bob_did = bob_did[0];

  let alice_did = JSON.parse(
    await reqImports.keyToIdentityIds(api, alice.publicKey)
  );
  alice_did = alice_did.Unique;

  await reqImports.distributePolyBatch(
    api,
    [bob, charlie, dave],
    reqImports.transfer_amount,
    alice
  );

  await reqImports.issueTokenPerDid(api, [alice], prepend);

  await reqImports.addActiveRule(api, alice, ticker);

  await reqImports.mintingAsset(api, alice, alice_did, prepend);

  let aliceACMEBalance = await api.query.asset.balanceOf(ticker, alice_did);
  let bobACMEBalance = await api.query.asset.balanceOf(ticker, bob_did);
  let charlieACMEBalance = await api.query.asset.balanceOf(ticker, charlie_did);
  let daveACMEBalance = await api.query.asset.balanceOf(ticker, dave_did);

  console.log("Balance for ACME (Before)");
  console.log(`alice asset balance -------->  ${aliceACMEBalance}`);
  console.log(`bob asset balance -------->  ${bobACMEBalance}`);
  console.log(`charlie asset balance -------->  ${charlieACMEBalance}`);
  console.log(`dave asset balance -------->  ${daveACMEBalance}`);
  console.log(" ");

  let venueCounter = await reqImports.createVenue(api, alice);

  let intructionCounterAB = await reqImports.addInstruction(
    api,
    venueCounter,
    alice,
    alice_did,
    bob_did,
    ticker,
    null,
    100
  );
 
  let intructionCounterAC = await reqImports.addInstruction(
    api,
    venueCounter,
    alice,
    alice_did,
    charlie_did,
    ticker,
    null,
    100
  );
  
  let intructionCounterAD = await reqImports.addInstruction(
    api,
    venueCounter,
    alice,
    alice_did,
    dave_did,
    ticker,
    null,
    100
  );

  await reqImports.authorizeInstruction(api, alice, intructionCounterAB);
  await reqImports.authorizeInstruction(api, bob, intructionCounterAB);
 
  await reqImports.authorizeInstruction(api, alice, intructionCounterAC);
  await reqImports.authorizeInstruction(api, charlie, intructionCounterAC);

  await reqImports.authorizeInstruction(api, alice, intructionCounterAD);
  await reqImports.rejectInstruction(api, dave, intructionCounterAD);
  

  aliceACMEBalance = await api.query.asset.balanceOf(ticker, alice_did);
  bobACMEBalance = await api.query.asset.balanceOf(ticker, bob_did);
  charlieACMEBalance = await api.query.asset.balanceOf(ticker, charlie_did);
  daveACMEBalance = await api.query.asset.balanceOf(ticker, dave_did);

  console.log("Balance for ACME (After)");
  console.log(`alice asset balance -------->  ${aliceACMEBalance}`);
  console.log(`bob asset balance -------->  ${bobACMEBalance}`);
  console.log(`charlie asset balance -------->  ${charlieACMEBalance}`);
  console.log(`dave asset balance -------->  ${daveACMEBalance}`);
 

  // if (reqImports.fail_count > 0) {
  //   console.log("Failed");
  // } else {
  //   console.log("Passed");
  //   process.exitCode = 0;
  // }

  process.exit();
}

main().catch(console.error);
