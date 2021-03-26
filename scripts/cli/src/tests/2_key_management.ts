import * as init from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { addSecondaryKeys, createMultiSig } from "../helpers/key_management_helper";

async function main(): Promise<void> {
	const testEntities = await init.initMain();
	const primaryDevSeed = init.generateRandomKey();
	const secondaryDevSeed = init.generateRandomKey();
	const alice = testEntities[0];
	const bob = await init.generateRandomEntity();
	const charlie = await init.generateRandomEntity();
	const dave = await init.generateRandomEntity();
	const primaryKeys = await init.generateKeys(2, primaryDevSeed);
	const secondaryKeys = await init.generateKeys(2, secondaryDevSeed);
	const bobSignatory = await init.signatory(bob, alice);
	const charlieSignatory = await init.signatory(charlie, alice);
	const daveSignatory = await init.signatory(dave, alice);
	const signatoryArray = [bobSignatory, charlieSignatory, daveSignatory];
	await createIdentities(primaryKeys, alice);
	await distributePolyBatch(primaryKeys, init.transferAmount, alice);
	await addSecondaryKeys(secondaryKeys, primaryKeys);
	await createMultiSig(alice, signatoryArray, 2);
}

main().catch((err) => console.log(`Error: ${err.message}`)).finally(() => process.exit());
