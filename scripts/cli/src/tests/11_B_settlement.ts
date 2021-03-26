import * as init from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { assetBalance, issueTokenToDid, mintingAsset } from "../helpers/asset_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";
import * as settlement from "../helpers/settlement_helper";

async function main(): Promise<void> {
		const api = await init.createApi();
		const ticker = await init.generateRandomTicker();
		const ticker2 = await init.generateRandomTicker();
		const testEntities = await init.initMain(api.api);
		const alice = testEntities[0];
		const bob = await init.generateRandomEntity(api.api);
		const charlie = await init.generateRandomEntity(api.api);
		const dave = await init.generateRandomEntity(api.api);
		const eve = await init.generateRandomEntity(api.api);
		const aliceDid = await init.keyToIdentityIds(api.api, alice.publicKey);
		const bobDid = (await createIdentities(api.api, [bob], alice))[0];
		const charlieDid = (await createIdentities(api.api, [charlie], alice))[0];
		const daveDid = (await createIdentities(api.api, [dave], alice))[0];
		const eveDid = (await createIdentities(api.api, [eve], alice))[0];

		await distributePolyBatch(api.api, [bob, charlie, dave, eve], init.transferAmount, alice);
		await issueTokenToDid(api.api, alice, ticker, 1000000, null);
		await issueTokenToDid(api.api, bob, ticker2, 1000000, null);
		await addComplianceRequirement(api.api, alice, ticker);
		await addComplianceRequirement(api.api, bob, ticker2);
		await mintingAsset(api.api, alice, ticker);
		await mintingAsset(api.api, bob, ticker2);

		let aliceBalance = await assetBalance(api.api, ticker, aliceDid);
		let bobBalance = await assetBalance(api.api, ticker, bobDid);
		let charlieBalance = await assetBalance(api.api, ticker, charlieDid);
		let daveBalance = await assetBalance(api.api, ticker, daveDid);
		let eveBalance = await assetBalance(api.api, ticker, eveDid);

		console.log("Balance for Alice Asset (Before)");
		console.log(`alice asset balance -------->  ${aliceBalance}`);
		console.log(`bob asset balance -------->  ${bobBalance}`);
		console.log(`charlie asset balance -------->  ${charlieBalance}`);
		console.log(`dave asset balance -------->  ${daveBalance}`);
		console.log(`eve asset balance -------->  ${eveBalance}`);
		console.log(" ");

		aliceBalance = await assetBalance(api.api, ticker2, aliceDid);
		bobBalance = await assetBalance(api.api, ticker2, bobDid);
		charlieBalance = await assetBalance(api.api, ticker2, charlieDid);
		daveBalance = await assetBalance(api.api, ticker2, daveDid);
		eveBalance = await assetBalance(api.api, ticker2, eveDid);

		console.log("Balance for Bob's Asset (Before)");
		console.log(`alice asset balance -------->  ${aliceBalance}`);
		console.log(`bob asset balance -------->  ${bobBalance}`);
		console.log(`charlie asset balance -------->  ${charlieBalance}`);
		console.log(`dave asset balance -------->  ${daveBalance}`);
		console.log(`eve asset balance -------->  ${eveBalance}`);
		console.log(" ");

		const venueCounter = await settlement.createVenue(api.api, alice);
		let instructionCounter = await settlement.addGroupInstruction(
			api.api,
			venueCounter,
			alice,
			[aliceDid, bobDid, charlieDid, daveDid, eveDid],
			ticker,
			ticker2,
			100
		);

		await settlement.affirmInstruction(api.api, alice, instructionCounter, aliceDid, 4);
		await settlement.affirmInstruction(api.api, bob, instructionCounter, bobDid, 1);
		await settlement.affirmInstruction(api.api, charlie, instructionCounter, charlieDid, 0);
		await settlement.affirmInstruction(api.api, dave, instructionCounter, daveDid, 0);
		//await settlement.rejectInstruction(api.api, eve, instructionCounter);
		await settlement.affirmInstruction(api.api, eve, instructionCounter, eveDid, 0);

		aliceBalance = await assetBalance(api.api, ticker, aliceDid);
		bobBalance = await assetBalance(api.api, ticker, bobDid);
		charlieBalance = await assetBalance(api.api, ticker, charlieDid);
		daveBalance = await assetBalance(api.api, ticker, daveDid);
		eveBalance = await assetBalance(api.api, ticker, eveDid);

		console.log("Balance for Alice Asset (After)");
		console.log(`alice asset balance -------->  ${aliceBalance}`);
		console.log(`bob asset balance -------->  ${bobBalance}`);
		console.log(`charlie asset balance -------->  ${charlieBalance}`);
		console.log(`dave asset balance -------->  ${daveBalance}`);
		console.log(`eve asset balance -------->  ${eveBalance}`);
		console.log(" ");

		aliceBalance = await assetBalance(api.api, ticker2, aliceDid);
		bobBalance = await assetBalance(api.api, ticker2, bobDid);
		charlieBalance = await assetBalance(api.api, ticker2, charlieDid);
		daveBalance = await assetBalance(api.api, ticker2, daveDid);
		eveBalance = await assetBalance(api.api, ticker2, eveDid);

		console.log("Balance for Bob's ASSET (After)");
		console.log(`alice asset balance -------->  ${aliceBalance}`);
		console.log(`bob asset balance -------->  ${bobBalance}`);
		console.log(`charlie asset balance -------->  ${charlieBalance}`);
		console.log(`dave asset balance -------->  ${daveBalance}`);
		console.log(`eve asset balance -------->  ${eveBalance}`);
}

main().catch((err) => console.log(`Error: ${err.message}`)).finally(() => process.exit());