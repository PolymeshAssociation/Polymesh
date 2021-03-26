// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");
const assert = require('assert');

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
  // Reset ActivePipLimit to 100
  await sendTx(alice, api.tx.sudo.sudo(api.tx.pips.setActivePipLimit(100)));
  await reqImports.createIdentities(api, [bob, dave, govCommittee1, govCommittee2], alice);

  // Bob and Dave needs some funds to use.
  await reqImports.distributePolyBatch(api, [bob, dave], reqImports.transfer_amount, alice);

  await sendTx(dave, api.tx.staking.bond(bob.publicKey, 1000000, "Staked"));

  // GC needs some funds to use.
  await reqImports.distributePolyBatch(api, [govCommittee1, govCommittee2], reqImports.transfer_amount, alice);

  let pipId = await basicVote(api.tx.pips.setActivePipLimit(100), { "Approve": "" });
  // Finally reschedule, demonstrating that it had been scheduled.
  await sendTx(alice, api.tx.pips.rescheduleExecution(pipId, null));

  pipId = await basicVote(api.tx.pips.setActivePipLimit(101), { "Skip": "" });
  assert.deepStrictEqual(api.query.pips.pipSkipCount(pipId), 2);

  pipId = await basicVote(api.tx.pips.setActivePipLimit(102), { "Reject": "" });
  assert.deepStrictEqual(api.query.pips.proposals(pipId).state, { "Rejected": "" });


  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();

  async function basicVote(dummyProposal, snapshotResult) {
    let pipId = await api.query.pips.pipIdSequence();
    await sendTx(bob, api.tx.pips.propose(dummyProposal, 9000000000, "basic-vote.com", "basicVote"));
    await committeeVote(api, pipId, [govCommittee1, govCommittee2], snapshotResult);
    return pipId;
  }
}

async function sendTx(signer, tx) {
  let nonceObj = { nonce: reqImports.nonces.get(signer.address) };
  const result = await reqImports.sendTransaction(tx, signer, nonceObj);
  const passed = result.findRecord('system', 'ExtrinsicSuccess');
  if (passed) reqImports.fail_count--;
  reqImports.nonces.set(signer.address, reqImports.nonces.get(signer.address).addn(1));
}

async function committeeVote(api, pipId, committees, snapshotResult) {
  await sendTx(committees[0], api.tx.pips.snapshot());
  let voteTx = api.tx.pips.enactSnapshotResults([[pipId, snapshotResult]])
  const vote = api.tx.polymeshCommittee.voteOrPropose(true, voteTx);
  for (let i = 0; i < committees.length; i++) {
    await sendTx(committees[i], vote);
  }
}

main().catch(console.error);
