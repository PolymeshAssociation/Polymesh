import * as init from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { issueTokenToDid } from "../helpers/asset_helper";
import { createClaimCompliance } from "../helpers/compliance_manager_helper";

async function main(): Promise<void> {
	const ticker = init.generateRandomTicker();
	const testEntities = await init.initMain();
	const alice = testEntities[0];
	const primaryDevSeed = init.generateRandomKey();
	const primaryKeys = await init.generateKeys(1, primaryDevSeed);
	let issuerDid = await createIdentities(alice, primaryKeys);
	await distributePolyBatch(alice, primaryKeys, init.transferAmount);
	await issueTokenToDid(primaryKeys[0], ticker, 1000000, null);
	await createClaimCompliance(primaryKeys[0], issuerDid[0], ticker);
}

main()
	.catch((err) => console.log(`Error: ${err.message}`))
	.finally(() => process.exit());
