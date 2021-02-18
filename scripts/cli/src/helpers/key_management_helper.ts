import { ApiPromise } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { AccountId } from "@polkadot/types/interfaces";
import { AuthorizationData, Expiry, Permissions, Signatory } from "../types";
import { sendTx } from "../util/init";

/**
 * @description Attaches a secondary key to each DID
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair[]} secondaryKeys - KeyringPair[]
 * @param {KeyringPair[]} primaryKeys - KeyringPair[]
 * @return {Promise<void>}
 */
export async function addSecondaryKeys(
	api: ApiPromise,
	secondaryKeys: KeyringPair[],
	primaryKeys: KeyringPair[]
): Promise<void> {
	let totalPermissions: Permissions = {
		asset: [],
		extrinsic: [],
		portfolio: [],
	};

	for (let i in primaryKeys) {
		let target: Signatory = {
			Account: secondaryKeys[i].publicKey as AccountId,
		};
		let authData: AuthorizationData = {
			JoinIdentity: totalPermissions,
		};
		let expiry: Expiry = undefined;
		// 1. Add Secondary Item to identity.
		const transaction = api.tx.identity.addAuthorization(target, authData, expiry);
		await sendTx(primaryKeys[i], transaction);
	}
}

/**
 * @description Attaches a secondary key to each DID
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} signer - KeyringPair
 * @param {Signatory[]} signatories - Array of signatories
 * @param {number} numOfSigners - Number of signers
 * @return {Promise<void>}
 */
export async function createMultiSig(
	api: ApiPromise,
	signer: KeyringPair,
	signatories: Signatory[],
	numOfSigners: number
): Promise<void> {
	const transaction = api.tx.multiSig.createMultisig(signatories, numOfSigners);
	await sendTx(signer, transaction);
}
