import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { cryptoWaitReady, blake2AsHex, mnemonicGenerate } from "@polkadot/util-crypto";
import { stringToU8a, u8aConcat, u8aFixLength, u8aToHex } from "@polkadot/util";
import BN from "bn.js";
import assert from "assert";
import fs from "fs";
import path from "path";
import cryptoRandomString from "crypto-random-string";
import { AccountId, Balance, Moment } from "@polkadot/types/interfaces/runtime";
import { SubmittableExtrinsic } from "@polkadot/api/types";
import { ISubmittableResult } from "@polkadot/types/types";
import { u8, u64 } from "@polkadot/types/primitive";
import { KeyringPair } from "@polkadot/keyring/types";
//import { some, none, ap, Option } from "fp-ts/lib/Option";
//import type { Option, Vec } from '@polkadot/types/codec';
import type { DispatchError, EventRecord } from "@polkadot/types/interfaces";
import { IdentityId, Scope, Ticker, NonceObject, AssetCompliance, Signatory, Expiry } from "../types";
import { createIdentities } from "../helpers/identity_helper";

let nonces = new Map();
let block_sizes: Number[] = [];
let block_times: Number[] = [];
let synced_block = 0;
let synced_block_ts = 0;

// Amount to seed each key with
export const transferAmount = new BN(25000).mul(new BN(10).pow(new BN(6))).toNumber();

function senderConditions1(trusted_did: IdentityId, data: Scope) {
	return [
		{
			condition_type: {
				IsPresent: {
					Exempted: data,
				},
			},
			issuers: [{ issuer: trusted_did, trusted_for: { Any: "" } }],
		},
	];
}

const receiverConditions1 = senderConditions1;

// Initialization Main is used to generate all entities e.g (Alice, Bob, Dave)
export async function initMain(api: ApiPromise) {
	let entities = [];

	let alice = await generateEntity(api, "Alice");
	let relay = await generateEntity(api, "relay_1");
	let govCommittee1 = await generateEntity(api, "governance_committee_1");
	let govCommittee2 = await generateEntity(api, "governance_committee_2");

	entities.push(alice);
	entities.push(relay);
	entities.push(govCommittee1);
	entities.push(govCommittee2);

	return entities;
}

export async function createApi() {
	// Schema path
	const filePath = path.join(__dirname + "../../../../../polymesh_schema.json");
	const { types } = JSON.parse(fs.readFileSync(filePath, "utf8"));

	// Start node instance
	const ws_provider = new WsProvider(process.env.WS_PROVIDER || "ws://127.0.0.1:9944/");
	const api = await ApiPromise.create({
		types,
		provider: ws_provider,
	});
	return { api, ws_provider };
}

export async function generateEntity(api: ApiPromise, name: string) {
	await cryptoWaitReady();
	let entity = new Keyring({ type: "sr25519" }).addFromUri(`//${name}`, {
		name: `${name}`,
	});
	let entityRawNonce = (await api.query.system.account(entity.address)).nonce;
	let entity_nonce = new BN(entityRawNonce.toString());
	nonces.set(entity.address, entity_nonce);

	return entity;
}

export async function generateKeys(api: ApiPromise, numberOfKeys: Number, keyPrepend: String) {
	let keys = [];
	await cryptoWaitReady();
	for (let i = 0; i < numberOfKeys; i++) {
		keys.push(
			new Keyring({ type: "sr25519" }).addFromUri("//" + keyPrepend + i.toString(), {
				name: i.toString(),
			})
		);
		let accountRawNonce = (await api.query.system.account(keys[i].address)).nonce;
		let account_nonce = new BN(accountRawNonce.toString());
		nonces.set(keys[i].address, account_nonce);
	}
	return keys;
}

export async function generateEntityFromUri(api: ApiPromise, uri: string) {
	await cryptoWaitReady();
	let entity = new Keyring({ type: "sr25519" }).addFromUri(uri);
	let accountRawNonce = (await api.query.system.account(entity.address)).nonce;
	let account_nonce = new BN(accountRawNonce.toString());
	nonces.set(entity.address, account_nonce);
	return entity;
}

export async function generateRandomEntity(api: ApiPromise) {
	let entity = await generateEntityFromUri(api, cryptoRandomString({ length: 10 }));
	return entity;
}

export async function generateRandomTicker() {
	let ticker = cryptoRandomString({ length: 12, type: "distinguishable" });
	return ticker;
}

export async function generateRandomKey() {
	let ticker = cryptoRandomString({ length: 12, type: "alphanumeric" });
	return ticker;
}

export async function blockTillPoolEmpty(api: ApiPromise) {
	let prev_block_pending = 0;
	let done_something = false;
	let done = false;
	const unsub = await api.rpc.chain.subscribeNewHeads(async (header) => {
		let last_synced_block = synced_block;
		if (header.number.toNumber() > last_synced_block) {
			for (let i = last_synced_block + 1; i <= header.number.toNumber(); i++) {
				let block_hash = await api.rpc.chain.getBlockHash(i);
				let block = await api.rpc.chain.getBlock(block_hash);
				block_sizes[i] = block["block"]["extrinsics"].length;
				if (block_sizes[i] > 2) {
					done_something = true;
				}
				let timestamp_extrinsic = block["block"]["extrinsics"][0];
				let new_block_ts = parseInt(JSON.stringify(timestamp_extrinsic["method"].args[0].toHuman()));
				block_times[i] = new_block_ts - synced_block_ts;
				synced_block_ts = new_block_ts;
				synced_block = i;
			}
		}
		let pool = await api.rpc.author.pendingExtrinsics();
		if (done_something && pool.length == 0) {
			unsub();
			done = true;
		}
	});
	// Should use a mutex here...
	while (!done) {
		await new Promise((resolve) => setTimeout(resolve, 1000));
	}
}

// Fetches DID that belongs to the Account Key
export async function keyToIdentityIds(
	api: ApiPromise,
	accountKey: AccountId | KeyringPair["publicKey"]
): Promise<IdentityId> {
	let account_did = await api.query.identity.keyToIdentityIds(accountKey);
	return account_did.toHuman() as IdentityId;
}

// Returns the asset did
export function tickerToDid(ticker: Ticker) {
	let tickerString = String.fromCharCode.apply(ticker);
	let tickerUintArray = Uint8Array.from(tickerString, (x) => x.charCodeAt(0));
	return blake2AsHex(u8aConcat(stringToU8a("SECURITY_TOKEN:"), u8aFixLength(tickerUintArray, 96, true)));
}

// Creates claim compliance for an asset
export async function createClaimCompliance(
	api: ApiPromise,
	accounts: KeyringPair[],
	dids: IdentityId[],
	ticker: Ticker
) {
	assert(ticker.length <= 12, "Ticker cannot be longer than 12 characters");

	let senderConditions = senderConditions1(dids[1], { Ticker: ticker });
	let receiverConditions = receiverConditions1(dids[1], { Ticker: ticker });

	const transaction = api.tx.complianceManager.addComplianceRequirement(ticker, senderConditions, receiverConditions);
	await sendTx(accounts[0], transaction);
}

// TODO Refactor function to deal with all the possible claim types and their values
export async function addClaimsToDids(
	api: ApiPromise,
	accounts: KeyringPair[],
	did: IdentityId,
	claimType: string,
	claimValue: Scope,
	expiry: Expiry
) {
	// Receieving Conditions Claim
	let claim = { [claimType]: claimValue };
	const transaction = api.tx.identity.addClaim(did, claim, expiry);
	await sendTx(accounts[1], transaction);
}

export async function generateStashKeys(api: ApiPromise, accounts: KeyringPair[]) {
	let keys = [];
	await cryptoWaitReady();
	for (let i = 0; i < accounts.length; i++) {
		keys.push(
			new Keyring({ type: "sr25519" }).addFromUri(`//${accounts[i]}//stash`, {
				name: `${accounts[i] + "_stash"}`,
			})
		);
		let accountRawNonce = (await api.query.system.account(keys[i].address)).nonce;
		let account_nonce = new BN(accountRawNonce.toString());
		nonces.set(keys[i].address, account_nonce);
	}
	return keys;
}

export function sendTransaction(
	transaction: SubmittableExtrinsic<"promise">,
	signer: KeyringPair,
	nonceObj: NonceObject
) {
	return new Promise((resolve, reject) => {
		const gettingUnsub = transaction.signAndSend(signer, nonceObj, (receipt) => {
			const { status } = receipt;

			if (receipt.isCompleted) {
				/*
				 * isCompleted === isFinalized || isError, which means
				 * no further updates, so we unsubscribe
				 */
				gettingUnsub.then((unsub) => {
					unsub();
				});

				if (receipt.isInBlock) {
					// tx included in a block and finalized
					const failed = receipt.findRecord("system", "ExtrinsicFailed");

					if (failed) {
						// get revert message from event
						let message = "";
						const dispatchError = (failed.event.data[0] as unknown) as DispatchError;

						if (dispatchError.isModule) {
							// known error
							const mod = dispatchError.asModule;
							const { section, name, documentation } = mod.registry.findMetaError(
								new Uint8Array([mod.index.toNumber(), mod.error.toNumber()])
							);

							message = `${section}.${name}: ${documentation.join(" ")}`;
						} else if (dispatchError.isBadOrigin) {
							message = "Bad origin";
						} else if (dispatchError.isCannotLookup) {
							message = "Could not lookup information required to validate the transaction";
						} else {
							message = "Unknown error";
						}

						reject(new Error(message));
					} else {
						resolve(receipt);
					}
				} else if (receipt.isError) {
					reject(new Error("Transaction Aborted"));
				}
			}
		});
	});
}

export async function signAndSendTransaction(transaction: SubmittableExtrinsic<"promise">, signer: KeyringPair) {
	let nonceObj = { nonce: nonces.get(signer.address) };
	await sendTransaction(transaction, signer, nonceObj);
	nonces.set(signer.address, nonces.get(signer.address).addn(1));
}

export async function generateOffchainKeys(api: ApiPromise, keyType: string) {
	const PHRASE = mnemonicGenerate();
	await cryptoWaitReady();
	const newPair = new Keyring({ type: "sr25519" }).addFromUri(PHRASE);
	await api.rpc.author.insertKey(keyType, PHRASE, u8aToHex(newPair.publicKey));
}

// Creates a Signatory Object
export async function signatory(api: ApiPromise, entity: KeyringPair, signer: KeyringPair) {
	let entityDid = (await createIdentities(api, [entity], signer))[0];
	let signatoryObj: Signatory = {
		Identity: entityDid,
	};
	return signatoryObj;
}

export async function mintingAsset(api: ApiPromise, minter: KeyringPair, ticker: Ticker) {
	const transaction = api.tx.asset.issue(ticker, 100);
	await sendTx(minter, transaction);
}

export async function sendTx(signer: KeyringPair, tx: SubmittableExtrinsic<"promise">) {
	let nonceObj = { nonce: nonces.get(signer.address) };
	let passed: EventRecord | undefined;
	try {
		const result = ((await sendTransaction(tx, signer, nonceObj)) as unknown) as ISubmittableResult;
		passed = result.findRecord("system", "ExtrinsicSuccess");
	} finally {
		nonces.set(signer.address, nonces.get(signer.address).addn(1));
	}
	if (!passed) return -1;
}

export async function addComplianceRequirement(api: ApiPromise, sender: KeyringPair, ticker: Ticker) {
	let assetCompliance = ((await api.query.complianceManager.assetCompliances(ticker)) as unknown) as AssetCompliance;

	if (assetCompliance.requirements.length == 0) {
		const transaction = api.tx.complianceManager.addComplianceRequirement(ticker, [], []);

		await sendTx(sender, transaction);
	} else {
		console.log("Asset already has compliance.");
	}
}

export async function createVenue(api: ApiPromise, sender: KeyringPair) {
	let venueCounter = await api.query.settlement.venueCounter();
	let venueDetails = [0];

	const transaction = api.tx.settlement.createVenue(venueDetails, [sender.address], 0);

	await sendTx(sender, transaction);

	return venueCounter;
}

export function getDefaultPortfolio(did: IdentityId) {
	return { did: did, kind: "Default" };
}

export async function affirmInstruction(
	api: ApiPromise,
	sender: KeyringPair,
	instructionCounter: u64,
	did: IdentityId,
	leg_counts: u64
) {
	const transaction = api.tx.settlement.affirmInstruction(instructionCounter, [getDefaultPortfolio(did)], leg_counts);

	await sendTx(sender, transaction);
}

export async function withdrawInstruction(
	api: ApiPromise,
	sender: KeyringPair,
	instructionCounter: u64,
	did: IdentityId
) {
	const transaction = api.tx.settlement.withdrawInstruction(instructionCounter, [getDefaultPortfolio(did)]);

	await sendTx(sender, transaction);
}

export async function rejectInstruction(
	api: ApiPromise,
	sender: KeyringPair,
	instructionCounter: u64,
	did: IdentityId
) {
	const transaction = api.tx.settlement.rejectInstruction(instructionCounter, [getDefaultPortfolio(did)]);

	await sendTx(sender, transaction);
}

export async function addInstruction(
	api: ApiPromise,
	venueCounter: u64,
	sender: KeyringPair,
	sender_did: IdentityId,
	receiver_did: IdentityId,
	ticker: Ticker,
	ticker2: Ticker,
	amount: Balance
) {
	let instructionCounter = await api.query.settlement.instructionCounter();
	let transaction;
	let leg = {
		from: sender_did,
		to: receiver_did,
		asset: ticker,
		amount: amount,
	};

	let leg2 = {
		from: receiver_did,
		to: sender_did,
		asset: ticker2,
		amount: amount,
	};

	if (ticker2 === null || ticker2 === undefined) {
		transaction = api.tx.settlement.addInstruction(venueCounter, 0, null, null, [leg]);
	} else {
		transaction = api.tx.settlement.addInstruction(venueCounter, 0, null, null, [leg, leg2]);
	}

	await sendTx(sender, transaction);

	return instructionCounter;
}

async function claimReceipt(
	api: ApiPromise,
	sender: KeyringPair,
	sender_did: IdentityId,
	receiver_did: IdentityId,
	ticker: Ticker,
	amount: Balance,
	instructionCounter: u64
) {
	let msg = {
		receipt_uid: 0,
		from: sender_did,
		to: receiver_did,
		asset: ticker,
		amount: amount,
	};

	let receiptDetails = {
		receipt_uid: 0,
		leg_id: 0,
		signer: sender.address,
		signature: 1,
	};

	const transaction = api.tx.settlement.claimReceipt(instructionCounter, receiptDetails);

	await sendTx(sender, transaction);
}
