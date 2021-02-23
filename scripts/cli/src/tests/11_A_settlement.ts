import * as init from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePoly } from "../helpers/poly_helper";
import { assetBalance, issueTokenToDid } from "../helpers/asset_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";
import * as settlement from "../helpers/settlement_helper";

async function main(): Promise<void> {
	try {
		const api = await init.createApi();
		const ticker = await init.generateRandomTicker();
		const testEntities = await init.initMain(api.api);
		const alice = testEntities[0];
		const bob = await init.generateRandomEntity(api.api);
		const bobDid = (await createIdentities(api.api, [bob], alice))[0];
		const aliceDid = await init.keyToIdentityIds(api.api, alice.publicKey);
		await distributePoly(api.api, bob, init.transferAmount, alice);
		await issueTokenToDid(api.api, alice, ticker, 1000000);
		await addComplianceRequirement(api.api, alice, ticker);

		let aliceBalance = await assetBalance(api.api, ticker, aliceDid);
		let bobBalance = await assetBalance(api.api, ticker, bobDid);

		console.log("Balance for Asset (Before)");
		console.log(`alice asset balance -------->  ${aliceBalance}`);
		console.log(`bob asset balance -------->  ${bobBalance}`);
		console.log(" ");

		let venueCounter = await settlement.createVenue(api.api, alice);

		let intructionCounterAB = await settlement.addInstruction(
			api.api,
			venueCounter,
			alice,
			aliceDid,
			bobDid,
			ticker,
			100
		);

		await settlement.affirmInstruction(api.api, alice, intructionCounterAB, aliceDid, 1);
		await settlement.affirmInstruction(api.api, bob, intructionCounterAB, bobDid, 0);

		//await rejectInstruction(api, bob, intructionCounter);
		//await unathorizeInstruction(api, alice, instructionCounter);

		aliceBalance = await assetBalance(api.api, ticker, aliceDid);
		bobBalance = await assetBalance(api.api, ticker, bobDid);

		console.log(`alice asset balance -------->  ${aliceBalance}`);
		console.log(`bob asset balance -------->  ${bobBalance}`);

		await api.ws_provider.disconnect();
	} catch (err) {
		console.log(err);
	}
}

main();