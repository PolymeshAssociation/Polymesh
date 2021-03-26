import { initMain, generateRandomKey, generateKeys, transferAmount } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";

async function main(): Promise<void> {
	const testEntities = await initMain();
	const alice = testEntities[0];
	const primaryDevSeed = generateRandomKey();
	const keys = await generateKeys(2, primaryDevSeed);
	await createIdentities(alice, keys);
	await distributePolyBatch(alice, keys, transferAmount);
}

main()
	.catch((err: unknown) => {
		if (err instanceof Error) {
			console.log(`Error: ${err.message}`);
		}
	})
	.finally(() => process.exit());
