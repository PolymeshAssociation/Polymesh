import { createApi, initMain, generateRandomKey, generateKeys, transferAmount } from "../util/init";
import { createIdentities, authorizeJoinToIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { addSecondaryKeys } from "../helpers/key_management_helper";

async function main(): Promise<void> {
	try {
		const api = await createApi();
		const testEntities = await initMain(api.api);
		const alice = testEntities[0];
		const primaryDevSeed = await generateRandomKey();
		const secondaryDevSeed = await generateRandomKey();
		const primaryKeys = await generateKeys(api.api, 2, primaryDevSeed);
		const secondaryKeys = await generateKeys(api.api, 2, secondaryDevSeed);
		const issuerDids = await createIdentities(api.api, primaryKeys, alice);
		await distributePolyBatch(api.api, primaryKeys, transferAmount, alice);
		await addSecondaryKeys(api.api, secondaryKeys, primaryKeys);
		await authorizeJoinToIdentities(api.api, primaryKeys, issuerDids, secondaryKeys);
		await api.ws_provider.disconnect();
	} catch (err) {
		console.log(err);
	}
}

main();