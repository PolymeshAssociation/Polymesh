import * as init from "../util/init";
import { createIdentities, addClaimsToDids } from "../helpers/identity_helper";
import { distributePoly } from "../helpers/poly_helper";
import { issueTokenToDid, mintingAsset } from "../helpers/asset_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";

async function main(): Promise<void> {
	const ticker = init.generateRandomTicker();
	const testEntities = await init.initMain();
	const alice = testEntities[0];
	const primaryDevSeed = init.generateRandomKey();
	const primaryKey = (await init.generateKeys(1, primaryDevSeed))[0];
	let issuerDid = await createIdentities(alice, [primaryKey]);
	await distributePoly(alice, primaryKey, init.transferAmount);
	await issueTokenToDid(primaryKey, ticker, 1000000, null);
	await addClaimsToDids(primaryKey, issuerDid[0], "Exempted", { Ticker: ticker }, null);
	await addComplianceRequirement(primaryKey, ticker);
	await mintingAsset(primaryKey, ticker);
}

main()
	.catch((err: unknown) => {
		if (err instanceof Error) {
			console.log(`Error: ${err.message}`);
		}
	})
	.finally(() => process.exit());
