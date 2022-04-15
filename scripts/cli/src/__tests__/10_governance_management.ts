import {
  initMain,
  generateEntityFromUri,
  disconnect,
  transferAmount,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import * as staking from "../helpers/staking_helper";
import * as pips from "../helpers/pips_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("10 - Governance Management Unit Test", () => {
  test("Create Proposals", async () => {
    const testEntities = await initMain();
    const alice = testEntities[0];
    const govCommittee1 = testEntities[2];
    const govCommittee2 = testEntities[3];
    const bob = await generateEntityFromUri("10_bob");
    const dave = await generateEntityFromUri("10_dave");
    await pips.setDefaultEnactmentPeriod(alice, 10);
    await createIdentities(alice, [bob, dave]);

    // Bob and Dave needs some funds to use.
    await expect(
      distributePolyBatch(alice, [bob, dave], transferAmount)
    ).resolves.not.toThrow();
    await expect(
      staking.bond(dave, bob, 1_000_000, "Staked")
    ).resolves.not.toThrow();
    const setLimit = await pips.setActivePipLimit(42);

    const firstPipCount = await pips.pipIdSequence();
    await expect(
      pips.propose(bob, setLimit, 9_000_000_000, "google.com", "first")
    ).resolves.not.toThrow();
    console.log("after propose");

    const secondPipCount = await pips.pipIdSequence();
    await expect(
      pips.propose(bob, setLimit, 10_000_000_000, "google.com", "second")
    ).resolves.not.toThrow();
    await expect(pips.vote(dave, 1, true, 10)).resolves.not.toThrow();

    // GC needs some funds to use.
    await expect(
      distributePolyBatch(alice, [govCommittee1, govCommittee2], transferAmount)
    ).resolves.not.toThrow();

    // Snapshot and approve second PIP.
    await expect(pips.snapshot(govCommittee1)).resolves.not.toThrow();
    const approvePIP = await pips.enactSnapshotResults(
      secondPipCount,
      "Approve"
    );
    expect(approvePIP).toBeTruthy();
    await expect(
      pips.voteResult([govCommittee1, govCommittee2], approvePIP)
    ).resolves.not.toThrow();

    // Reject the first PIP
    const rejectPIP = await pips.rejectProposal(firstPipCount);
    expect(rejectPIP).toBeTruthy();
    await expect(
      pips.voteResult([govCommittee1, govCommittee2], rejectPIP)
    ).resolves.not.toThrow();

    // Finally reschedule, demonstrating that it had been scheduled.
    await expect(
      pips.rescheduleProposal(govCommittee1, secondPipCount)
    ).resolves.not.toThrow();
  });
});
