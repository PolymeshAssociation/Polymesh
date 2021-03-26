import { createApi, initMain, generateRandomKey, generateKeys, transferAmount } from "../util/init";
import { createIdentities, addClaimsToDids } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";

async function main(): Promise<void> {
		const api = await createApi();
		const testEntities = await initMain(api.api);
		const alice = testEntities[0];
		const primaryDevSeed = await generateRandomKey();
		const claimDevSeed = await generateRandomKey();
		const primaryKeys = await generateKeys(api.api, 2, primaryDevSeed);
		const claimKeys = await generateKeys(api.api, 2, claimDevSeed);
		const issuerDids = await createIdentities(api.api, primaryKeys, alice);
		const claimIssuerDids = await createIdentities(api.api, claimKeys, alice);
		await distributePolyBatch(api.api, primaryKeys.concat(claimKeys), transferAmount, alice);
		await addClaimsToDids(api.api, claimKeys[0], issuerDids[0], "Exempted", { Identity: claimIssuerDids[1] }, null);
}

main().catch((err) => console.log(`Error: ${err.message}`)).finally(() => process.exit());