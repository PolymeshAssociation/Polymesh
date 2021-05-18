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

  await sendTx(alice, api.tx.sudo.sudo(api.tx.pips.setDefaultEnactmentPeriod(10)));

  const setLimit = api.tx.pips.setActivePipLimit(42);

  let pipCount = await api.query.pips.pipIdSequence();
  await sendTx(alice, api.tx.pips.propose(setLimit, 10000000000, "google.com", "second"));

  // Snapshot and approve PIP.
  await sendTx(alice, api.tx.pips.snapshot());
  await sendTx(alice, api.tx.polymeshCommittee.voteOrPropose(true, api.tx.pips.enactSnapshotResults([[pipCount, { "approve": "" }]])));

  // Finally reschedule, demonstrating that it had been scheduled.
  await sendTx(alice, api.tx.pips.rescheduleExecution(pipCount, null));
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

main().catch(console.error);
