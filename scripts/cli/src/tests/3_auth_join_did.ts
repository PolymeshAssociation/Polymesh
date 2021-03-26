import { initMain, generateRandomKey, generateKeys, transferAmount } from "../util/init";
import { createIdentities, authorizeJoinToIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { addSecondaryKeys } from "../helpers/key_management_helper";

async function main(): Promise<void> {
	const testEntities = await initMain();
	const alice = testEntities[0];
	const primaryDevSeed = generateRandomKey();
	const secondaryDevSeed = generateRandomKey();
	const primaryKeys = await generateKeys(2, primaryDevSeed);
	const secondaryKeys = await generateKeys(2, secondaryDevSeed);
	await createIdentities(alice, primaryKeys);
	await distributePolyBatch(alice, primaryKeys, transferAmount);
	await addSecondaryKeys(primaryKeys, secondaryKeys);
	await authorizeJoinToIdentities(secondaryKeys, primaryKeys);
}

main()
	.catch((err: unknown) => {
		if (err instanceof Error) {
			console.log(`Error: ${err.message}`);
		}
	})
	.finally(() => process.exit());
