// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");
const { encodeAddress } = require("@polkadot/util-crypto");
// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

/*
 * This test checks the ability to do a manual STO receiving payment in an asset and reciepts 
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

  await reqImports.issueTokenPerDid(api, [bob], prepend2);

  await reqImports.addActiveRule(api, alice, ticker);
  await reqImports.addActiveRule(api, bob, ticker2);

  await reqImports.mintingAsset(api, alice, alice_did, prepend);

  await reqImports.mintingAsset(api, bob, bob_did, prepend2);

  

  let aliceACMEBalance = await api.query.asset.balanceOf(ticker, alice_did);
  let bobACMEBalance = await api.query.asset.balanceOf(ticker, bob_did);
  let charlieACMEBalance = await api.query.asset.balanceOf(ticker, charlie_did);
  let daveACMEBalance = await api.query.asset.balanceOf(ticker, dave_did);

  let aliceUSDBalance = await api.query.asset.balanceOf(ticker2, alice_did);
  let bobUSDBalance = await api.query.asset.balanceOf(ticker2, bob_did);
  let charlieUSDBalance = await api.query.asset.balanceOf(ticker2, charlie_did);
  let daveUSDBalance = await api.query.asset.balanceOf(ticker2, dave_did);

  console.log("Balance for ACME (Before)");
  console.log(`alice asset balance -------->  ${aliceACMEBalance}`);
  console.log(`bob asset balance -------->  ${bobACMEBalance}`);
  console.log(`charlie asset balance -------->  ${charlieACMEBalance}`);
  console.log(`dave asset balance -------->  ${daveACMEBalance}`);
  console.log(" ");
  console.log("Balance for USD_ASSET (Before)");
  console.log(`alice asset balance -------->  ${aliceUSDBalance}`);
  console.log(`bob asset balance -------->  ${bobUSDBalance}`);
  console.log(`charlie asset balance -------->  ${charlieUSDBalance}`);
  console.log(`dave asset balance -------->  ${daveUSDBalance}`);
  console.log(" ");

  let venueCounter = await reqImports.createVenue(api, alice);
  let bobVenueCounter = await reqImports.createVenue(api, bob);

  let intructionUSDCounterBC = await reqImports.addInstruction(
    api,
    bobVenueCounter,
    bob,
    bob_did,
    charlie_did,
    ticker2,
    null,
    500
  );

  let intructionUSDCounterBD = await reqImports.addInstruction(
    api,
    bobVenueCounter,
    bob,
    bob_did,
    dave_did,
    ticker2,
    null,
    500
  );

  await reqImports.authorizeInstruction(api, bob, intructionUSDCounterBC);
  await reqImports.authorizeInstruction(api, charlie, intructionUSDCounterBC);
  
  await reqImports.authorizeInstruction(api, bob, intructionUSDCounterBD);
  await reqImports.authorizeInstruction(api, dave, intructionUSDCounterBD);

  aliceUSDBalance = await api.query.asset.balanceOf(ticker2, alice_did);
  bobUSDBalance = await api.query.asset.balanceOf(ticker2, bob_did);
  charlieUSDBalance = await api.query.asset.balanceOf(ticker2, charlie_did);
  daveUSDBalance = await api.query.asset.balanceOf(ticker2, dave_did);

  console.log("Balance for USD_ASSET (After)");
  console.log(`alice asset balance -------->  ${aliceUSDBalance}`);
  console.log(`bob asset balance -------->  ${bobUSDBalance}`);
  console.log(`charlie asset balance -------->  ${charlieUSDBalance}`);
  console.log(`dave asset balance -------->  ${daveUSDBalance}`);
  console.log(" ");

  let instructionCounterAB = await reqImports.addInstruction(
    api,
    venueCounter,
    alice,
    alice_did,
    bob_did,
    ticker,
    ticker2,
    100
  );
  
  let instructionCounterAC = await reqImports.addInstruction(
    api,
    venueCounter,
    alice,
    alice_did,
    charlie_did,
    ticker,
    ticker2,
    100
  );
  
  let instructionCounterAD = await reqImports.addInstruction(
    api,
    venueCounter,
    alice,
    alice_did,
    dave_did,
    ticker,
    ticker2,
    100
  );

  await reqImports.authorizeInstruction(api, alice, instructionCounterAB);

  await claimReceipt(api, alice, alice_did, bob_did, ticker, 100, instructionCounterAB);
  await reqImports.authorizeInstruction(api, bob, instructionCounterAB);

  await reqImports.authorizeInstruction(api, alice, instructionCounterAC);
  await reqImports.authorizeInstruction(api, charlie, instructionCounterAC);
  
  await reqImports.authorizeInstruction(api, alice, instructionCounterAD);
  await reqImports.rejectInstruction(api, dave, instructionCounterAD);


  aliceACMEBalance = await api.query.asset.balanceOf(ticker, alice_did);
  bobACMEBalance = await api.query.asset.balanceOf(ticker, bob_did);
  charlieACMEBalance = await api.query.asset.balanceOf(ticker, charlie_did);
  daveACMEBalance = await api.query.asset.balanceOf(ticker, dave_did);

  aliceUSDBalance = await api.query.asset.balanceOf(ticker2, alice_did);
  bobUSDBalance = await api.query.asset.balanceOf(ticker2, bob_did);
  charlieUSDBalance = await api.query.asset.balanceOf(ticker2, charlie_did);
  daveUSDBalance = await api.query.asset.balanceOf(ticker2, dave_did);

  console.log("Balance for ACME (After)");
  console.log(`alice asset balance -------->  ${aliceACMEBalance}`);
  console.log(`bob asset balance -------->  ${bobACMEBalance}`);
  console.log(`charlie asset balance -------->  ${charlieACMEBalance}`);
  console.log(`dave asset balance -------->  ${daveACMEBalance}`);
  console.log(" ");
  console.log("Balance for USD_ASSET (After)");
  console.log(`alice asset balance -------->  ${aliceUSDBalance}`);
  console.log(`bob asset balance -------->  ${bobUSDBalance}`);
  console.log(`charlie asset balance -------->  ${charlieUSDBalance}`);
  console.log(`dave asset balance -------->  ${daveUSDBalance}`);

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function claimReceipt(api, sender, sender_did, receiver_did, ticker, amount, instructionCounter) {
  let msg = {
    receipt_uid: 0,
    from: sender_did,
    to: receiver_did,
    asset: ticker,
    amount: amount,
  };

let receiptDetails = {
  receipt_uid: 0,
  leg_id: 0,
  signer: sender.address,
  signature: 1
  }

  const transaction = await api.tx.settlement.claimReceipt(
    instructionCounter,
    receiptDetails
  );

  let tx = await reqImports.sendTx(sender, transaction);
  if(tx !== -1) reqImports.fail_count--;
}

main().catch(console.error);
