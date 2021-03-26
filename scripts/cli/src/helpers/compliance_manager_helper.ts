import type { KeyringPair } from "@polkadot/keyring/types";
import type { Ticker, Scope } from "../types";
import { sendTx, ApiSingleton } from "../util/init";
import { assert } from "chai";
import type { IdentityId } from "../interfaces";

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
 * @param {KeyringPair} signer - KeyringPair
 * @param {IdentityId} did - IdentityId
 * @param {Ticker} ticker - Ticker
 * @return {Promise<void>}
 */
export async function createClaimCompliance(signer: KeyringPair, did: IdentityId, ticker: Ticker): Promise<void> {
	const api = await ApiSingleton.getInstance();
	assert(ticker.length <= 12, "Ticker cannot be longer than 12 characters");

	let senderConditions = senderConditions1(did, { Ticker: ticker });
	let receiverConditions = receiverConditions1(did, { Ticker: ticker });

	const transaction = api.tx.complianceManager.addComplianceRequirement(
		ticker,
		[senderConditions],
		[receiverConditions]
	);
	await sendTx(signer, transaction);
}

/**
 * @description Creates claim compliance for an asset
 * @param {KeyringPair} signer - KeyringPair
 * @param {Ticker} ticker - Ticker
 * @return {Promise<void>}
 */
export async function addComplianceRequirement(sender: KeyringPair, ticker: Ticker): Promise<void> {
	const api = await ApiSingleton.getInstance();
	let assetCompliance = await api.query.complianceManager.assetCompliances(ticker);

	if (assetCompliance.requirements.length == 0) {
		const transaction = api.tx.complianceManager.addComplianceRequirement(ticker, [], []);

		await sendTx(sender, transaction);
	} else {
		console.log("Asset already has compliance.");
	}
}
