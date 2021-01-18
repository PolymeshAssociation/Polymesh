// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");
// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

/**
 * This test checks the ability to do a manual STO receiving payment in an asset and reciepts
 */

async function main() {
  const api = await reqImports.createApi();
  const ticker = await reqImports.generateRandomTicker(api);
  const ticker2 = await reqImports.generateRandomTicker(api);
  const testEntities = await reqImports.initMain(api);

  let alice = testEntities[0];
  let bob = await reqImports.generateRandomEntity(api);
  let charlie = await reqImports.generateRandomEntity(api);
  let dave = await reqImports.generateRandomEntity(api);
  let eve = await reqImports.generateRandomEntity(api);

  let eve_did = await reqImports.createIdentities(api, [eve], alice);
  eve_did = eve_did[0];

  let dave_did = await reqImports.createIdentities(api, [dave], alice);
  dave_did = dave_did[0];

  let charlie_did = await reqImports.createIdentities(api, [charlie], alice);
  charlie_did = charlie_did[0];

  let bob_did = await reqImports.createIdentities(api, [bob], alice);
  bob_did = bob_did[0];

  let alice_did = await reqImports.keyToIdentityIds(api, alice.publicKey);

  await reqImports.distributePolyBatch(
    api,
    [bob, charlie, dave, eve],
    reqImports.transfer_amount,
    alice
  );

  await reqImports.issueTokenPerDid(api, [alice], ticker, 1000000, null);

  await reqImports.issueTokenPerDid(api, [bob], ticker2, 1000000, null);

  await reqImports.addComplianceRequirement(api, alice, ticker);
  await reqImports.addComplianceRequirement(api, bob, ticker2);

  await reqImports.mintingAsset(api, alice, alice_did, ticker);

  await reqImports.mintingAsset(api, bob, bob_did, ticker2);

  let aliceACMEBalance = await api.query.asset.balanceOf(ticker, alice_did);
  let bobACMEBalance = await api.query.asset.balanceOf(ticker, bob_did);
  let charlieACMEBalance = await api.query.asset.balanceOf(ticker, charlie_did);
  let daveACMEBalance = await api.query.asset.balanceOf(ticker, dave_did);
  let eveACMEBalance = await api.query.asset.balanceOf(ticker, eve_did);

  let aliceUSDBalance = await api.query.asset.balanceOf(ticker2, alice_did);
  let bobUSDBalance = await api.query.asset.balanceOf(ticker2, bob_did);
  let charlieUSDBalance = await api.query.asset.balanceOf(ticker2, charlie_did);
  let daveUSDBalance = await api.query.asset.balanceOf(ticker2, dave_did);
  let eveUSDBalance = await api.query.asset.balanceOf(ticker2, eve_did);

  console.log("Balance for EUR (Before)");
  console.log(`alice asset balance -------->  ${aliceACMEBalance}`);
  console.log(`bob asset balance -------->  ${bobACMEBalance}`);
  console.log(`charlie asset balance -------->  ${charlieACMEBalance}`);
  console.log(`dave asset balance -------->  ${daveACMEBalance}`);
  console.log(`eve asset balance -------->  ${eveACMEBalance}`);
  console.log(" ");
  console.log("Balance for USD_ASSET (Before)");
  console.log(`alice asset balance -------->  ${aliceUSDBalance}`);
  console.log(`bob asset balance -------->  ${bobUSDBalance}`);
  console.log(`charlie asset balance -------->  ${charlieUSDBalance}`);
  console.log(`dave asset balance -------->  ${daveUSDBalance}`);
  console.log(`eve asset balance -------->  ${eveUSDBalance}`);
  console.log(" ");

  let venueCounter = await reqImports.createVenue(api, alice);

  let instructionCounter = await addGroupInstruction(
    api,
    venueCounter,
    alice,
    [alice_did, bob_did, charlie_did, dave_did, eve_did],
    ticker,
    ticker2,
    100
  );

  await reqImports.affirmInstruction(api, alice, instructionCounter, alice_did);

  await reqImports.affirmInstruction(api, bob, instructionCounter, bob_did);

  await reqImports.affirmInstruction(api, charlie, instructionCounter, charlie_did);

  await reqImports.affirmInstruction(api, dave, instructionCounter, dave_did);

  //await reqImports.rejectInstruction(api, eve, instructionCounter);
  await reqImports.affirmInstruction(api, eve, instructionCounter, eve_did);

  aliceACMEBalance = await api.query.asset.balanceOf(ticker, alice_did);
  bobACMEBalance = await api.query.asset.balanceOf(ticker, bob_did);
  charlieACMEBalance = await api.query.asset.balanceOf(ticker, charlie_did);
  daveACMEBalance = await api.query.asset.balanceOf(ticker, dave_did);
  eveACMEBalance = await api.query.asset.balanceOf(ticker, eve_did);

  aliceUSDBalance = await api.query.asset.balanceOf(ticker2, alice_did);
  bobUSDBalance = await api.query.asset.balanceOf(ticker2, bob_did);
  charlieUSDBalance = await api.query.asset.balanceOf(ticker2, charlie_did);
  daveUSDBalance = await api.query.asset.balanceOf(ticker2, dave_did);
  eveUSDBalance = await api.query.asset.balanceOf(ticker2, eve_did);

  console.log("Balance for EUR (After)");
  console.log(`alice asset balance -------->  ${aliceACMEBalance}`);
  console.log(`bob asset balance -------->  ${bobACMEBalance}`);
  console.log(`charlie asset balance -------->  ${charlieACMEBalance}`);
  console.log(`dave asset balance -------->  ${daveACMEBalance}`);
  console.log(`eve asset balance -------->  ${eveACMEBalance}`);
  console.log(" ");
  console.log("Balance for USD_ASSET (After)");
  console.log(`alice asset balance -------->  ${aliceUSDBalance}`);
  console.log(`bob asset balance -------->  ${bobUSDBalance}`);
  console.log(`charlie asset balance -------->  ${charlieUSDBalance}`);
  console.log(`dave asset balance -------->  ${daveUSDBalance}`);
  console.log(`eve asset balance -------->  ${eveUSDBalance}`);

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function addGroupInstruction(
  api,
  venueCounter,
  sender,
  group,
  ticker,
  ticker2,
  amount
) {
  let instructionCounter = await api.query.settlement.instructionCounter();
  let leg = {
    from: group[1],
    to: group[0],
    asset: ticker2,
    amount: amount,
  };

  let leg2 = {
    from: group[0],
    to: group[1],
    asset: ticker,
    amount: amount,
  };

  let leg3 = {
    from: group[0],
    to: group[2],
    asset: ticker,
    amount: amount,
  };

  let leg4 = {
    from: group[0],
    to: group[3],
    asset: ticker,
    amount: amount,
  };

  let leg5 = {
    from: group[0],
    to: group[4],
    asset: ticker,
    amount: amount,
  };

  transaction = await api.tx.settlement.addInstruction(venueCounter, 0, null, null, [
    leg,
    leg2,
    leg3,
    leg4,
    leg5,
  ]);

  let tx = await reqImports.sendTx(sender, transaction);
  if (tx !== -1) reqImports.fail_count--;

  return instructionCounter;
}

main().catch(console.error);
