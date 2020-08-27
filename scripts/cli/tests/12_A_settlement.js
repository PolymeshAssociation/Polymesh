// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

/*
 * This test is for the basic peer to peer transfer of tokens
 */

const prepend = "ACME";

async function main() {
  const api = await reqImports.createApi();
  const ticker = `token${prepend}0`.toUpperCase();
  const testEntities = await reqImports.initMain(api);

  let alice = testEntities[0];
  let bob = testEntities[1];

  let bob_did = await reqImports.createIdentities(api, [bob], alice);
  bob_did = bob_did[0];

  let alice_did = JSON.parse(
    await reqImports.keyToIdentityIds(api, alice.publicKey)
  );
  alice_did = alice_did.Unique;

  await reqImports.distributePolyBatch(
    api,
    [bob],
    reqImports.transfer_amount,
    alice
  );

  await reqImports.issueTokenPerDid(api, [alice], prepend);

  await addActiveRule(api, alice, ticker);

  let aliceACMEBalance = await api.query.asset.balanceOf(ticker, alice_did);
  let bobACMEBalance = await api.query.asset.balanceOf(ticker, bob_did);

  console.log("Balance for ACME (Before)");
  console.log(`alice asset balance -------->  ${aliceACMEBalance}`);
  console.log(`bob asset balance -------->  ${bobACMEBalance}`);
  console.log(" ");

  let venueCounter = await createVenue(api, alice);

  let intructionCounterAB = await addInstruction(
    api,
    venueCounter,
    alice,
    alice_did,
    bob_did,
    ticker,
    100
  );

  await authorizeInstruction(api, alice, intructionCounterAB);
  await authorizeInstruction(api, bob, intructionCounterAB);

  //await rejectInstruction(api, bob, intructionCounter);
  //await unathorizeInstruction(api, alice, instructionCounter);

  aliceACMEBalance = await api.query.asset.balanceOf(ticker, alice_did);
  bobACMEBalance = await api.query.asset.balanceOf(ticker, bob_did);

  console.log(`alice asset balance -------->  ${aliceACMEBalance}`);
  console.log(`bob asset balance -------->  ${bobACMEBalance}`);


  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}


async function addActiveRule(api, sender, ticker) {

  const transaction = await api.tx.complianceManager.addActiveRule(
    ticker,
    [],
    []
  );

  let tx = await reqImports.sendTx(sender, transaction);
  if(tx !== -1) reqImports.fail_count--;
}

async function createVenue(api, sender) {
  let venueCounter = await api.query.settlement.venueCounter();
  let venueDetails = [0];

  const transaction = await api.tx.settlement.createVenue(venueDetails, [
    sender.address,
  ], 0);

  let tx = await reqImports.sendTx(sender, transaction);
  if(tx !== -1) reqImports.fail_count--;

  return venueCounter;
}

async function addInstruction(
  api,
  venueCounter,
  sender,
  sender_did,
  receiver_did,
  ticker,
  amount
) {

  let instructionCounter = await api.query.settlement.instructionCounter();

  let leg = {
    from: sender_did,
    to: receiver_did,
    asset: ticker,
    amount: amount,
  };

    transaction = await api.tx.settlement.addInstruction(
      venueCounter,
      0,
      null,
      [leg]
    );

    let tx = await reqImports.sendTx(sender, transaction);
    if(tx !== -1) reqImports.fail_count--;

  return instructionCounter;
}

async function authorizeInstruction(api, sender, instructionCounter) {

  const transaction = await api.tx.settlement.authorizeInstruction(
    instructionCounter
  );

  let tx = await reqImports.sendTx(sender, transaction);
  if(tx !== -1) reqImports.fail_count--;
}

async function unauthorizeInstruction(api, sender, instructionCounter) {

  const transaction = await api.tx.settlement.unauthorizeInstruction(
    instructionCounter
  );

  let tx = await reqImports.sendTx(sender, transaction);
  if(tx !== -1) reqImports.fail_count--;
}

async function rejectInstruction(api, sender, instructionCounter) {

  const transaction = await api.tx.settlement.rejectInstruction(
    instructionCounter
  );

  let tx = await reqImports.sendTx(sender, transaction);
  if(tx !== -1) reqImports.fail_count--;
}

main().catch(console.error);
