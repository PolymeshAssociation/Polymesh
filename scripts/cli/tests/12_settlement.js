// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

const prepend = "DEMOAT";

async function main() {
  const api = await reqImports.createApi();
  const ticker = `token${prepend}0`.toUpperCase();

  const testEntities = await reqImports.initMain(api);

  let alice = testEntities[0];
  let bob = testEntities[1];
  let charlie = testEntities[2];
  let dave = testEntities[3];

  let dave_did = await reqImports.createIdentities( api, [dave], alice );
  dave_did = dave_did[0];
  console.log(`---------> Dave DID: ${dave_did}`);

  let charlie_did = await reqImports.createIdentities( api, [charlie], alice );
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

  console.log(0);

  await reqImports.distributePolyBatch(
    api,
    [bob, charlie, dave],
    reqImports.transfer_amount,
    alice
  );

  console.log(1);

  await reqImports.issueTokenPerDid(api, [alice], prepend);

  console.log(2);

  await addActiveRule(api, alice, `token${prepend}0`);

  console.log(2.5);

  let aliceBalance = await api.query.asset.balanceOf(ticker, alice_did);
  let bobBalance = await api.query.asset.balanceOf(ticker, bob_did);
  let charlieBalance = await api.query.asset.balanceOf(ticker, charlie_did);
  let daveBalance = await api.query.asset.balanceOf(ticker, dave_did);

  console.log(`alice asset balance -------->  ${aliceBalance}`);
  console.log(`bob asset balance -------->  ${bobBalance}`);
  console.log(`charlie asset balance -------->  ${charlieBalance}`);
  console.log(`dave asset balance -------->  ${daveBalance}`);


  console.log(3);

  let venueCounter = await createVenue(api, alice);

  console.log(4);

  let intructionCounterAB = await addInstruction(
    api,
    venueCounter,
    alice,
    alice_did,
    bob_did,
    `token${prepend}0`,
    100
  );
console.log(`instructionAB -> ${intructionCounterAB}`);
  let intructionCounterAC = await addInstruction(
    api,
    venueCounter,
    alice,
    alice_did,
    charlie_did,
    `token${prepend}0`,
    100
  );
  console.log(`instructionAC -> ${intructionCounterAC}`);
  let intructionCounterAD = await addInstruction(
    api,
    venueCounter,
    alice,
    alice_did,
    dave_did,
    `token${prepend}0`,
    100
  );
  console.log(`instructionAD -> ${intructionCounterAD}`);

   console.log(6);
   await authorizeInstruction(api, alice, intructionCounterAB);
   await authorizeInstruction(api, bob, intructionCounterAB);
   console.log(7);
   await authorizeInstruction(api, alice, intructionCounterAC);
   console.log(7.5);
   await authorizeInstruction(api, charlie, intructionCounterAC);
   //await rejectInstruction(api, bob, intructionCounter);
   console.log(8);
   await authorizeInstruction(api, alice, intructionCounterAD);
   await rejectInstruction(api, dave, intructionCounterAD);
   console.log(9);
  

  aliceBalance = await api.query.asset.balanceOf(ticker, alice_did);
  bobBalance = await api.query.asset.balanceOf(ticker, bob_did);
  charlieBalance = await api.query.asset.balanceOf(ticker, charlie_did);
  daveBalance = await api.query.asset.balanceOf(ticker, dave_did);

  console.log(`alice asset balance -------->  ${aliceBalance}`);
  console.log(`bob asset balance -------->  ${bobBalance}`);
  console.log(`charlie asset balance -------->  ${charlieBalance}`);
  console.log(`dave asset balance -------->  ${daveBalance}`);

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function addActiveRule(api, sender, ticker) {
  let uppercaseTicker = ticker.toUpperCase();
  let nonceObj = { nonce: reqImports.nonces.get(sender.address) };
  const transaction = await api.tx.complianceManager.addActiveRule(
    uppercaseTicker,
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
  reciever_did,
  ticker,
  amount
) {
  let instructionCounter = await api.query.settlement.instructionCounter();
  let nonceObj = { nonce: reqImports.nonces.get(sender.address) };
  let leg = {
    from: sender_did,
    to: reciever_did,
    asset: ticker.toUpperCase(),
    amount: amount,
  };
 console.log(`leg info -> ${JSON.stringify(leg)}`);
  const transaction = await api.tx.settlement.addInstruction(
    venueCounter,
     0,
    null,
    [leg]
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
