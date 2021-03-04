import { createApi, initMain, generateRandomKey, generateKeys, handle } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";

async function main(): Promise<void> {
		const [api, apiErr] = await handle(createApi());
		if (apiErr) throw new Error("Failed to create Api");
		const [testEntities, testEntitiesErr] = await handle(initMain(api.api));
		if (testEntitiesErr) throw new Error("Failed to get test entities");
		const alice = testEntities[0];
		const primaryDevSeed = generateRandomKey();
		const [keys, keysErr] = await handle(generateKeys(api.api, 2, primaryDevSeed));
		if (keysErr) throw new Error("Failed to create keys");
		const [dids, didsErr] = await handle(createIdentities(api.api, keys, alice));
		if (didsErr) throw new Error("Failed to create identities");
		await api.ws_provider.disconnect().catch(err => console.log(`Error: ${err.message}`));
}

main().catch(err => console.log(`Error: ${err.message}`));