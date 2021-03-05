import { createApi, initMain, generateRandomKey, generateKeys, handle } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";

async function main(): Promise<void> {
	const [apiErr, api] = await handle(createApi());
	if (apiErr) throw new Error("Failed to create Api");

	const [testEntitiesErr, testEntities] = await handle(initMain(api.api));
	if (testEntitiesErr) throw new Error("Failed to get test entities");

	const alice = testEntities[0];
	const primaryDevSeed = generateRandomKey();

	const [keysErr, keys] = await handle(generateKeys(api.api, 2, primaryDevSeed));
	if (keysErr) throw new Error("Failed to create keys");

	const [didsErr] = await handle(createIdentities(api.api, keys, alice));
	if (didsErr) throw new Error("Failed to create identities");

	const [disconnectErr] = await handle(api.ws_provider.disconnect());
	if (disconnectErr) throw new Error("Failed to disconnect");
}

main().catch((err) => console.log(`Error: ${err.message}`));
