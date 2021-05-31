import * as init from "../util/init";
import { authorizeJoinToIdentities, createIdentitiesWithExpiry } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { addSecondaryKeys } from "../helpers/key_management_helper";
import { addNominator } from "../helpers/staking_helper";

async function main(): Promise<void> {
	const testEntities = await init.initMain();
	const alice = testEntities[0];
	const controllers = ["nom1", "nom2", "nom3", "nom4", "nom5"];
    // And an signer key to the keystore to sign the offchain worker data
    await init.generateOffchainKeys("cddw"); // Off-chain worker Key type
    // Create stash accounts for the nominators
    const stash_nominators = await init.generateStashKeys(controllers);
    // Create keys of validators
    const validators_key = await init.generateKeys(2, "validator"); 
    // Get the key of cdd provider
    let provider = await init.getValidCddProvider(alice);
    // Provide the DID to the stash_nominators
    // Calculate the bonding duration
    let expiryTime = new Uint8Array(await init.getExpiries(stash_nominators.length));
    await createIdentitiesWithExpiry(provider, stash_nominators, [expiryTime]);
    // fund stash keys
    await distributePolyBatch(alice, stash_nominators, init.transferAmount);
    // get controller key for nominators
    const controller_keys = await init.generateEntities(controllers);
    // Add as secondary key to the existing did who wants to be a potential nominator
    await addSecondaryKeys(stash_nominators, controller_keys);
    await authorizeJoinToIdentities(stash_nominators, controller_keys);
    await addNominator(controller_keys, stash_nominators, alice, validators_key);
    init.subscribeCddOffchainWorker(); 
}

main()
	.catch((err: unknown) => {
		if (err instanceof Error) {
			console.log(`Error: ${err.message}`);
		}
	})
	.finally(() => process.exit());
