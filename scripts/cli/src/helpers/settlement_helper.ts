import type { KeyringPair } from "@polkadot/keyring/types";
import type { Ticker } from "../types";
import { sendTx, getDefaultPortfolio, ApiSingleton } from "../util/init";
import type { IdentityId } from '../interfaces';

/**
 * @description Creates a Venue
 * @param {KeyringPair} sender - KeyringPair
 * @return {Promise<number>}
 */
export async function createVenue(sender: KeyringPair): Promise<number> {
	const api = await ApiSingleton.getInstance();
	let venueCounter = ((await api.query.settlement.venueCounter()) as unknown) as number;
	let venueDetails = "created venue";
	const transaction = api.tx.settlement.createVenue(venueDetails, [sender.address], 0);
	await sendTx(sender, transaction);
	return venueCounter;
}

/**
 * @description Adds an Instruction
 * @param {number} venueCounter - Venue Id
 * @param {KeyringPair} sender - KeyringPair
 * @param {IdentityId} senderDid - IdentityId
 * @param {IdentityId} receiverDid - IdentityId
 * @param {ticker} ticker - Ticker
 * @param {number} amount - Amount to be transferred
 * @return {Promise<number>}
 */
export async function addInstruction(
	venueCounter: number,
	sender: KeyringPair,
	senderDid: IdentityId,
	receiverDid: IdentityId,
	ticker: Ticker,
	amount: number
): Promise<number> {
	const api = await ApiSingleton.getInstance();
	let instructionCounter = ((await api.query.settlement.instructionCounter()) as unknown) as number;

	let leg = {
		from: getDefaultPortfolio(senderDid),
		to: getDefaultPortfolio(receiverDid),
		asset: ticker,
		amount: amount,
	};

	const transaction = api.tx.settlement.addInstruction(venueCounter, {SettleOnAffirmation: ""}, null, null, [leg]);
	await sendTx(sender, transaction);

	return instructionCounter;
}

/**
 * @description Affirms an Instruction
 * @param {KeyringPair} sender - KeyringPair
 * @param {number} instructionCounter - Instruction Id
 * @param {IdentityId} did - IdentityId
 * @param {number} legCounter - Number of legs
 * @return {Promise<void>}
 */
export async function affirmInstruction(
	sender: KeyringPair,
	instructionCounter: number,
	did: IdentityId,
	legCounter: number
): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.settlement.affirmInstruction(instructionCounter, [getDefaultPortfolio(did)], legCounter);
	await sendTx(sender, transaction);
}

/**
 * @description Withdraws a Instruction
 * @param {KeyringPair} sender - KeyringPair
 * @param {number} instructionCounter - Instruction Id
 * @param {IdentityId} did - IdentityId
 * @return {Promise<void>}
 */
export async function withdrawInstruction(
	sender: KeyringPair,
	instructionCounter: number,
	did: IdentityId
): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.settlement.withdrawInstruction(instructionCounter, [getDefaultPortfolio(did)]);
	await sendTx(sender, transaction);
}

/**
 * @description Rejects a Instruction
 * @param {KeyringPair} sender - KeyringPair
 * @param {number} instructionCounter - Instruction Id
 * @param {IdentityId} did - IdentityId
 * @return {Promise<void>}
 */
export async function rejectInstruction(
	sender: KeyringPair,
	instructionCounter: number,
	did: IdentityId
): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.settlement.rejectInstruction(instructionCounter, [getDefaultPortfolio(did)], 5);
	await sendTx(sender, transaction);
}

/**
 * @description Adds a group Instruction
 * @param {number} venueCounter - number
 * @param {KeyringPair} sender - KeyringPair
 * @param {IdentityId[]} group - IdentityId[]
 * @param {Ticker} ticker - Ticker
 * @param {Ticker} ticker2 - Ticker
 * @param {number} amount - number
 * @return {Promise<number>}
 */
export async function addGroupInstruction(
	venueCounter: number,
	sender: KeyringPair,
	group: IdentityId[],
	ticker: Ticker,
	ticker2: Ticker,
	amount: number
): Promise<number> {
	const api = await ApiSingleton.getInstance();
	let instructionCounter = ((await api.query.settlement.instructionCounter()) as unknown) as number;
	let leg = {
		from: group[1],
		to: group[0],
		asset: ticker2,
		amount: amount,
	};

	let leg2 = {
		from: group[0],
		to: group[1],
		asset: ticker,
		amount: amount,
	};

	let leg3 = {
		from: group[0],
		to: group[2],
		asset: ticker,
		amount: amount,
	};

	let leg4 = {
		from: group[0],
		to: group[3],
		asset: ticker,
		amount: amount,
	};

	let leg5 = {
		from: group[0],
		to: group[4],
		asset: ticker,
		amount: amount,
	};

	const transaction = api.tx.settlement.addInstruction(venueCounter, 
		{SettleOnAffirmation: ""}, null, null, [leg, leg2, leg3, leg4, leg5]);

	await sendTx(sender, transaction);
	return instructionCounter;
}

/**
 * @description Creates a Claim Receipt
 * @param {KeyringPair} sender - KeyringPair
 * @param {IdentityId} sender_did - IdentityId
 * @param {IdentityId} receiver_did - IdentityId
 * @param {Ticker} ticker - Ticker
 * @param {Ticker} amount - number
 * @param {number} instructionCounter - number
 * @return {Promise<void>}
 */
async function claimReceipt(
	sender: KeyringPair,
	sender_did: IdentityId,
	receiver_did: IdentityId,
	ticker: Ticker,
	amount: number,
	instructionCounter: number
): Promise<void> {
	const api = await ApiSingleton.getInstance();
	
	let msg = {
		receipt_uid: 0,
		from: sender_did,
		to: receiver_did,
		asset: ticker,
		amount: amount,
	};

	let receiptDetails = {
		receipt_uid: 0,
		leg_id: 0,
		signer: sender.address,
		signature: 1,
	};

	const transaction = api.tx.settlement.claimReceipt(instructionCounter, receiptDetails);
	await sendTx(sender, transaction);
}
