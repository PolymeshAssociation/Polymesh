import type { KeyringPair } from "@polkadot/keyring/types";
import type { SubmittableExtrinsic } from "@polkadot/api/types";
import type { ISubmittableResult } from "@polkadot/types/types/extrinsic";
import { sendTx, ApiSingleton } from "../util/init";

/**
 * @description Sets the default enactment period for a PIP
 * @param {KeyringPair} signer - KeyringPair
 * @param {number} duration - Blocknumber
 * @return {Promise<void>}
 */
export async function setDefaultEnactmentPeriod(signer: KeyringPair, duration: number): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.sudo.sudo(api.tx.pips.setDefaultEnactmentPeriod(duration));
	await sendTx(signer, transaction).catch((err) => console.log(`Error: ${err.message}`));
}

/**
 * @description Returns Id to keep track of PIPs
 * @return {Promise<number>}
 */
export async function pipIdSequence(): Promise<number> {
	const api = await ApiSingleton.getInstance();
	return (await api.query.pips.pipIdSequence()).toNumber();
}

/**
 * @description Sets active PIP limit
 * @param {number} pipLimit - number
 * @return {Promise<SubmittableExtrinsic<"promise", ISubmittableResult>>}
 */
export async function setActivePipLimit(
	pipLimit: number
): Promise<SubmittableExtrinsic<"promise", ISubmittableResult>> {
	const api = await ApiSingleton.getInstance();
	return api.tx.pips.setActivePipLimit(pipLimit);
}

/**
 * @description Creates a proposal
 * @param {KeyringPair} signer - KeyringPair
 * @param {number} proposal
 * @param {number} deposit - number
 * @param {string=} url - Proposal URL link
 * @param {string=} description - Proposal description
 * @return {Promise<void>}
 */
export async function propose(
	signer: KeyringPair,
	proposal: SubmittableExtrinsic<"promise", ISubmittableResult>,
	deposit: number,
	url: string | null,
	description: string | null
): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.pips.propose(proposal, deposit, url, description);
	await sendTx(signer, transaction).catch((err) => console.log(`Error: ${err.message}`));
}

/**
 * @description Create a Snapshot
 * @param {KeyringPair} signer - KeyringPair
 * @return {Promise<void>}
 */
export async function snapshot(signer: KeyringPair): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.pips.snapshot();
	await sendTx(signer, transaction).catch((err) => console.log(`Error: ${err.message}`));
}

/**
 * @description Enacts `snapshotResult` for the PIPs in the snapshot queue.
 * @param {number} pipId - number
 * @param {object} snapshotResult
 * @return {Promise<SubmittableExtrinsic<"promise", ISubmittableResult>>}
 */
export async function enactSnapshotResults(
	pipId: number,
	snapshotResult: number | Uint8Array | "Approve" | "Reject" | "Skip"
): Promise<SubmittableExtrinsic<"promise", ISubmittableResult>> {
	const api = await ApiSingleton.getInstance();
	return api.tx.pips.enactSnapshotResults([[pipId, snapshotResult]]);
}

/**
 * @description Reject the proposal
 * @param {number} pipId - number
 * @return {Promise<SubmittableExtrinsic<"promise", ISubmittableResult>>}
 */
export async function rejectProposal(pipId: number): Promise<SubmittableExtrinsic<"promise", ISubmittableResult>> {
	const api = await ApiSingleton.getInstance();
	return api.tx.pips.rejectProposal(pipId);
}

/**
 * @description Reschedules a proposal
 * @param {KeyringPair} signer - KeyringPair
 * @param {number} pipId - number
 * @return {Promise<void>}
 */
export async function rescheduleProposal(signer: KeyringPair, pipId: number): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.pips.rescheduleExecution(pipId, null);
	await sendTx(signer, transaction).catch((err) => console.log(`Error: ${err.message}`));
}

/**
 * @description Vote Result
 * @param {KeyringPair[]} signers - KeyringPair[]
 * @param {SubmittableExtrinsic<"promise", ISubmittableResult>} tx - SubmittableExtrinsic
 * @return {Promise<void>}
 */
export async function voteResult(
	signers: KeyringPair[],
	tx: SubmittableExtrinsic<"promise", ISubmittableResult>
): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const vote = api.tx.polymeshCommittee.voteOrPropose(true, tx);
	for (let signer of signers) {
		await sendTx(signer, vote).catch((err) => console.log(`Error: ${err.message}`));
	}
}
