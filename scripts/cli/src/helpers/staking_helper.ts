import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@polkadot/keyring/types";
import { sendTx } from "../util/init";

/**
 * @description Take the origin account as a stash and lock up `value` of its balance.
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} controller - KeyringPair
 * @param {number} value - Amount to bond
 * @param {string} payee - RewardDestination
 * @param {KeyringPair} signer - KeyringPair
 * @return {Promise<void>}
 */
export async function bond(
	api: ApiPromise,
	controller: KeyringPair,
	value: number,
	payee: string,
	signer: KeyringPair
): Promise<void> {
	const transaction = api.tx.staking.bond(controller.publicKey, value, payee);
	await sendTx(signer, transaction);
}
