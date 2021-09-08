import type { KeyringPair } from "@polkadot/keyring/types";
import type { AccountId } from "@polkadot/types/interfaces/runtime";
import { sendTx, ApiSingleton } from "../util/init";
import { ItnRewardStatus } from "../types";
import { u8aToHex, numberToU8a } from "@polkadot/util";

/**
 * @description Claim an ITN reward with a valid signature.
 */
export async function claimItnReward(
	signer: KeyringPair,
	itnAddress: string | Uint8Array | AccountId
) {
	const api = await ApiSingleton.getInstance();
	const rewardAddress = signer.publicKey;
	let signature = u8aToHex(rewardAddress, 512);
	signature = signature + "claim_itn_reward";
	const transaction = api.tx.rewards.claimItnReward(rewardAddress, itnAddress, {
		Sr25519: signature,
	});
	await sendTx(signer, transaction);
}

/**
 * @description Set the status of an account ITN reward,
 * can only be called by root.
 */
export async function setItnRewardStatus(
	signer: KeyringPair,
	itnAddress: string | Uint8Array | AccountId,
	status: ItnRewardStatus
) {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.rewards.setItnRewardStatus(itnAddress, status);
	await sendTx(signer, transaction);
}
