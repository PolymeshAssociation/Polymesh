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
  console.log(`---------> Dave DID: ${dave_did}`);

  let charlie_did = await reqImports.createIdentities(api, [charlie], alice);
  charlie_did = charlie_did[0];
  console.log(`---------> Charlie DID: ${charlie_did}`);

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
    [bob, charlie, dave],
    reqImports.transfer_amount,
    alice
  );

  await reqImports.issueTokenPerDid(api, [alice], prepend);

  await addActiveRule(api, alice, ticker);

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
 
  let intructionCounterAC = await addInstruction(
    api,
    venueCounter,
    alice,
    alice_did,
    charlie_did,
    ticker,
    100
  );
  
  let intructionCounterAD = await addInstruction(
    api,
    venueCounter,
    alice,
    alice_did,
    dave_did,
    ticker,
    100
  );

  await authorizeInstruction(api, alice, intructionCounterAB);
  await authorizeInstruction(api, bob, intructionCounterAB);
 
  await authorizeInstruction(api, alice, intructionCounterAC);
  await authorizeInstruction(api, charlie, intructionCounterAC);

  await authorizeInstruction(api, alice, intructionCounterAD);
  await rejectInstruction(api, dave, intructionCounterAD);
  

  aliceACMEBalance = await api.query.asset.balanceOf(ticker, alice_did);
  bobACMEBalance = await api.query.asset.balanceOf(ticker, bob_did);
  charlieACMEBalance = await api.query.asset.balanceOf(ticker, charlie_did);
  daveACMEBalance = await api.query.asset.balanceOf(ticker, dave_did);

  console.log("Balance for ACME (After)");
  console.log(`alice asset balance -------->  ${aliceACMEBalance}`);
  console.log(`bob asset balance -------->  ${bobACMEBalance}`);
  console.log(`charlie asset balance -------->  ${charlieACMEBalance}`);
  console.log(`dave asset balance -------->  ${daveACMEBalance}`);
 

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
