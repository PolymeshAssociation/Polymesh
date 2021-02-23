import { ApiPromise } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { sendTx, getDefaultPortfolio } from "../util/init";
import { IdentityId, Ticker } from "../types";

/**
 * @description Creates a Venue
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} sender - KeyringPair
 * @return {Promise<number>}
 */
export async function createVenue(api: ApiPromise, sender: KeyringPair): Promise<number> {
	let venueCounter = ((await api.query.settlement.venueCounter()) as unknown) as number;
	let venueDetails = "created venue";
	const transaction = api.tx.settlement.createVenue(venueDetails, [sender.address], 0);
	await sendTx(sender, transaction);
	return venueCounter;
}

/**
 * @description Adds an Instruction
 * @param {ApiPromise}  api - ApiPromise
 * @param {number} venueCounter - Venue Id
 * @param {KeyringPair} sender - KeyringPair
 * @param {IdentityId} senderDid - IdentityId
 * @param {IdentityId} receiverDid - IdentityId
 * @param {ticker} ticker - Ticker
 * @param {number} amount - Amount to be transferred
 * @return {Promise<number>}
 */
export async function addInstruction(
	api: ApiPromise,
	venueCounter: number,
	sender: KeyringPair,
	senderDid: IdentityId,
	receiverDid: IdentityId,
	ticker: Ticker,
	amount: number
): Promise<number> {
	let instructionCounter = ((await api.query.settlement.instructionCounter()) as unknown) as number;

	let leg = {
		from: getDefaultPortfolio(senderDid),
		to: getDefaultPortfolio(receiverDid),
		asset: ticker,
		amount: amount,
	};

	const transaction = api.tx.settlement.addInstruction(venueCounter, 0, null, null, [leg]);
	await sendTx(sender, transaction);

	return instructionCounter;
}

/**
 * @description Affirms an Instruction
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} sender - KeyringPair
 * @param {number} instructionCounter - Instruction Id
 * @param {IdentityId} did - IdentityId
 * @param {number} legCounter - Number of legs
 * @return {Promise<void>}
 */
export async function affirmInstruction(
	api: ApiPromise,
	sender: KeyringPair,
	instructionCounter: number,
	did: IdentityId,
	legCounter: number
): Promise<void> {
	const transaction = api.tx.settlement.affirmInstruction(instructionCounter, [getDefaultPortfolio(did)], legCounter);
	await sendTx(sender, transaction);
}

/**
 * @description Withdraws a Instruction
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} sender - KeyringPair
 * @param {number} instructionCounter - Instruction Id
 * @param {IdentityId} did - IdentityId
 * @return {Promise<void>}
 */
export async function withdrawInstruction(
	api: ApiPromise,
	sender: KeyringPair,
	instructionCounter: number,
	did: IdentityId
): Promise<void> {
	const transaction = api.tx.settlement.withdrawInstruction(instructionCounter, [getDefaultPortfolio(did)]);
	await sendTx(sender, transaction);
}

/**
 * @description Rejects a Instruction
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} sender - KeyringPair
 * @param {number} instructionCounter - Instruction Id
 * @param {IdentityId} did - IdentityId
 * @return {Promise<void>}
 */
export async function rejectInstruction(
	api: ApiPromise,
	sender: KeyringPair,
	instructionCounter: number,
	did: IdentityId
): Promise<void> {
	const transaction = api.tx.settlement.rejectInstruction(instructionCounter, [getDefaultPortfolio(did)]);
	await sendTx(sender, transaction);
}
