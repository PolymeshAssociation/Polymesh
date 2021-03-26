import { initMain, generateRandomKey, generateKeys } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";

async function main(): Promise<void> {
	const testEntities = await initMain();
	const alice = testEntities[0];
	const primaryDevSeed = generateRandomKey();
	const keys = await generateKeys(2, primaryDevSeed);
	await createIdentities(keys, alice);	
}

main().catch((err) => console.log(`Error: ${err.message}`)).finally(() => process.exit());
