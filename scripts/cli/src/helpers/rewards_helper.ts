import type { KeyringPair } from "@polkadot/keyring/types";
import type { ItnRewardStatus } from "../types";
import { sendTx, ApiSingleton } from "../util/init";
import { u8aToHex } from "@polkadot/util";

/**
 * @description Claim an ITN reward with a valid signature.
 */
export async function claimItnReward(itn: KeyringPair, reward: KeyringPair) {
  const api = await ApiSingleton.getInstance();
  let claim_msg_hex = "636c61696d5f69746e5f726577617264"; // "claim_itn_reward" as hex.
  let msg = u8aToHex(reward.publicKey, 512).toString() + claim_msg_hex;
  let signature = itn.sign(msg);
  const transaction = api.tx.rewards.claimItnReward(msg, itn.address, {
    Sr25519: signature,
  });
  return transaction.send();
}

/**
 * @description Set the status of an account ITN reward,
 * can only be called by root.
 */
export async function setItnRewardStatus(
  signer: KeyringPair,
  itnAddress: KeyringPair,
  status: ItnRewardStatus
) {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.sudo.sudo(
    api.tx.rewards.setItnRewardStatus(itnAddress.publicKey, status)
  );
  return sendTx(signer, transaction);
}
