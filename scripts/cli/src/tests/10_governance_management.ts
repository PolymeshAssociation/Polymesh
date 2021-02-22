import { createApi, initMain, generateRandomEntity, transferAmount } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import * as staking from "../helpers/staking_helper";
import * as pips from "../helpers/pips_helper";

async function main(): Promise<void> {
	try {
		const api = await createApi();
		const testEntities = await initMain(api.api);
		const alice = testEntities[0];
		const govCommittee1 = testEntities[2];
		const govCommittee2 = testEntities[3];
		const bob = await generateRandomEntity(api.api);
		const dave = await generateRandomEntity(api.api);
		await pips.setDefaultEnactmentPeriod(api.api, 10, alice);
		await createIdentities(api.api, [bob, dave], alice);

		// Bob and Dave needs some funds to use.
		await distributePolyBatch(api.api, [bob, dave], transferAmount, alice);
		await staking.bond(api.api, bob, 1000000, "Staked", dave);
		const setLimit = pips.setActivePipLimit(api.api, 42);

		const firstPipCount = await pips.pipIdSequence(api.api);
		await pips.propose(api.api, setLimit, 9000000000, bob, "google.com", "first");

		const secondPipCount = await pips.pipIdSequence(api.api);
		await pips.propose(api.api, setLimit, 10000000000, bob, "google.com", "second");

		// GC needs some funds to use.
		await distributePolyBatch(api.api, [govCommittee1, govCommittee2], transferAmount, alice);

		// Snapshot and approve second PIP.
		await pips.snapshot(api.api, govCommittee1);
		const approvePIP = pips.enactSnapshotResults(api.api, secondPipCount, { Approve: "" });
		await pips.voteResult(api.api, approvePIP, [govCommittee1, govCommittee2]);

		// Reject the first PIP
		const rejectPIP = pips.rejectProposal(api.api, firstPipCount);
		await pips.voteResult(api.api, rejectPIP, [govCommittee1, govCommittee2]);

		// Finally reschedule, demonstrating that it had been scheduled.
		await pips.rescheduleProposal(api.api, secondPipCount, alice);

		await api.ws_provider.disconnect();
	} catch (err) {
		console.log(err);
	}
}

main();
