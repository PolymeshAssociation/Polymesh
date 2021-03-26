import * as init from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePoly } from "../helpers/poly_helper";
import { assetBalance, issueTokenToDid } from "../helpers/asset_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";
import * as settlement from "../helpers/settlement_helper";

async function main(): Promise<void> {
	const ticker = init.generateRandomTicker();
	const testEntities = await init.initMain();
	const alice = testEntities[0];
	const bob = await init.generateRandomEntity();
	const bobDid = (await createIdentities(alice, [bob]))[0];
	const aliceDid = await init.keyToIdentityIds(alice.publicKey);
	await distributePoly(alice, bob, init.transferAmount);
	await issueTokenToDid(alice, ticker, 1000000, null);
	await addComplianceRequirement(alice, ticker);

	let aliceBalance = await assetBalance(ticker, aliceDid);
	let bobBalance = await assetBalance(ticker, bobDid);

	console.log("Balance for Asset (Before)");
	console.log(`alice asset balance -------->  ${aliceBalance}`);
	console.log(`bob asset balance -------->  ${bobBalance}`);
	console.log(" ");

	let venueCounter = await settlement.createVenue(alice);

	let intructionCounterAB = await settlement.addInstruction(alice, venueCounter, aliceDid, bobDid, ticker, 100);

	await settlement.affirmInstruction(alice, intructionCounterAB, aliceDid, 1);
	await settlement.affirmInstruction(bob, intructionCounterAB, bobDid, 0);

	//await rejectInstruction(bob, intructionCounter);
	//await unathorizeInstruction(alice, instructionCounter);

	aliceBalance = await assetBalance(ticker, aliceDid);
	bobBalance = await assetBalance(ticker, bobDid);

	console.log(`alice asset balance -------->  ${aliceBalance}`);
	console.log(`bob asset balance -------->  ${bobBalance}`);
}

main()
	.catch((err: unknown) => {
		if (err instanceof Error) {
			console.log(`Error: ${err.message}`);
		}
	})
	.finally(() => process.exit());
