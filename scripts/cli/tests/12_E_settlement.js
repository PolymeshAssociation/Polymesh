// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

/**
 * This test checks the ability of an exchange to mediate a transfer of assets between Alice and Bob.
 */

const prepend = "ACME";
const prepend2 = "USD";
async function main() {
  const api = await reqImports.createApi();
  const ticker = `token${prepend}0`.toUpperCase();
  const ticker2 = `token${prepend2}0`.toUpperCase();
  const testEntities = await reqImports.initMain(api);

  let alice = testEntities[0];
  let bob = testEntities[1];
  let charlie = testEntities[2];

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
    [bob, charlie],
    reqImports.transfer_amount,
    alice
  );

  await reqImports.issueTokenPerDid(api, [alice], prepend);

  await reqImports.issueTokenPerDid(api, [bob], prepend2);

  await addActiveRule(api, alice, ticker);
  await addActiveRule(api, bob, ticker2);

  await reqImports.mintingAsset(api, alice, alice_did, prepend);

  await reqImports.mintingAsset(api, bob, bob_did, prepend2);

  let aliceACMEBalance = await api.query.asset.balanceOf(ticker, alice_did);
  let bobACMEBalance = await api.query.asset.balanceOf(ticker, bob_did);

  let aliceUSDBalance = await api.query.asset.balanceOf(ticker2, alice_did);
  let bobUSDBalance = await api.query.asset.balanceOf(ticker2, bob_did);

  console.log("Balance for ACME (Before)");
  console.log(`alice asset balance -------->  ${aliceACMEBalance}`);
  console.log(`bob asset balance -------->  ${bobACMEBalance}`);

  console.log(" ");
  console.log("Balance for USD_ASSET (Before)");
  console.log(`alice asset balance -------->  ${aliceUSDBalance}`);
  console.log(`bob asset balance -------->  ${bobUSDBalance}`);

  let venueCounter = await createVenue(api, charlie);

  let instructionCounter1 = await addInstruction(
    api,
    venueCounter,
    alice_did,
    bob_did,
    ticker,
    ticker2,
    100,
    charlie
  );

  let instructionCounter2 = await addInstruction(
    api,
    venueCounter,
    alice_did,
    bob_did,
    ticker,
    ticker2,
    100,
    charlie
  );

  await authorizeWithReceipts(api, charlie, alice, instructionCounter1, 0);

  await authorizeWithReceipts(api, charlie, bob, instructionCounter1, 1);

  await authorizeInstruction(api, alice, instructionCounter2);

  await claimReceipt(
    api,
    alice,
    alice_did,
    bob_did,
    ticker,
    100,
    instructionCounter2,
    charlie
  );

  await authorizeInstruction(api, bob, instructionCounter2);

  aliceACMEBalance = await api.query.asset.balanceOf(ticker, alice_did);
  bobACMEBalance = await api.query.asset.balanceOf(ticker, bob_did);

  aliceUSDBalance = await api.query.asset.balanceOf(ticker2, alice_did);
  bobUSDBalance = await api.query.asset.balanceOf(ticker2, bob_did);

  console.log("Balance for ACME (After)");
  console.log(`alice asset balance -------->  ${aliceACMEBalance}`);
  console.log(`bob asset balance -------->  ${bobACMEBalance}`);
  console.log(" ");
  console.log("Balance for USD_ASSET (After)");
  console.log(`alice asset balance -------->  ${aliceUSDBalance}`);
  console.log(`bob asset balance -------->  ${bobUSDBalance}`);

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function authorizeWithReceipts(
  api,
  exchange,
  sender,
  instructionCounter,
  leg
) {
  let nonceObj = { nonce: reqImports.nonces.get(sender.address) };

  receiptID = Math.floor(Math.random() * 101); // returns a random integer from 0 to 100
  console.log(`receipt 1 ID -> ${receiptID}`);
  receiptID2 = Math.floor(Math.random() * 101); // returns a random integer from 0 to 100
  console.log(`receipt 2 ID -> ${receiptID2}`);

  let receiptDetails = {
    receipt_uid: receiptID,
    leg_id: leg,
    signer: exchange.address,
    signature: 1,
  };

  const transaction = await api.tx.settlement.authorizeWithReceipts(
    instructionCounter,
    [receiptDetails]
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

async function claimReceipt(
  api,
  from,
  from_did,
  receiver_did,
  ticker,
  amount,
  instructionCounter,
  exchange
) {
  let nonceObj = { nonce: reqImports.nonces.get(from.address) };
  console.log(JSON.stringify(nonceObj));
  let msg = {
    receipt_uid: 0,
    from: from_did,
    to: receiver_did,
    asset: ticker,
    amount: amount,
  };

  receiptID = Math.floor(Math.random() * 101); // returns a random integer from 0 to 100

  let receiptDetails = {
    receipt_uid: receiptID,
    leg_id: 0,
    signer: exchange.address,
    signature: 1,
  };

  const transaction = await api.tx.settlement.claimReceipt(
    instructionCounter,
    receiptDetails
  );

  const result = await reqImports.sendTransaction(transaction, from, nonceObj);

  const passed = result.findRecord("system", "ExtrinsicSuccess");
  if (passed) reqImports.fail_count--;

  reqImports.nonces.set(
    from.address,
    reqImports.nonces.get(from.address).addn(1)
  );
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
  from_did,
  receiver_did,
  ticker,
  ticker2,
  amount,
  sender
) {
  let result;
  let instructionCounter = await api.query.settlement.instructionCounter();
  let nonceObj = { nonce: reqImports.nonces.get(sender.address) };

  let leg = {
    from: from_did,
    to: receiver_did,
    asset: ticker,
    amount: amount,
  };

  console.log(`leg -> ${JSON.stringify(leg)}`);

  let leg2 = {
    from: receiver_did,
    to: from_did,
    asset: ticker2,
    amount: amount,
  };

  if (ticker2 === null || ticker2 === undefined) {
    transaction = await api.tx.settlement.addInstruction(
      venueCounter,
      0,
      null,
      [leg]
    );

    result = await reqImports.sendTransaction(transaction, sender, nonceObj);
  } else {
    transaction = await api.tx.settlement.addInstruction(
      venueCounter,
      0,
      null,
      [leg, leg2]
    );

    result = await reqImports.sendTransaction(transaction, sender, nonceObj);
  }

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

  console.log(`instruction Num -> ${instructionCounter}`);
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

main().catch(console.error);
