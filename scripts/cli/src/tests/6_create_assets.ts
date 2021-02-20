import {
	createApi,
	initMain,
	generateRandomKey,
	generateKeys,
	generateRandomTicker,
	transferAmount,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { issueTokenToDid } from "../helpers/asset_helper";

async function main(): Promise<void> {
	try {
		const api = await createApi();
		const ticker = await generateRandomTicker();
		const testEntities = await initMain(api.api);
		const alice = testEntities[0];
		const primaryDevSeed = await generateRandomKey();
		const primaryKeys = await generateKeys(api.api, 1, primaryDevSeed);
		await createIdentities(api.api, primaryKeys, alice);
		await distributePolyBatch(api.api, primaryKeys, transferAmount, alice);
		await issueTokenToDid(api.api, primaryKeys[0], ticker, 1000000);
		await api.ws_provider.disconnect();
	} catch (err) {
		console.log(err);
	}
}

main();