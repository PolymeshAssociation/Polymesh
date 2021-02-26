import * as init from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { issueTokenToDid } from "../helpers/asset_helper";
import { createClaimCompliance } from "../helpers/compliance_manager_helper";

async function main(): Promise<void> {
	try {
		const api = await init.createApi();
		const ticker = await init.generateRandomTicker();
		const testEntities = await init.initMain(api.api);
		const alice = testEntities[0];
		const primaryDevSeed = await init.generateRandomKey();
		const primaryKeys = await init.generateKeys(api.api, 1, primaryDevSeed);
		let issuerDid = await createIdentities(api.api, primaryKeys, alice);
		await distributePolyBatch(api.api, primaryKeys, init.transferAmount, alice);
		await issueTokenToDid(api.api, primaryKeys[0], ticker, 1000000);
		await createClaimCompliance(api.api, primaryKeys[0], issuerDid[0], ticker);
		await api.ws_provider.disconnect();
	} catch (err) {
		console.log(err);
	}
}

main();