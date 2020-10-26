// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");
// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

/**
 * This test checks the ability to do a manual STO receiving payment in an asset and reciepts
 */

const prepend = "EUR";
const prepend2 = "USD";

async function main() {
  const api = await reqImports.createApi();
  const ticker = `token${prepend}0`.toUpperCase();
  const ticker2 = `token${prepend2}0`.toUpperCase();
  const testEntities = await reqImports.initMain(api);

  let alice = testEntities[0];
  let bob = testEntities[1];
  let charlie = testEntities[2];
  let dave = testEntities[3];
  let eve = testEntities[7];

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

  await reqImports.issueTokenPerDid(api, [alice], prepend);

  await reqImports.issueTokenPerDid(api, [bob], prepend2);

  await reqImports.addComplianceRequirement(api, alice, ticker);
  await reqImports.addComplianceRequirement(api, bob, ticker2);

  await reqImports.mintingAsset(api, alice, alice_did, prepend);

  await reqImports.mintingAsset(api, bob, bob_did, prepend2);

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

  await reqImports.authorizeInstruction(api, alice, instructionCounter, alice_did);

  await reqImports.authorizeInstruction(api, bob, instructionCounter, bob_did);

  await reqImports.authorizeInstruction(api, charlie, instructionCounter, charlie_did);

  await reqImports.authorizeInstruction(api, dave, instructionCounter, dave_did);

  //await reqImports.rejectInstruction(api, eve, instructionCounter);
  await reqImports.authorizeInstruction(api, eve, instructionCounter, eve_did);

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
  let makeLeg = function ([from, to]) {
    let leg = {
      NonConfidentialLeg: {
        from: group[from],
        to: group[to],
        asset: ticker,
        amount: amount,
      },
    };
    return leg;
  };
  let legs = [
    [1, 0],
    [0, 1],
    [0, 2],
    [0, 3],
    [0, 4],
  ].map(makeLeg);
  transaction = await api.tx.settlement.addInstruction(venueCounter, 0, null, legs);

  let tx = await reqImports.sendTx(sender, transaction);
  if (tx !== -1) reqImports.fail_count--;

  return instructionCounter;
}

main().catch(console.error);
