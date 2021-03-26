import type { KeyringPair } from "@polkadot/keyring/types";
import type { AccountId } from "@polkadot/types/interfaces";
import type { Expiry, Permissions, Signatory } from "../types";
import { sendTx, ApiSingleton } from "../util/init";

/**
 * @description Attaches a secondary key to each DID
 * @param {KeyringPair[]} secondaryKeys - KeyringPair[]
 * @param {KeyringPair[]} primaryKeys - KeyringPair[]
 * @return {Promise<void>}
 */
export async function addSecondaryKeys(secondaryKeys: KeyringPair[], primaryKeys: KeyringPair[]): Promise<void> {
	const api = await ApiSingleton.getInstance();
	let totalPermissions: Permissions = {
		asset: [],
		extrinsic: [],
		portfolio: [],
	};

	for (let i in primaryKeys) {
		let target = {
			Account: secondaryKeys[i].publicKey as AccountId,
		};
		let authData = {
			JoinIdentity: totalPermissions,
		};
		let expiry: Expiry = null;
		// 1. Add Secondary Item to identity.
		const transaction = api.tx.identity.addAuthorization(target, authData, expiry);
		await sendTx(primaryKeys[i], transaction).catch((err) => console.log(`Error: ${err.message}`));
	}
}

/**
 * @description Attaches a secondary key to each DID
 * @param {KeyringPair} signer - KeyringPair
 * @param {Signatory[]} signatories - Array of signatories
 * @param {number} numOfSigners - Number of signers
 * @return {Promise<void>}
 */
export async function createMultiSig(
	signer: KeyringPair,
	signatories: Signatory[],
	numOfSigners: number
): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.multiSig.createMultisig(signatories, numOfSigners);
	await sendTx(signer, transaction).catch((err) => console.log(`Error: ${err.message}`));
}
