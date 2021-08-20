import type { KeyringPair } from "@polkadot/keyring/types";
import type { AccountId } from "@polkadot/types/interfaces/runtime";
import { sendTx, ApiSingleton } from "../util/init";

/**
 * @description creates an authorization to allow a `user_key`
 * to accept a `paying_key` as their subsidiser.
 */
export async function setPayingKey(
	signer: KeyringPair,
	userKey: string | Uint8Array | AccountId,
	polyLimit: number
) {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.relayer.setPayingKey(userKey, polyLimit);
	await sendTx(signer, transaction);
}

/**
 * @description accepts a `paying_key` authorization.
 */
export async function acceptPayingKey(signer: KeyringPair, authId: number) {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.relayer.acceptPayingKey(authId);
	await sendTx(signer, transaction);
}

/**
 * @description removes the `paying_key` from a `user_key`.
 */
export async function removePayingKey(
	signer: KeyringPair,
	userKey: string | Uint8Array | AccountId
) {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.relayer.removePayingKey(userKey, signer.publicKey);
	await sendTx(signer, transaction);
}

/**
 * @description updates the available POLYX for a `user_key`.
 */
export async function updatePolyxLimit(
	signer: KeyringPair,
	userKey: string | Uint8Array | AccountId,
	amount: number
) {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.relayer.updatePolyxLimit(userKey, amount);
	await sendTx(signer, transaction);
}

/**
 * @description increases the available POLYX for a `user_key`.
 */
export async function increasePolyxLimit(
	signer: KeyringPair,
	userKey: string | Uint8Array | AccountId,
	amount: number
) {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.relayer.increasePolyxLimit(userKey, amount);
	await sendTx(signer, transaction);
}

/**
 * @description decreases the available POLYX for a `user_key`.
 */
export async function decreasePolyxLimit(
	signer: KeyringPair,
	userKey: string | Uint8Array | AccountId,
	amount: number
) {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.relayer.decreasePolyxLimit(userKey, amount);
	await sendTx(signer, transaction);
}
