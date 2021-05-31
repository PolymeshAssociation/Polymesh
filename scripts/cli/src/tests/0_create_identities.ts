import { initMain, generateRandomKey, generateKeys } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";

async function main(): Promise<void> {
	const testEntities = await initMain();
	const alice = testEntities[0];
	const primaryDevSeed = generateRandomKey();
	const keys = await generateKeys(2, primaryDevSeed);
	await createIdentities(alice, keys);
}

main()
	.catch((err: unknown) => {
		if (err instanceof Error) {
			console.log(`Error: ${err.message}`);
		}
	})
	.finally(() => process.exit());
