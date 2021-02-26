import * as init from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { addSecondaryKeys, createMultiSig } from "../helpers/key_management_helper";

async function main(): Promise<void> {
	try {
		const api = await init.createApi();
		const testEntities = await init.initMain(api.api);
		const alice = testEntities[0];
		const bob = await init.generateRandomEntity(api.api);
		const charlie = await init.generateRandomEntity(api.api);
		const dave = await init.generateRandomEntity(api.api);
		const primaryDevSeed = await init.generateRandomKey();
		const secondaryDevSeed = await init.generateRandomKey();
		const primaryKeys = await init.generateKeys(api.api, 2, primaryDevSeed);
		const secondaryKeys = await init.generateKeys(api.api, 2, secondaryDevSeed);
		await createIdentities(api.api, primaryKeys, alice);
		await distributePolyBatch(api.api, primaryKeys, init.transferAmount, alice);
		await addSecondaryKeys(api.api, secondaryKeys, primaryKeys);
		const bobSignatory = await init.signatory(api.api, bob, alice);
		const charlieSignatory = await init.signatory(api.api, charlie, alice);
		const daveSignatory = await init.signatory(api.api, dave, alice);
		const signatoryArray = [bobSignatory, charlieSignatory, daveSignatory];
		await createMultiSig(api.api, alice, signatoryArray, 2);
		await api.ws_provider.disconnect();
	} catch (err) {
		console.log(err);
	}
}

main();