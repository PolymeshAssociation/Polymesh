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

  let bob_did = await reqImports.createIdentities(api, [bob], alice);
  bob_did = bob_did[0];

  let alice_did = JSON.parse(
    await reqImports.keyToIdentityIds(api, alice.publicKey)
  );
  alice_did = alice_did.Unique;

  await reqImports.distributePolyBatch(
    api,
    [bob, charlie],
    reqImports.transfer_amount,
    alice
  );

  await reqImports.issueTokenPerDid(api, [alice], prepend);

  await reqImports.issueTokenPerDid(api, [bob], prepend2);

  await reqImports.addActiveRule(api, alice, ticker);
  await reqImports.addActiveRule(api, bob, ticker2);

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
  console.log(" ");
  
  let venueCounter = await reqImports.createVenue(api, charlie);

  let instructionCounter1 = await reqImports.addInstruction(
    api,
    venueCounter,
    charlie,
    alice_did,
    bob_did,
    ticker,
    ticker2,
    100
  );

  let instructionCounter2 = await reqImports.addInstruction(
    api,
    venueCounter,
    charlie,
    alice_did,
    bob_did,
    ticker,
    ticker2,
    100
  );

  await authorizeWithReceipts(api, charlie, alice, instructionCounter1, 0);

  await authorizeWithReceipts(api, charlie, bob, instructionCounter1, 1);

  await reqImports.authorizeInstruction(api, alice, instructionCounter2);

  await exchangeClaimReceipt(
    api,
    alice,
    alice_did,
    bob_did,
    ticker,
    100,
    instructionCounter2,
    charlie
  );

  await reqImports.authorizeInstruction(api, bob, instructionCounter2);

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

  receiptID = Math.floor(Math.random() * 101); // returns a random integer from 0 to 100
 
  receiptID2 = Math.floor(Math.random() * 101); // returns a random integer from 0 to 100

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

  const tx = await reqImports.sendTx(sender, transaction);
  if (tx !== -1) reqImports.fail_count--;
}

async function exchangeClaimReceipt(
  api,
  from,
  from_did,
  receiver_did,
  ticker,
  amount,
  instructionCounter,
  exchange
) {
  
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

  const tx = await reqImports.sendTx(from, transaction);
  if (tx !== -1) reqImports.fail_count--;
  
}

main().catch(console.error);
