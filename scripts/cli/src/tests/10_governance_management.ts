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
	await pips.setDefaultEnactmentPeriod(alice, 10);
	await createIdentities(alice, [bob, dave]);

	// Bob and Dave needs some funds to use.
	await distributePolyBatch(alice, [bob, dave], transferAmount);
	await staking.bond(dave, bob, 1000000, "Staked");
	const setLimit = await pips.setActivePipLimit(42);

	const firstPipCount = await pips.pipIdSequence();
	await pips.propose(bob, setLimit, 9000000000, "google.com", "first");

	const secondPipCount = await pips.pipIdSequence();
	await pips.propose(bob, setLimit, 10000000000, "google.com", "second");

	// GC needs some funds to use.
	await distributePolyBatch(alice, [govCommittee1, govCommittee2], transferAmount);

	// Snapshot and approve second PIP.
	await pips.snapshot(govCommittee1);
	const approvePIP = await pips.enactSnapshotResults(secondPipCount, "Approve");
	await pips.voteResult([govCommittee1, govCommittee2], approvePIP);

	// Reject the first PIP
	const rejectPIP = await pips.rejectProposal(firstPipCount);
	await pips.voteResult([govCommittee1, govCommittee2], rejectPIP);

	// Finally reschedule, demonstrating that it had been scheduled.
	await pips.rescheduleProposal(alice, secondPipCount);
}

main()
	.catch((err: unknown) => {
		if (err instanceof Error) {
			console.log(`Error: ${err.message}`);
		}
	})
	.finally(() => process.exit());
