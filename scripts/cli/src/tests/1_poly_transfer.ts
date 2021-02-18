import { createApi, initMain, generateRandomKey, generateKeys, transferAmount } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";

async function main(): Promise<void> {
	try {
		const api = await createApi();
		const testEntities = await initMain(api.api);
		const alice = testEntities[0];
		const primaryDevSeed = await generateRandomKey();
		const keys = await generateKeys(api.api, 2, primaryDevSeed);
		await createIdentities(api.api, keys, alice);
		await distributePolyBatch(api.api, keys, transferAmount, alice);
		await api.ws_provider.disconnect();
	} catch (err) {
		console.log(err);
	}
}

main();