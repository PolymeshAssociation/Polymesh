import { initMain, generateRandomEntity, transferAmount } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { claimItnReward, setItnRewardStatus } from "../helpers/rewards_helper";
import { distributePoly } from "../helpers/poly_helper";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
	const testEntities = await initMain();
	const alice = testEntities[0];
	const bob = await generateRandomEntity();
	await createIdentities(alice, [bob]);
	await distributePoly(alice, bob, transferAmount);
	await claimItnReward(alice, bob.publicKey);
}

main()
	.catch((err: any) => {
		const pe = new PrettyError();
		console.error(pe.render(err));
		process.exit(1);
	})
	.finally(() => process.exit());
