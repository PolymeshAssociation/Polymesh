import * as init from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { addSecondaryKeys, createMultiSig } from "../helpers/key_management_helper";

async function main(): Promise<void> {
	const [apiErr, api] = await init.handle(init.createApi());
	if (apiErr) throw new Error("Failed to create Api");

	const [testEntitiesErr, testEntities] = await init.handle(init.initMain(api.api));
	if (testEntitiesErr) throw new Error("Failed to get test entities");

	const primaryDevSeed = init.generateRandomKey();
	const secondaryDevSeed = init.generateRandomKey();
	const alice = testEntities[0];

	const [bobErr, bob] = await init.handle(init.generateRandomEntity(api.api));
	if (bobErr) throw new Error("Failed to create random entity");
	const [charlieErr, charlie] = await init.handle(init.generateRandomEntity(api.api));
	if (charlieErr) throw new Error("Failed to create random entity");
	const [daveErr, dave] = await init.handle(init.generateRandomEntity(api.api));
	if (daveErr) throw new Error("Failed to create random entity");

	const [primaryKeysErr, primaryKeys] = await init.handle(init.generateKeys(api.api, 2, primaryDevSeed));
	if (primaryKeysErr) throw new Error("Failed to create keys");
	const [secondaryKeysErr, secondaryKeys] = await init.handle(init.generateKeys(api.api, 2, secondaryDevSeed));
	if (secondaryKeysErr) throw new Error("Failed to create keys");

	const [didsErr] = await init.handle(createIdentities(api.api, primaryKeys, alice));
	if (didsErr) throw new Error("Failed to create identities");
	const [polyBatchErr] = await init.handle(distributePolyBatch(api.api, primaryKeys, init.transferAmount, alice));
	if (polyBatchErr) throw new Error("Failed to distribute poly");
	const [addSecKeyErr] = await init.handle(addSecondaryKeys(api.api, secondaryKeys, primaryKeys));
	if (addSecKeyErr) throw new Error("Failed to add secondary key");

	const [bobSignatoryErr, bobSignatory] = await init.handle(init.signatory(api.api, bob, alice));
	if (bobSignatoryErr) throw new Error("Failed to create signatory");
	const [charlieSignatoryErr, charlieSignatory] = await init.handle(init.signatory(api.api, charlie, alice));
	if (charlieSignatoryErr) throw new Error("Failed to create signatory");
	const [daveSignatoryErr, daveSignatory] = await init.handle(init.signatory(api.api, dave, alice));
	if (daveSignatoryErr) throw new Error("Failed to create signatory");

	const signatoryArray = [bobSignatory, charlieSignatory, daveSignatory];
	const [multisigErr] = await init.handle(createMultiSig(api.api, alice, signatoryArray, 2));
	if (multisigErr) throw new Error("Failed to create multisig");
	const [disconnectErr] = await init.handle(api.ws_provider.disconnect());
	if (disconnectErr) throw new Error("Failed to disconnect");
}

main().catch((err) => console.log(`Error: ${err.message}`));
