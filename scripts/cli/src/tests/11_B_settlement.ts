import * as init from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { assetBalance, issueTokenToDid, mintingAsset } from "../helpers/asset_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";
import * as settlement from "../helpers/settlement_helper";

async function main(): Promise<void> {
		const ticker = init.generateRandomTicker();
		const ticker2 = init.generateRandomTicker();
		const testEntities = await init.initMain();
		const alice = testEntities[0];
		const bob = await init.generateRandomEntity();
		const charlie = await init.generateRandomEntity();
		const dave = await init.generateRandomEntity();
		const eve = await init.generateRandomEntity();
		const aliceDid = await init.keyToIdentityIds(alice.publicKey);
		const bobDid = (await createIdentities([bob], alice))[0];
		const charlieDid = (await createIdentities([charlie], alice))[0];
		const daveDid = (await createIdentities([dave], alice))[0];
		const eveDid = (await createIdentities([eve], alice))[0];

		await distributePolyBatch([bob, charlie, dave, eve], init.transferAmount, alice);
		await issueTokenToDid(alice, ticker, 1000000, null);
		await issueTokenToDid(bob, ticker2, 1000000, null);
		await addComplianceRequirement(alice, ticker);
		await addComplianceRequirement(bob, ticker2);
		await mintingAsset(alice, ticker);
		await mintingAsset(bob, ticker2);

		let aliceBalance = await assetBalance(ticker, aliceDid);
		let bobBalance = await assetBalance(ticker, bobDid);
		let charlieBalance = await assetBalance(ticker, charlieDid);
		let daveBalance = await assetBalance(ticker, daveDid);
		let eveBalance = await assetBalance(ticker, eveDid);

		console.log("Balance for Alice Asset (Before)");
		console.log(`alice asset balance -------->  ${aliceBalance}`);
		console.log(`bob asset balance -------->  ${bobBalance}`);
		console.log(`charlie asset balance -------->  ${charlieBalance}`);
		console.log(`dave asset balance -------->  ${daveBalance}`);
		console.log(`eve asset balance -------->  ${eveBalance}`);
		console.log(" ");

		aliceBalance = await assetBalance(ticker2, aliceDid);
		bobBalance = await assetBalance(ticker2, bobDid);
		charlieBalance = await assetBalance(ticker2, charlieDid);
		daveBalance = await assetBalance(ticker2, daveDid);
		eveBalance = await assetBalance(ticker2, eveDid);

		console.log("Balance for Bob's Asset (Before)");
		console.log(`alice asset balance -------->  ${aliceBalance}`);
		console.log(`bob asset balance -------->  ${bobBalance}`);
		console.log(`charlie asset balance -------->  ${charlieBalance}`);
		console.log(`dave asset balance -------->  ${daveBalance}`);
		console.log(`eve asset balance -------->  ${eveBalance}`);
		console.log(" ");

		const venueCounter = await settlement.createVenue(alice);
		let instructionCounter = await settlement.addGroupInstruction(
						venueCounter,
			alice,
			[aliceDid, bobDid, charlieDid, daveDid, eveDid],
			ticker,
			ticker2,
			100
		);

		await settlement.affirmInstruction(alice, instructionCounter, aliceDid, 4);
		await settlement.affirmInstruction(bob, instructionCounter, bobDid, 1);
		await settlement.affirmInstruction(charlie, instructionCounter, charlieDid, 0);
		await settlement.affirmInstruction(dave, instructionCounter, daveDid, 0);
		//await settlement.rejectInstruction(eve, instructionCounter);
		await settlement.affirmInstruction(eve, instructionCounter, eveDid, 0);

		aliceBalance = await assetBalance(ticker, aliceDid);
		bobBalance = await assetBalance(ticker, bobDid);
		charlieBalance = await assetBalance(ticker, charlieDid);
		daveBalance = await assetBalance(ticker, daveDid);
		eveBalance = await assetBalance(ticker, eveDid);

		console.log("Balance for Alice Asset (After)");
		console.log(`alice asset balance -------->  ${aliceBalance}`);
		console.log(`bob asset balance -------->  ${bobBalance}`);
		console.log(`charlie asset balance -------->  ${charlieBalance}`);
		console.log(`dave asset balance -------->  ${daveBalance}`);
		console.log(`eve asset balance -------->  ${eveBalance}`);
		console.log(" ");

		aliceBalance = await assetBalance(ticker2, aliceDid);
		bobBalance = await assetBalance(ticker2, bobDid);
		charlieBalance = await assetBalance(ticker2, charlieDid);
		daveBalance = await assetBalance(ticker2, daveDid);
		eveBalance = await assetBalance(ticker2, eveDid);

		console.log("Balance for Bob's ASSET (After)");
		console.log(`alice asset balance -------->  ${aliceBalance}`);
		console.log(`bob asset balance -------->  ${bobBalance}`);
		console.log(`charlie asset balance -------->  ${charlieBalance}`);
		console.log(`dave asset balance -------->  ${daveBalance}`);
		console.log(`eve asset balance -------->  ${eveBalance}`);
}

main().catch((err) => console.log(`Error: ${err.message}`)).finally(() => process.exit());