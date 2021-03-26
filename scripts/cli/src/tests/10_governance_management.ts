import { initMain, generateRandomEntity, transferAmount } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import * as staking from "../helpers/staking_helper";
import * as pips from "../helpers/pips_helper";

async function main(): Promise<void> {
		const testEntities = await initMain();
		const alice = testEntities[0];
		const govCommittee1 = testEntities[2];
		const govCommittee2 = testEntities[3];
		const bob = await generateRandomEntity();
		const dave = await generateRandomEntity();
		await pips.setDefaultEnactmentPeriod(10, alice);
		await createIdentities([bob, dave], alice);

		// Bob and Dave needs some funds to use.
		await distributePolyBatch([bob, dave], transferAmount, alice);
		await staking.bond(bob, 1000000, "Staked", dave);
		const setLimit = await pips.setActivePipLimit(42);

		const firstPipCount = await pips.pipIdSequence();
		await pips.propose(setLimit, 9000000000, bob, "google.com", "first");

		const secondPipCount = await pips.pipIdSequence();
		await pips.propose(setLimit, 10000000000, bob, "google.com", "second");

		// GC needs some funds to use.
		await distributePolyBatch([govCommittee1, govCommittee2], transferAmount, alice);

		// Snapshot and approve second PIP.
		await pips.snapshot(govCommittee1);
		const approvePIP = await pips.enactSnapshotResults(secondPipCount, "Approve");
		await pips.voteResult(approvePIP, [govCommittee1, govCommittee2]);

		// Reject the first PIP
		const rejectPIP = await pips.rejectProposal(firstPipCount);
		await pips.voteResult(rejectPIP, [govCommittee1, govCommittee2]);

		// Finally reschedule, demonstrating that it had been scheduled.
		await pips.rescheduleProposal(secondPipCount, alice);
}

main().catch((err) => console.log(`Error: ${err.message}`)).finally(() => process.exit());