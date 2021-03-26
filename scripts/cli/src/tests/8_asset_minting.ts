import * as init from "../util/init";
import { createIdentities, addClaimsToDids } from "../helpers/identity_helper";
import { distributePoly } from "../helpers/poly_helper";
import { issueTokenToDid, mintingAsset } from "../helpers/asset_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";

async function main(): Promise<void> {
		const api = await init.createApi();
		const ticker = await init.generateRandomTicker();
		const testEntities = await init.initMain(api.api);
		const alice = testEntities[0];
		const primaryDevSeed = await init.generateRandomKey();
		const primaryKey = (await init.generateKeys(api.api, 1, primaryDevSeed))[0];
		let issuerDid = await createIdentities(api.api, [primaryKey], alice);
		await distributePoly(api.api, primaryKey, init.transferAmount, alice);
		await issueTokenToDid(api.api, primaryKey, ticker, 1000000, null);
		await addClaimsToDids(api.api, primaryKey, issuerDid[0], "Exempted", { Ticker: ticker }, null);
		await addComplianceRequirement(api.api, primaryKey, ticker);
		await mintingAsset(api.api, primaryKey, ticker);
}

main().catch((err) => console.log(`Error: ${err.message}`)).finally(() => process.exit());