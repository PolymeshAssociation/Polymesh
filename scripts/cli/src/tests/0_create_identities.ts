import { createApi, initMain, generateRandomKey, generateKeys } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";

async function main(): Promise<void> {
	const api = await createApi();
	const testEntities = await initMain(api.api);
	const alice = testEntities[0];
	const primaryDevSeed = generateRandomKey();
	const keys = await generateKeys(api.api, 2, primaryDevSeed);
	await createIdentities(api.api, keys, alice);	
}

main().catch((err) => console.log(`Error: ${err.message}`)).finally(() => process.exit());
