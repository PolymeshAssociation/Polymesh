import { ApiPromise } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { sendTx } from "../util/init";
import { assert } from "chai";
import { IdentityId, Ticker, AssetCompliance, Scope } from "../types";

const senderConditions1 = function (trusted_did: IdentityId, data: Scope) {
	return {
		condition_type: {
			IsPresent: {
				Exempted: data,
			},
		},
		issuers: [
			{
				issuer: trusted_did,
				trusted_for: { Any: "" },
			},
		],
	};
};

const receiverConditions1 = senderConditions1;

/**
 * @description Creates claim compliance for an asset
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} signer - KeyringPair
 * @param {IdentityId} did - IdentityId
 * @param {Ticker} ticker - Ticker
 * @return {Promise<void>}
 */
export async function createClaimCompliance(
	api: ApiPromise,
	signer: KeyringPair,
	did: IdentityId,
	ticker: Ticker
): Promise<void> {
	assert(ticker.length <= 12, "Ticker cannot be longer than 12 characters");

	let senderConditions = senderConditions1(did, {Ticker: ticker});
	let receiverConditions = receiverConditions1(did, {Ticker: ticker});

	const transaction = api.tx.complianceManager.addComplianceRequirement(
		ticker,
		[senderConditions],
		[receiverConditions]
	);
	await sendTx(signer, transaction);
}

/**
 * @description Creates claim compliance for an asset
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} signer - KeyringPair
 * @param {Ticker} ticker - Ticker
 * @return {Promise<void>}
 */
export async function addComplianceRequirement(api: ApiPromise, sender: KeyringPair, ticker: Ticker): Promise<void> {
	let assetCompliance = ((await api.query.complianceManager.assetCompliances(ticker)) as unknown) as AssetCompliance;

	if (assetCompliance.requirements.length == 0) {
		const transaction = api.tx.complianceManager.addComplianceRequirement(ticker, [], []);

		await sendTx(sender, transaction);
	} else {
		console.log("Asset already has compliance.");
	}
}
