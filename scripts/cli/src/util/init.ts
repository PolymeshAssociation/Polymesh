import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { cryptoWaitReady, blake2AsHex, mnemonicGenerate } from "@polkadot/util-crypto";
import { stringToU8a, u8aConcat, u8aFixLength, u8aToHex } from "@polkadot/util";
import BN from "bn.js";
import fs from "fs";
import path from "path";
import cryptoRandomString from "crypto-random-string";
import type { AccountId } from "@polkadot/types/interfaces/runtime";
import type { SubmittableExtrinsic } from "@polkadot/api/types";
import type { KeyringPair } from "@polkadot/keyring/types";
//import { some, none, ap, Option } from "fp-ts/lib/Option";
//import type { Option, Vec } from '@polkadot/types/codec';
import type { DispatchError } from "@polkadot/types/interfaces";
import type { IdentityId, Ticker, NonceObject, Signatory } from "../types";
import { createIdentities } from "../helpers/identity_helper";

let nonces = new Map();
let block_sizes: Number[] = [];
let block_times: Number[] = [];
let synced_block = 0;
let synced_block_ts = 0;

// Amount to seed each key with
export const transferAmount = new BN(25000).mul(new BN(10).pow(new BN(6))).toNumber();

export async function sleep(ms: number) {
	return new Promise(resolve => setTimeout(resolve, ms));
}

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

export function generateRandomTicker() {
	let ticker = cryptoRandomString({ length: 12, type: "distinguishable" });
	return ticker;
}

export function generateRandomKey() {
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

export async function handle <T>(promise: Promise<T>): Promise<T[] | [T, any]> {
 	try {
		 const data: T = await promise;
		 return ([data, undefined]);
	 } catch (error) {
		 return ([undefined, error]);
	 }
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

export async function sendTx(signer: KeyringPair, tx: SubmittableExtrinsic<"promise">) {
	let nonceObj = { nonce: nonces.get(signer.address) };	
	nonces.set(signer.address, nonces.get(signer.address).addn(1));	
	const [result, resultErr] = await handle(sendTransaction(tx, signer, nonceObj));
	if (resultErr) throw new Error("Transaction Failed");
	return result ;		
}

export function getDefaultPortfolio(did: IdentityId) {
	return { did: did, kind: "Default" };
}
