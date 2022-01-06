import { initMain, generateEntityFromUri, transferAmount, keyToIdentityIds } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { claimItnReward, setItnRewardStatus } from "../helpers/rewards_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { createTable } from "../util/sqlite3";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
    createTable();
	const testEntities = await initMain();
	const alice = testEntities[0];
    const dave = await generateEntityFromUri("17_dave");
    const dave2 = await generateEntityFromUri("17_dave2");
    console.log("Create Identity");
    await createIdentities(alice, [dave2]);
    console.log("Set ITN Rewards Claim Status"); 
    await setItnRewardStatus(alice, dave, {Unclaimed: 2_000_000});
    console.log("Claim ITN Rewards");
	await claimItnReward(dave, dave2);
}

main()
	.catch((err: any) => {
		const pe = new PrettyError();
		console.error(pe.render(err));
		process.exit(1);
	})
	.finally(() => {
        console.log("Completed: ITN Rewards Test");
        process.exit();
    });