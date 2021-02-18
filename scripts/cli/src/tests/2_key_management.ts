import {
	createApi,
	initMain,
	generateRandomEntity,
	generateRandomKey,
	generateKeys,
	transferAmount,
	signatory,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { addSecondaryKeys, createMultiSig } from "../helpers/key_management_helper";

async function main(): Promise<void> {
	try {
		const api = await createApi();
		const testEntities = await initMain(api.api);
		const alice = testEntities[0];
		const bob = await generateRandomEntity(api.api);
		const charlie = await generateRandomEntity(api.api);
		const dave = await generateRandomEntity(api.api);
		const primaryDevSeed = await generateRandomKey();
		const secondaryDevSeed = await generateRandomKey();
		const primaryKeys = await generateKeys(api.api, 2, primaryDevSeed);
		const secondaryKeys = await generateKeys(api.api, 2, secondaryDevSeed);
		await createIdentities(api.api, primaryKeys, alice);
		await distributePolyBatch(api.api, primaryKeys, transferAmount, alice);
		await addSecondaryKeys(api.api, secondaryKeys, primaryKeys);
		const bobSignatory = await signatory(api.api, bob, alice);
		const charlieSignatory = await signatory(api.api, charlie, alice);
		const daveSignatory = await signatory(api.api, dave, alice);
		const signatoryArray = [bobSignatory, charlieSignatory, daveSignatory];
		await createMultiSig(api.api, alice, signatoryArray, 2);
		await api.ws_provider.disconnect();
	} catch (err) {
		console.log(err);
	}
}

main();