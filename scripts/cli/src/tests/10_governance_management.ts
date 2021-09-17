import {
  ApiSingleton,
  generateEntityFromUri,
  initMain,
  sendTx,
  transferAmount,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import * as staking from "../helpers/staking_helper";
import * as pips from "../helpers/pips_helper";
import * as committee from "../helpers/committee_helper";
import Keyring from "@polkadot/keyring";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
  const testEntities = await initMain();
  const alice = testEntities[0];
  const govCommittee1 = testEntities[2];
  const govCommittee2 = testEntities[3];
  const bob = await generateEntityFromUri("10_bob");
  const dave = await generateEntityFromUri("10_dave");
  await pips.setDefaultEnactmentPeriod(alice, 10);
  await createIdentities(alice, [bob, dave]);

  // Bob and Dave needs some funds to use.
  await distributePolyBatch(alice, [bob, dave], transferAmount);
  await staking.bond(dave, bob, 1_000_000, "Staked");
  const setLimit = await pips.setActivePipLimit(42);

  const firstPipCount = await pips.pipIdSequence();
  await pips.propose(bob, setLimit, 9_000_000_000, "google.com", "first");
  console.log("after propose");

  const secondPipCount = await pips.pipIdSequence();
  await pips.propose(bob, setLimit, 10_000_000_000, "google.com", "second");
  await sendTx(
    dave,
    (await ApiSingleton.getInstance()).tx.pips.vote(1, true, 10)
  );

  // GC needs some funds to use.
  await distributePolyBatch(
    alice,
    [govCommittee1, govCommittee2],
    transferAmount
  );

  // Snapshot and approve second PIP.
  await pips.snapshot(govCommittee1);
  const approvePIP = await pips.enactSnapshotResults(secondPipCount, "Approve");
  await pips.voteResult([govCommittee1, govCommittee2], approvePIP);

  // Reject the first PIP
  const rejectPIP = await pips.rejectProposal(firstPipCount);
  await pips.voteResult([govCommittee1, govCommittee2], rejectPIP);

  // Finally reschedule, demonstrating that it had been scheduled.
  await pips.rescheduleProposal(govCommittee1, secondPipCount);
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => {
    console.log("Completed: GOVERNANCE MANAGEMENT");
    process.exit();
  });
