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
  console.log(`---------> Bob DID: ${bob_did}`);

  let alice_did = JSON.parse(
    await reqImports.keyToIdentityIds(api, alice.publicKey)
  );
  alice_did = alice_did.Unique;
  console.log(`---------> Alice DID: ${alice_did}`);

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

  let venueCounter = await createVenue(api, alice);

  let intructionCounterAB = await addInstruction(
    api,
    venueCounter,
    alice,
    alice_did,
    bob_did,
    ticker,
    20
  );

  console.log(`instructionAB -> ${intructionCounterAB}`);
 
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
  let nonceObj = { nonce: reqImports.nonces.get(sender.address) };
  const transaction = await api.tx.complianceManager.addActiveRule(
    ticker,
    [],
    []
  );

  const result = await reqImports.sendTransaction(
    transaction,
    sender,
    nonceObj
  );

  const passed = result.findRecord("system", "ExtrinsicSuccess");
  if (passed) reqImports.fail_count--;

  reqImports.nonces.set(
    sender.address,
    reqImports.nonces.get(sender.address).addn(1)
  );
}

async function createVenue(api, sender) {
  let venueCounter = await api.query.settlement.venueCounter();
  let nonceObj = { nonce: reqImports.nonces.get(sender.address) };
  let venueDetails = [0];

  const transaction = await api.tx.settlement.createVenue(venueDetails, [
    sender.address,
  ]);

  const result = await reqImports.sendTransaction(
    transaction,
    sender,
    nonceObj
  );

  const passed = result.findRecord("system", "ExtrinsicSuccess");
  if (passed) reqImports.fail_count--;

  reqImports.nonces.set(
    sender.address,
    reqImports.nonces.get(sender.address).addn(1)
  );

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
  let result;
  let instructionCounter = await api.query.settlement.instructionCounter();
  let nonceObj = { nonce: reqImports.nonces.get(sender.address) };
  let leg = {
    from: sender_did,
    to: receiver_did,
    asset: ticker,
    amount: amount,
  };

  console.log(`leg -> ${JSON.stringify(leg)}`);
 
    transaction = await api.tx.settlement.addInstruction(
      venueCounter,
      0,
      null,
      [leg]
    );

    result = await reqImports.sendTransaction(transaction, sender, nonceObj);
  

  const passed = result.findRecord("system", "ExtrinsicSuccess");
  if (passed) reqImports.fail_count--;

  reqImports.nonces.set(
    sender.address,
    reqImports.nonces.get(sender.address).addn(1)
  );

  return instructionCounter;
}

async function authorizeInstruction(api, sender, instructionCounter) {
  let nonceObj = { nonce: reqImports.nonces.get(sender.address) };
  const transaction = await api.tx.settlement.authorizeInstruction(
    instructionCounter
  );
  const result = await reqImports.sendTransaction(
    transaction,
    sender,
    nonceObj
  );
  const passed = result.findRecord("system", "ExtrinsicSuccess");
  if (passed) reqImports.fail_count--;

  reqImports.nonces.set(
    sender.address,
    reqImports.nonces.get(sender.address).addn(1)
  );
}

async function unauthorizeInstruction(api, sender, instructionCounter) {
  let nonceObj = { nonce: reqImports.nonces.get(sender.address) };
  const transaction = await api.tx.settlement.unauthorizeInstruction(
    instructionCounter
  );
  const result = await reqImports.sendTransaction(
    transaction,
    sender,
    nonceObj
  );
  const passed = result.findRecord("system", "ExtrinsicSuccess");
  if (passed) reqImports.fail_count--;

  reqImports.nonces.set(
    sender.address,
    reqImports.nonces.get(sender.address).addn(1)
  );
}

async function rejectInstruction(api, sender, instructionCounter) {
  let nonceObj = { nonce: reqImports.nonces.get(sender.address) };
  const transaction = await api.tx.settlement.rejectInstruction(
    instructionCounter
  );
  const result = await reqImports.sendTransaction(
    transaction,
    sender,
    nonceObj
  );
  const passed = result.findRecord("system", "ExtrinsicSuccess");
  if (passed) reqImports.fail_count--;

  reqImports.nonces.set(
    sender.address,
    reqImports.nonces.get(sender.address).addn(1)
  );
}

main().catch(console.error);
