const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { cryptoWaitReady, blake2AsHex, mnemonicGenerate } = require("@polkadot/util-crypto");
const { stringToU8a, u8aConcat, u8aFixLength, u8aToHex } = require('@polkadot/util');
const BN = require("bn.js");
const assert = require("assert");
const fs = require("fs");
const path = require("path");

let nonces = new Map();
let sk_roles = [[0], [1], [2], [1, 2]];

let fail_count = 1;
let block_sizes = {};
let block_times = {};

let synced_block = 0;
let synced_block_ts = 0;

// Amount to seed each key with
let transfer_amount = new BN(25000).mul(new BN(10).pow(new BN(6)));

let prepend = "demo";

// Used for creating a single ticker
const ticker = `token${prepend}0`.toUpperCase();
assert( ticker.length <= 12, "Ticker cannot be longer than 12 characters");

const senderRules1 = function(trusted_did, asset_did) {
    return [
    {
      "rule_type": {
        "IsPresent": {
          "Exempted": asset_did
        }
      },
      "issuers": [trusted_did]
    }
  ];
}

const receiverRules1 = function(trusted_did, asset_did) {
    return [
    {
      "rule_type": {
        "IsPresent": {
          "Exempted": asset_did
        }
      },
      "issuers": [trusted_did]
    }
  ];
}

// Initialization Main is used to generate all entities e.g (Alice, Bob, Dave)
async function initMain(api) {
  let entities = [];
  
  let alice = await generateEntity(api, "Alice");
  let bob = await generateEntity(api, "Bob");
  let charlie = await generateEntity(api, "Charlie");
  let dave = await generateEntity(api, "Dave");
  let relay = await generateEntity(api, "relay_1");
  let govCommittee1 = await generateEntity(api, "governance_committee_1");
  let govCommittee2 = await generateEntity(api, "governance_committee_2");

  entities.push(alice);
  entities.push(bob);
  entities.push(charlie);
  entities.push(dave);
  entities.push(relay);
  entities.push(govCommittee1);
  entities.push(govCommittee2);

  return entities;
}

const createApi = async function() {
  // Schema path
  const filePath = reqImports.path.join(__dirname + "/../../../polymesh_schema.json");
  const { types } = JSON.parse(reqImports.fs.readFileSync(filePath, "utf8"));

  // Start node instance
  const ws_provider = new reqImports.WsProvider("ws://127.0.0.1:9944/");
  const api = await reqImports.ApiPromise.create({
    types,
    provider: ws_provider
  });
  return api;
}

let generateEntity = async function (api, name) {
  let entity = [];
  await cryptoWaitReady();
  entity = new Keyring({ type: "sr25519" }).addFromUri(`//${name}`, { name: `${name}` });
  let entityRawNonce = (await api.query.system.account(entity.address)).nonce;
  let entity_nonce = new BN(entityRawNonce.toString());
  nonces.set(entity.address, entity_nonce);

  return entity;
};



const generateKeys = async function (api, numberOfKeys, keyPrepend) {
  let keys = [];
  await cryptoWaitReady();
  for (let i = 0; i < numberOfKeys; i++) {
    keys.push(
      new Keyring({ type: "sr25519" }).addFromUri("//" + keyPrepend + i.toString(), {
        name: i.toString()
      })
    );
    let accountRawNonce = (await api.query.system.account(keys[i].address)).nonce;
    let account_nonce = new BN(accountRawNonce.toString());
    nonces.set(keys[i].address, account_nonce);
  }
  return keys;
};

const generateEntityFromUri = async function (api, uri) {
  await cryptoWaitReady();
  let entity = new Keyring({ type: "sr25519" }).addFromUri(uri);
  let accountRawNonce = (await api.query.system.account(entity.address)).nonce;
  let account_nonce = new BN(accountRawNonce.toString());
  nonces.set(entity.address, account_nonce);
  return entity;
};

const blockTillPoolEmpty = async function (api) {
  let prev_block_pending = 0;
  let done_something = false;
  let done = false;
  const unsub = await api.rpc.chain.subscribeNewHeads(async header => {
    let last_synced_block = synced_block;
    if (header.number > last_synced_block) {
      for (let i = last_synced_block + 1; i <= header.number; i++) {
        let block_hash = await api.rpc.chain.getBlockHash(i);
        let block = await api.rpc.chain.getBlock(block_hash);
        block_sizes[i] = block["block"]["extrinsics"].length;
        if (block_sizes[i] > 2) {
          done_something = true;
        }
        let timestamp_extrinsic = block["block"]["extrinsics"][0];
        let new_block_ts = parseInt(
          JSON.stringify(timestamp_extrinsic.raw["method"].args[0].raw)
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
    await new Promise(resolve => setTimeout(resolve, 1000));
  }
};

// Create a new DID for each of accounts[]
// precondition - accounts all have enough POLY
const createIdentities = async function(api, accounts, alice) {
    return await createIdentitiesWithExpiry(api, accounts, alice, []);
};

const createIdentitiesWithExpiry = async function(api, accounts, alice, expiries) {
  let dids = [];

  for (let i = 0; i < accounts.length; i++) {
    // let nonceObj = {nonce: nonces.get(alice.address)};
    // const transaction = api.tx.identity.cddRegisterDid(accounts[i].address, null, []);
    // await sendTransaction(transaction, alice, nonceObj);

      let expiry = expiries.length == 0 ? null : expiries[i];
      await api.tx.identity
        .cddRegisterDid(accounts[i].address, expiry, [])
        .signAndSend(alice, { nonce: nonces.get(alice.address) });

    nonces.set(alice.address, nonces.get(alice.address).addn(1));
  }
  await blockTillPoolEmpty(api);
  for (let i = 0; i < accounts.length; i++) {
    const d = await api.query.identity.keyToIdentityIds(accounts[i].publicKey);
    dids.push(d.raw.asUnique);
  }
  let did_balance = 1000 * 10**6;
  for (let i = 0; i < dids.length; i++) {
    // let nonceObjTwo = {nonce: nonces.get(alice.address)};
    // const transactionTwo = api.tx.balances.topUpIdentityBalance(dids[i], did_balance);
    // await sendTransaction(transactionTwo, alice, nonceObjTwo);

    await topUpIdentityBalance(api, alice, dids[i], did_balance);

  }
  return dids;
}

// Top up identity balance
async function topUpIdentityBalance(api, signer, did, did_balance) {
  let nonceObj = {nonce: nonces.get(signer.address)};
  const transaction = api.tx.balances.topUpIdentityBalance(did, did_balance);
  await sendTransaction(transaction, signer, nonceObj);

  nonces.set( signer.address, nonces.get(signer.address).addn(1));
}

// Fetches DID that belongs to the Account Key
async function keyToIdentityIds(api, accountKey) {
  let account_did = await api.query.identity.keyToIdentityIds(accountKey);
  return account_did;
}

// Sends transfer_amount to accounts[] from alice
async function distributePoly(api, to, amount, from) {
    // Perform the transfers
    let nonceObj = {nonce: nonces.get(from.address)};
    const transaction = api.tx.balances.transfer(to.address, amount);
    await sendTransaction(transaction, from, nonceObj);

    nonces.set( from.address, nonces.get(from.address).addn(1));
}


async function distributePolyBatch(api, to, amount, from) {
  // Perform the transfers
  for (let i = 0; i < to.length; i++) {
    await distributePoly(api, to[i], amount, from);
  }
}

// Attach a signing key to each DID
async function addSigningKeys(api, accounts, dids, signing_accounts) {
  for (let i = 0; i < accounts.length; i++) {
    // 1. Add Signing Item to identity.

    let nonceObj = {nonce: nonces.get(accounts[i].address)};
    const transaction = api.tx.identity.addAuthorizationAsKey({AccountKey: signing_accounts[i].publicKey}, {JoinIdentity: { target_did: dids[i], signing_item: null }}, null);
    await sendTransaction(transaction, accounts[i], nonceObj);

    // const unsub = await api.tx.identity
    // .addAuthorizationAsKey({AccountKey: signing_accounts[i].publicKey}, {JoinIdentity: { target_did: dids[i], signing_item: null }}, null)
    // .signAndSend(accounts[i], { nonce: nonces.get(accounts[i].address) });

    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
  }
}

// Authorizes the join of signing keys to a DID
async function authorizeJoinToIdentities(api, accounts, dids, signing_accounts) {

  for (let i = 0; i < accounts.length; i++) {
    // 1. Authorize
    const auths = await api.query.identity.authorizations.entries({
      AccountKey: signing_accounts[i].publicKey
    });
    let last_auth_id = 0;
    for (let i = 0; i < auths.length; i++) {
      if (auths[i][1].auth_id.toNumber() > last_auth_id) {
        last_auth_id = auths[i][1].auth_id.toNumber()
      }
    }

    let nonceObj = {nonce: nonces.get(signing_accounts[i].address)};
    const transaction = api.tx.identity.joinIdentityAsKey([last_auth_id]);
    await sendTransaction(transaction, signing_accounts[i], nonceObj);

    // const unsub = await api.tx.identity
    //   .joinIdentityAsKey([last_auth_id])
    //   .signAndSend(signing_accounts[i], { nonce: nonces.get(signing_accounts[i].address) });
    // nonces.set(signing_accounts[i].address, nonces.get(signing_accounts[i].address).addn(1));
  }

  return dids;
}

// Creates a token for a did
async function issueTokenPerDid(api, accounts) {

  // let nonceObj = {nonce: nonces.get(accounts[0].address)};
  // const transaction = api.tx.asset.createAsset(ticker, ticker, 1000000, true, 0, [], "abc");
  // await sendTransaction(transaction, accounts[0], nonceObj);

    const unsub = await api.tx.asset
      .createAsset(ticker, ticker, 1000000, true, 0, [], "abc")
      .signAndSend(accounts[0], { nonce: nonces.get(accounts[0].address) });

    nonces.set(accounts[0].address, nonces.get(accounts[0].address).addn(1));

}

// Returns the asset did
function tickerToDid(ticker) {
    return blake2AsHex(
      u8aConcat(stringToU8a("SECURITY_TOKEN:"), u8aFixLength(stringToU8a(ticker), 96, true)
        ));
}

// Creates claim rules for an asset
async function createClaimRules(api, accounts, dids) {

    const asset_did = tickerToDid(ticker);

    let senderRules = senderRules1(dids[1], asset_did);
    let receiverRules = receiverRules1(dids[1], asset_did);

    let nonceObj = {nonce: nonces.get(accounts[0].address)};
    const transaction = api.tx.complianceManager.addActiveRule(ticker, senderRules, receiverRules);
    await sendTransaction(transaction, accounts[0], nonceObj);

      nonces.set(accounts[0].address, nonces.get(accounts[0].address).addn(1));


}

// Adds claim to did
async function addClaimsToDids(api, accounts, did, claimType, claimValue, expiry) {

  // Receieving Rules Claim
  let claim = {[claimType]: claimValue};


    let nonceObj = {nonce: nonces.get(accounts[1].address)};
    expiry = expiry == 0 ? null : expiry;
    const transaction = api.tx.identity.addClaim(did, claim, expiry);
    await sendTransaction(transaction, accounts[1], nonceObj);

    nonces.set(accounts[1].address, nonces.get(accounts[1].address).addn(1));

}

const generateStashKeys = async function(api, accounts) {
  let keys = [];
  await cryptoWaitReady();
  for (let i = 0; i < accounts.length; i++) {
    keys.push(
      new Keyring({ type: "sr25519" }).addFromUri(`//${accounts[i]}//stash`, { name: `${accounts[i]+ "_stash"}`
      })
    );
    let accountRawNonce = (await api.query.system.account(keys[i].address)).nonce;
    let account_nonce = new BN(accountRawNonce.toString());
    nonces.set(keys[i].address, account_nonce);
  }
  return keys;
}

function sendTransaction(transaction, signer, nonceObj) {
  return new Promise((resolve, reject) => {

    const gettingUnsub = transaction.signAndSend(signer, nonceObj, receipt => {

      const { status } = receipt;

      if (receipt.isCompleted) {

        /*
         * isCompleted === isFinalized || isError, which means
         * no further updates, so we unsubscribe
         */
        gettingUnsub.then(unsub => {

          unsub();

        });

        if (receipt.isInBlock) {

          // tx included in a block and finalized
          const failed = receipt.findRecord('system', 'ExtrinsicFailed');

          if (failed) {

            // get revert message from event
            let message = "";
            const dispatchError = failed.event.data[0];

            if (dispatchError.isModule) {

              // known error
              const mod = dispatchError.asModule;
              const { section, name, documentation } = mod.registry.findMetaError(
                new Uint8Array([mod.index.toNumber(), mod.error.toNumber()])
              );

              message = `${section}.${name}: ${documentation.join(' ')}`;
            } else if (dispatchError.isBadOrigin) {
              message = 'Bad origin';
            } else if (dispatchError.isCannotLookup) {
              message = 'Could not lookup information required to validate the transaction';
            } else {
              message = 'Unknown error';
            }

            reject(new Error(message));
          } else {

            resolve(receipt);
          }
        } else if (receipt.isError) {

          reject(new Error('Transaction Aborted'));

        }
      }
    });
  });
}

async function signAndSendTransaction(transaction, signer) {
  let nonceObj = {nonce: nonces.get(signer.address)};
  await sendTransaction(transaction, signer, nonceObj);
  nonces.set(signer.address, nonces.get(signer.address).addn(1));
}

async function generateOffchainKeys(api, keyType) {
  const PHRASE = mnemonicGenerate();
  await cryptoWaitReady();
  const newPair = new Keyring({ type: 'sr25519' }).addFromUri(PHRASE);
  await api.rpc.author.insertKey(keyType, PHRASE, u8aToHex(newPair.publicKey));
}

// Creates a Signatory Object
async function signatory(api, entity, signer) {
  let entityKey = entity.publicKey;
  let entityDid = await createIdentities(api, [entity], signer);

  let signatoryObj = {
      "Identity": entityDid,
      "AccountKey": entityKey
  }
  return signatoryObj;
}

// Creates a multiSig Key
async function createMultiSig( api, signer, dids, numOfSigners ) {

  let nonceObj = {nonce: nonces.get(signer.address)};
  const transaction = api.tx.multiSig.createMultisig(dids, numOfSigners);
  await sendTransaction(transaction, signer, nonceObj);

  nonces.set(signer.address, nonces.get(signer.address).addn(1));

}

async function jumpLightYears() {

  await api.tx.timestamp.set()
}

// this object holds the required imports for all the scripts
let reqImports = {
  ApiPromise,
  WsProvider,
  path,
  fs,
  nonces,
  transfer_amount,
  fail_count,
  sk_roles,
  prepend,
  ticker,
  createApi,
  createIdentities,
  initMain,
  blockTillPoolEmpty,
  generateKeys,
  generateEntityFromUri,
  distributePoly,
  addSigningKeys,
  authorizeJoinToIdentities,
  issueTokenPerDid,
  senderRules1,
  receiverRules1,
  createClaimRules,
  addClaimsToDids,
  tickerToDid,
  sendTransaction,
  generateStashKeys,
  generateEntity,
  signAndSendTransaction,
  distributePolyBatch,
  createIdentitiesWithExpiry,
  generateOffchainKeys,
  signatory,
  createMultiSig,
  u8aToHex,
  topUpIdentityBalance,
  keyToIdentityIds
};

export { reqImports };
