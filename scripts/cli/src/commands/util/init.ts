import { AccountId, Address } from "@polkadot/types/interfaces/runtime";
import { Type } from "@polkadot/types";
import { SubmittableExtrinsic } from "@polkadot/api/types";
import { IKeyringPair, ISubmittableResult } from "@polkadot/types/types";
import {
  cryptoWaitReady,
  blake2AsHex,
  mnemonicGenerate,
} from "@polkadot/util-crypto";
import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import BN from "bn.js";
import fs from "fs";
import path from "path";
import { KeyringPair } from "@polkadot/keyring/types";

export let api: ApiPromise;

export type Account = {
  address: Address;
  isLocked: Boolean;
  meta: { name: String };
  publicKey: AccountId;
  type: Type;
};

export let cdd_provider: IKeyringPair;

export let nonces = new Map();

export async function setAPI(endpoint: string) {
  // Schema path
  const filePath = path.join(
    __dirname + "/../../../../../polymesh_schema.json"
  );
  const { types } = JSON.parse(fs.readFileSync(filePath, "utf8"));

  // Start node instance
  const ws_provider = new WsProvider(endpoint);
  api = await ApiPromise.create({
    types,
    provider: ws_provider,
  });
}

export function getAPI() {
  return api;
}

export async function setCddProvider(api: ApiPromise, name: string) {
  cdd_provider = await generateEntity(api, name);
}

export function getCddProvider() {
  return cdd_provider;
}

export async function generateEntity(api: ApiPromise, name: string) {
  let entity: KeyringPair;
  await cryptoWaitReady();
  entity = new Keyring({ type: "sr25519" }).addFromUri(`//${name}`, {
    name: `${name}`,
  });
  let entityRawNonce = (await api.query.system.account(entity.address)).nonce;
  let entity_nonce = new BN(entityRawNonce.toString());
  nonces.set(entity.address, entity_nonce);

  return entity;
}

export async function generateKeys(
  api: ApiPromise,
  numberOfKeys: number,
  keyPrepend: string
) {
  let keys: KeyringPair[] = [];
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
    let accountRawNonce = (await api.query.system.account(keys[i].address))
      .nonce;
    let account_nonce = new BN(accountRawNonce.toString());
    nonces.set(keys[i].address, account_nonce);
  }
  return keys;
}

export function sendTransaction(
  transaction: SubmittableExtrinsic<"promise">,
  signer: IKeyringPair,
  nonceObj: {}
) {
  return new Promise((resolve, reject) => {
    let receipt: ISubmittableResult;
    const gettingUnsub = transaction.signAndSend(
      signer,
      nonceObj,
      (receipt) => {
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
              const dispatchError: any = failed.event.data[0];

              if (dispatchError.isModule) {
                // known error
                const mod = dispatchError.asModule;
                const {
                  section,
                  name,
                  documentation,
                } = mod.registry.findMetaError(
                  new Uint8Array([mod.index.toNumber(), mod.error.toNumber()])
                );

                message = `${section}.${name}: ${documentation.join(" ")}`;
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
  });
}
