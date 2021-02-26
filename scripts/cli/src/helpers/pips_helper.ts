import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@polkadot/keyring/types";
import type { SubmittableExtrinsic } from "@polkadot/api/types";
import type { ISubmittableResult } from "@polkadot/types/types";
import { sendTx } from "../util/init";

/**
 * @description Sets the default enactment period for a PIP
 * @param {ApiPromise}  api - ApiPromise
 * @param {number} duration - Blocknumber
 * @param {KeyringPair} signer - KeyringPair
 * @return {Promise<void>}
 */
export async function setDefaultEnactmentPeriod(api: ApiPromise, duration: number, signer: KeyringPair): Promise<void> {
	const transaction = api.tx.sudo.sudo(api.tx.pips.setDefaultEnactmentPeriod(duration));
	await sendTx(signer, transaction);
}

/**
 * @description Returns Id to keep track of PIPs
 * @param {ApiPromise}  api - ApiPromise
 * @return {Promise<number>}
 */
export async function pipIdSequence(api: ApiPromise): Promise<number> {
	return ((await api.query.pips.pipIdSequence()) as unknown) as number;
}

/**
 * @description Sets active PIP limit
 * @param {ApiPromise}  api - ApiPromise
 * @param {number} pipLimit - number
 * @return {SubmittableExtrinsic<"promise", ISubmittableResult>}
 */
export function setActivePipLimit(
	api: ApiPromise,
	pipLimit: number
): SubmittableExtrinsic<"promise", ISubmittableResult> {
	return api.tx.pips.setActivePipLimit(pipLimit);
}

/**
 * @description Creates a proposal
 * @param {ApiPromise}  api - ApiPromise
 * @param {number} proposal
 * @param {number} deposit - number
 * @param {KeyringPair} signer - KeyringPair
 * @param {string=} url - Proposal URL link
 * @param {string=} description - Proposal description
 * @return {Promise<void>}
 */
export async function propose(
	api: ApiPromise,
	proposal: SubmittableExtrinsic<"promise", ISubmittableResult>,
	deposit: number,
	signer: KeyringPair,
	url?: string,
	description?: string
): Promise<void> {
	const transaction = api.tx.pips.propose(proposal, deposit, url, description);
	await sendTx(signer, transaction);
}

/**
 * @description Create a Snapshot
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} signer - KeyringPair
 * @return {Promise<void>}
 */
export async function snapshot(api: ApiPromise, signer: KeyringPair): Promise<void> {
	const transaction = api.tx.pips.snapshot();
	await sendTx(signer, transaction);
}

/**
 * @description Enacts `snapshotResult` for the PIPs in the snapshot queue.
 * @param {ApiPromise}  api - ApiPromise
 * @param {number} pipId - number
 * @param {object} snapshotResult
 * @return {SubmittableExtrinsic<"promise", ISubmittableResult>}
 */
export function enactSnapshotResults(
	api: ApiPromise,
	pipId: number,
	snapshotResult: object
): SubmittableExtrinsic<"promise", ISubmittableResult> {
	return api.tx.pips.enactSnapshotResults([[pipId, snapshotResult]]);
}

/**
 * @description Reject the proposal
 * @param {ApiPromise}  api - ApiPromise
 * @param {number} pipId - number
 * @return {SubmittableExtrinsic<"promise", ISubmittableResult>}
 */
export function rejectProposal(api: ApiPromise, pipId: number): SubmittableExtrinsic<"promise", ISubmittableResult> {
	return api.tx.pips.rejectProposal(pipId);
}

/**
 * @description Reschedules a proposal
 * @param {ApiPromise}  api - ApiPromise
 * @param {number} pipId - number
 * @param {KeyringPair} signer - KeyringPair
 * @return {Promise<void>}
 */
export async function rescheduleProposal(api: ApiPromise, pipId: number, signer: KeyringPair): Promise<void> {
	const transaction = api.tx.pips.rescheduleExecution(pipId, null);
	await sendTx(signer, transaction);
}

/**
 * @description Vote Result
 * @param {ApiPromise}  api - ApiPromise
 * @param {SubmittableExtrinsic<"promise", ISubmittableResult>} tx - SubmittableExtrinsic
 * @param {KeyringPair[]} signers - KeyringPair[]
 * @return {Promise<void>}
 */
export async function voteResult(
	api: ApiPromise,
	tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
	signers: KeyringPair[]
): Promise<void> {
	const vote = api.tx.polymeshCommittee.voteOrPropose(true, tx);
	for (let signer of signers) {
		await sendTx(signer, vote);
	}
}
