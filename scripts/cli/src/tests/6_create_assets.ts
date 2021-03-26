import * as init from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { issueTokenToDid } from "../helpers/asset_helper";

async function main(): Promise<void> {
		const api = await init.createApi();
		const ticker = await init.generateRandomTicker();
		const testEntities = await init.initMain(api.api);
		const alice = testEntities[0];
		const primaryDevSeed = await init.generateRandomKey();
		const primaryKeys = await init.generateKeys(api.api, 1, primaryDevSeed);
		await createIdentities(api.api, primaryKeys, alice);
		await distributePolyBatch(api.api, primaryKeys, init.transferAmount, alice);
		await issueTokenToDid(api.api, primaryKeys[0], ticker, 1000000, null);
}

main().catch((err) => console.log(`Error: ${err.message}`)).finally(() => process.exit());