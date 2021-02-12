// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {

  const api = await reqImports.createApi();

  const testEntities = await reqImports.initMain(api);

  let alice = testEntities[0];
  let dave = await reqImports.generateRandomEntity(api);
  let bob = await reqImports.generateRandomEntity(api);
  let govCommittee1 = testEntities[2];
  let govCommittee2 = testEntities[3];

  await sendTx(alice, api.tx.sudo.sudo(api.tx.pips.setDefaultEnactmentPeriod(10)));
  await reqImports.createIdentities(api, [bob, dave, govCommittee1, govCommittee2], alice);

  // Bob and Dave needs some funds to use.
  await reqImports.distributePolyBatch(api, [bob, dave], reqImports.transfer_amount, alice);

  await sendTx(dave, api.tx.staking.bond(bob.publicKey, 20000, "Staked"));
  const setLimit = api.tx.pips.setActivePipLimit(42);

  let firstPipCount = await api.query.pips.pipIdSequence();
  await sendTx(bob, api.tx.pips.propose(setLimit, 9000000000, "google.com", "first"));

  let secondPipCount = await api.query.pips.pipIdSequence();
  await sendTx(bob, api.tx.pips.propose(setLimit, 10000000000, "google.com", "second"));

  // GC needs some funds to use.
  await reqImports.distributePolyBatch(api, [govCommittee1, govCommittee2], reqImports.transfer_amount, alice);

  // Snapshot and approve second PIP.
  await sendTx(govCommittee1, api.tx.pips.snapshot());
  const approvePIP = api.tx.pips.enactSnapshotResults([[secondPipCount, { "Approve": "" }]]);
  await voteResult(api, approvePIP, [govCommittee1, govCommittee2]);

  // Reject the first PIP
  const rejectPIP = api.tx.pips.rejectProposal(firstPipCount);
  await voteResult(api, rejectPIP, [govCommittee1, govCommittee2]);

  // Finally reschedule, demonstrating that it had been scheduled.
  await sendTx(alice, api.tx.pips.rescheduleExecution(secondPipCount, null));
  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function sendTx(signer, tx) {
  let nonceObj = { nonce: reqImports.nonces.get(signer.address) };
  const result = await reqImports.sendTransaction(tx, signer, nonceObj);
  const passed = result.findRecord('system', 'ExtrinsicSuccess');
  if (passed) reqImports.fail_count--;
  reqImports.nonces.set(signer.address, reqImports.nonces.get(signer.address).addn(1));
}

async function voteResult(api, tx, signers) {
  const vote = api.tx.polymeshCommittee.voteOrPropose(true, tx);
  for (let i = 0; i < signers.length; i++) {
    await sendTx(signers[i], vote);
  }
}

main().catch(console.error);
