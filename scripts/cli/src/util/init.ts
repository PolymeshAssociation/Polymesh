import "@polkadot/api/augment";
import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import {
  cryptoWaitReady,
  blake2AsHex,
  mnemonicGenerate,
} from "@polkadot/util-crypto";
import {
  stringToU8a,
  u8aConcat,
  u8aFixLength,
  u8aToHex,
  hexToString,
} from "@polkadot/util";
import BN from "bn.js";
import fs from "fs";
import path from "path";
import type { AccountId } from "@polkadot/types/interfaces/runtime";
import type { SubmittableExtrinsic } from "@polkadot/api/types";
import type { KeyringPair } from "@polkadot/keyring/types";
import type { DispatchError, EventRecord, AccountInfo, ActiveEraInfo } from "@polkadot/types/interfaces";
import type { ISubmittableResult } from "@polkadot/types/types";
import type { Ticker } from "../types";
import { createIdentities } from "../helpers/identity_helper";
import { distributePoly } from "../helpers/poly_helper";
import type { IdentityId } from "../interfaces";
import { assert } from "chai";
import { Option } from "@polkadot/types-codec";

let block_sizes: Number[] = [];
let block_times: Number[] = [];
let genesisEntities: KeyringPair[] = [];
let synced_block = 0;
let synced_block_ts = 0;

// Amount to seed each key with
export const transferAmount = new BN(25000)
  .mul(new BN(10).pow(new BN(6)))
  .toNumber();

export class ApiSingleton {
  private static api: Promise<ApiPromise>;

  private constructor() {
    ApiSingleton.api = this.createApi();
  }

  public static getInstance() {
    if (!ApiSingleton.api) {
      new ApiSingleton();
    }
    return ApiSingleton.api;
  }

  async createApi() {
    // Schema path
    const filePath = path.join(
      __dirname + "../../../../../polymesh_schema.json"
    );
    const { types } = JSON.parse(fs.readFileSync(filePath, "utf8"));

    // Start node instance
    const ws_provider = new WsProvider(
      process.env.WS_PROVIDER || "ws://127.0.0.1:9944/"
    );
    const api = await ApiPromise.create({
      types,
      provider: ws_provider,
    });
    return api;
  }
}

export async function disconnect() {
  const api = await ApiSingleton.getInstance();
  await api.disconnect();
}

export async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
const getEra = async () => {
  const api = await ApiSingleton.getInstance();
  const activeEra = await api.query.staking.activeEra();
  activeEra.unwrap().index.toJSON();
}

export async function waitNextEra() {
  const era = await getEra();
  while ((await getEra()) === era) {
    await waitNextBlock();
  }
}

export async function currentBlock() {
  const api = await ApiSingleton.getInstance();
  return (await api.query.system.number()).toNumber();
}

export async function waitBlocks(blocks: number) {
  let end_block = (await currentBlock()) + blocks;
  while ((await currentBlock()) < end_block) {
    await sleep(100);
  }
}

export async function waitNextBlock() {
  await waitBlocks(1);
}

// Initialization Main is used to generate all entities e.g (Alice, Bob, Dave)
export async function initMain(): Promise<KeyringPair[]> {
  if (genesisEntities.length === 0) {
    genesisEntities = [
      await generateEntity("Alice"),
      await generateEntity("relay_1"),
      await generateEntity("polymath_1"),
      await generateEntity("polymath_2"),
      await generateEntity("Bob"),
    ];
  }
  return genesisEntities;
}

export async function generateEntities(accounts: string[]) {
  let entites = [];
  for (let i = 0; i < accounts.length; i++) {
    let entity = await generateEntity(accounts[i]);
    entites.push(entity);
  }
  return entites;
}

export async function generateEntity(name: string): Promise<KeyringPair> {
  await cryptoWaitReady();
  return new Keyring({ type: "sr25519" }).addFromUri(`//${name}`, {
    name: `${name}`,
  });
}

export async function generateKeys(
  numberOfKeys: Number,
  keyPrepend: String
): Promise<KeyringPair[]> {
  let keys = [];
  await cryptoWaitReady();
  for (let i = 0; i < numberOfKeys; i++) {
    keys.push(
      new Keyring({ type: "sr25519" }).addFromUri(
        "//" + keyPrepend + i.toString(),
        {
          name: i.toString(),
        }
      )
    );
  }
  return keys;
}

export async function generateEntityFromUri(uri: string): Promise<KeyringPair> {
  await cryptoWaitReady();
  return new Keyring({ type: "sr25519" }).addFromUri(uri);
}
const NULL_12 = "\0".repeat(12);
export function padTicker(ticker: string) {
  return (ticker + NULL_12).substring(0, 12);
}
export async function throws(fn: () => any, error?: any) {
  try {
    await fn();
  } catch (e) {
    if (error && error != e) {
      throw new Error(`error ${e} != ${error}`);
    }
    return;
  }
  throw new Error("The function didn't throw");
}

export async function blockTillPoolEmpty() {
  const api = await ApiSingleton.getInstance();
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
        let new_block_ts = parseInt(
          JSON.stringify(timestamp_extrinsic["method"].args[0].toHuman())
        );
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
  accountKey: AccountId | KeyringPair["publicKey"]
): Promise<IdentityId> {
  const api = await ApiSingleton.getInstance();
  let opt_rec = await api.query.identity.keyRecords(accountKey);
  if (opt_rec.isSome) {
    const rec = opt_rec.unwrap();
    if (rec.isPrimaryKey) {
      return rec.asPrimaryKey;
    } else if (rec.isSecondaryKey) {
      return rec.asSecondaryKey[0];
    }
  }
  return <IdentityId>(0 as unknown);
}

// Returns the asset did
export function tickerToDid(ticker: Ticker) {
  let tickerString = String.fromCharCode.apply(ticker);
  let tickerUintArray = Uint8Array.from(tickerString, (x) => x.charCodeAt(0));
  return blake2AsHex(
    u8aConcat(
      stringToU8a("SECURITY_TOKEN:"),
      u8aFixLength(tickerUintArray, 96, true)
    )
  );
}

export async function generateStashKeys(
  accounts: string[]
): Promise<KeyringPair[]> {
  let keys = [];
  await cryptoWaitReady();
  for (let account of accounts) {
    keys.push(
      new Keyring({ type: "sr25519" }).addFromUri(`//${account}//stash`, {
        name: `${account + "_stash"}`,
      })
    );
  }
  return keys;
}

export async function sendTx(
  signer: KeyringPair,
  tx: SubmittableExtrinsic<"promise">
) {
   return sendTransaction(signer, tx, -1);
}

export function sendTransaction(
  signer: KeyringPair,
  transaction: SubmittableExtrinsic<"promise">,
  nonceObj: any
) {
  return new Promise<ISubmittableResult>((resolve, reject) => {
    const gettingUnsub = transaction.signAndSend(
      signer,
      nonceObj,
      (receipt) => {
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
              const dispatchError = <DispatchError>failed.event.data[0];
              if (dispatchError.isModule) {
                // known error
                const mod = dispatchError.asModule;
                const { section, name, docs } =
                  mod.registry.findMetaError(mod);

                message = `${section}.${name}: ${docs.join(" ")}`;
              } else if (dispatchError.isBadOrigin) {
                message = "Bad origin";
              } else if (dispatchError.isCannotLookup) {
                message =
                  "Could not lookup information required to validate the transaction";
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
      }
    );
    gettingUnsub.catch(reject);
  });
}

export async function generateOffchainKeys(keyType: string) {
  const api = await ApiSingleton.getInstance();
  const PHRASE = mnemonicGenerate();
  await cryptoWaitReady();
  const newPair = new Keyring({ type: "sr25519" }).addFromUri(PHRASE);
  await api.rpc.author.insertKey(keyType, PHRASE, u8aToHex(newPair.publicKey));
}

// Creates a Signatory Object
export async function signatory(signer: KeyringPair, entity: KeyringPair) {
  let entityDid = (await createIdentities(signer, [entity]))[0];
  let signatoryObj = {
    Identity: entityDid,
  };
  return signatoryObj;
}

export function getDefaultPortfolio(did: IdentityId) {
  return { did: did, kind: { Default: ""} };
}

export async function getValidCddProvider(alice: KeyringPair) {
  const api = await ApiSingleton.getInstance();
  let transfer_amount = new BN(1000).mul(new BN(10).pow(new BN(6)));
  // Fetch the cdd providers key and provide them right fuel to spent for
  // cdd creation
  let service_providers = await api.query.cddServiceProviders.activeMembers();
  let service_provider_1_key = await generateEntity("service_provider_1");

  // match the identity within the identity pallet
  const service_provider_1_identity = await keyToIdentityIds(
    service_provider_1_key.publicKey
  );
  assert.equal(
    service_provider_1_identity.toString(),
    service_providers[0].toString()
  );

  // fund the service_provider_1 account key to successfully call the `register_did` dispatchable
  let old_balance = (
    await api.query.system.account(service_provider_1_key.address)
  ).data.free;

  await distributePoly(alice, service_provider_1_key, transferAmount);
  //await blockTillPoolEmpty();

  // check the funds of service_provider_1
  let new_free_balance = (
    await api.query.system.account(service_provider_1_key.address)
  ).data.free;
  assert.equal(
    new_free_balance.toString(),
    transfer_amount.add(old_balance).toString()
  );
  return service_provider_1_key;
}

export async function getExpiries(length: number) {
  const api = await ApiSingleton.getInstance();
  let blockTime = (api.consts.babe.expectedBlockTime).toNumber();
  let bondingDuration = (api.consts.staking.bondingDuration).toNumber();
  let sessionPerEra = (api.consts.staking.sessionsPerEra).toNumber();
  let session_length = (api.consts.babe.epochDuration).toNumber();
  const currentBlockTime = (await api.query.timestamp.now()).toNumber();

  const bondingTime =
    bondingDuration *
    sessionPerEra *
    session_length;
  let expiryTime = currentBlockTime + bondingTime * 1000;

  let expiries = [];
  for (let i = 1; i <= length; i++) {
    // Providing 15 block as the extra time
    let temp = expiryTime + i * 5 * blockTime;
    expiries.push(temp);
  }
  return expiries;
}

export async function subscribeCddOffchainWorker() {
  const api = await ApiSingleton.getInstance();
  let eventCount = 0;
  const unsubscribe = await api.rpc.chain.subscribeNewHeads(async (header) => {
    console.log(`Chain is at block: #${header.number.unwrap()}`);
    let hash = await api.rpc.chain.getBlockHash(header.number.unwrap());
    let events = await api.query.system.events.at(hash.toString());
    for (let i = 0; i < Object.keys(events).length - 1; i++) {
      try {
        if (events[i].event.data.section == "CddOffchainWorker") {
          let typeList = events[i].event.data.typeDef;
          console.log(
            `EventName - ${
              events[i].event.data.method
            } at block number ${header.number.unwrap()}`
          );
          for (let j = 0; j < typeList.length; j++) {
            let value = events[i].event.data[j].toString();
            if (typeList[j].type == "Bytes")
              value = hexToString(u8aToHex(events[i].event.data[j].toU8a()));
            console.log(`${typeList[j].type} : ${value}`);
            eventCount++;
          }
          console.log("***************************************");
        }
      } catch (error) {
        console.log(`Event is not present in this block ${header.number}`);
      }
    }
    if (eventCount >= 5) {
      process.exit(0);
    }
  });
}
