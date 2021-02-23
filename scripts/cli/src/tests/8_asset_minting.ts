import {
	createApi,
	initMain,
	generateRandomKey,
	generateKeys,
	generateRandomTicker,
	transferAmount,
} from "../util/init";
import { createIdentities, addClaimsToDids } from "../helpers/identity_helper";
import { distributePoly } from "../helpers/poly_helper";
import { issueTokenToDid, mintingAsset } from "../helpers/asset_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";

async function main(): Promise<void> {
	try {
		const api = await createApi();
		const ticker = await generateRandomTicker();
		const testEntities = await initMain(api.api);
		const alice = testEntities[0];
		const primaryDevSeed = await generateRandomKey();
		const primaryKey = (await generateKeys(api.api, 1, primaryDevSeed))[0];
		let issuerDid = await createIdentities(api.api, [primaryKey], alice);
		await distributePoly(api.api, primaryKey, transferAmount, alice);
		await issueTokenToDid(api.api, primaryKey, ticker, 1000000);
		await addClaimsToDids(api.api, primaryKey, issuerDid[0], "Exempted", { Ticker: ticker });
		await addComplianceRequirement(api.api, primaryKey, ticker);
		await mintingAsset(api.api, primaryKey, ticker);

		await api.ws_provider.disconnect();
	} catch (err) {
		console.log(err);
	}
}

main();
