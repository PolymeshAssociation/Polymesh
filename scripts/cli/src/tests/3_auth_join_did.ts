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
		const issuerDids = await createIdentities(primaryKeys, alice);
		await distributePolyBatch(primaryKeys, transferAmount, alice);
		await addSecondaryKeys(secondaryKeys, primaryKeys);
		await authorizeJoinToIdentities(primaryKeys, issuerDids, secondaryKeys);
}

main().catch((err) => console.log(`Error: ${err.message}`)).finally(() => process.exit());