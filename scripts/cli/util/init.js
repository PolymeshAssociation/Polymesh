const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { cryptoWaitReady } = require("@polkadot/util-crypto");
const BN = require("bn.js");
const assert = require("assert");
const fs = require("fs");
const path = require("path");
const { blake2AsHex } = require('@polkadot/util-crypto');
const { stringToU8a, u8aConcat, u8aFixLength } = require('@polkadot/util');

let nonces = new Map();
let sk_roles = [[0], [1], [2], [1, 2]];

let fail_count = 0;
let block_sizes = {};
let block_times = {};

let synced_block = 0;
let synced_block_ts = 0;

// Amount to seed each key with
let transfer_amount = 10 * 10 ** 12;
let prepend = "demo";

// Used for creating a single ticker 
const ticker = `token${prepend}0`.toUpperCase();
assert( ticker.length <= 12, "Ticker cannot be longer than 12 characters");

const senderRules1 = function(trusted_did, asset_did) {
    return [
    {
      "rule_type": {
        "IsPresent": {
          "Whitelisted": asset_did 
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
          "Whitelisted": asset_did
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
  entities.push(alice);
  entities.push(bob);

  return entities;
}

let generateEntity = async function (api, name) {
  let entity = [];
  await cryptoWaitReady();
  entity = new Keyring({ type: "sr25519" }).addFromUri(`//${name}`, { name: `${name}` });
  let entityRawNonce = await api.query.system.accountNonce(entity.address);
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
    let accountRawNonce = await api.query.system.accountNonce(keys[i].address);
    let account_nonce = new BN(accountRawNonce.toString());
    nonces.set(keys[i].address, account_nonce);
  }
  return keys;
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
const createIdentities = async function (api, accounts) {
  let dids = [];

  for (let i = 0; i < accounts.length; i++) {
    await api.tx.identity
      .registerDid([])
      .signAndSend(accounts[i], { nonce: nonces.get(accounts[i].address) });

    nonces.set(accounts[i].address, nonces.get(accounts[i].address).addn(1));
  }
  await blockTillPoolEmpty(api);
  for (let i = 0; i < accounts.length; i++) {
    const d = await api.query.identity.keyToIdentityIds(accounts[i].publicKey);
    dids.push(d.raw.asUnique);
  }
  return dids;
};

// Sends transfer_amount to accounts[] from alice
async function distributePoly(api, accounts, transfer_amount, signingEntity) {
  // Perform the transfers
  for (let i = 0; i < accounts.length; i++) {
    const unsub = await api.tx.balances
      .transfer(accounts[i].address, transfer_amount)
      .signAndSend(
        signingEntity,
        { nonce: nonces.get(signingEntity.address) });

    nonces.set(signingEntity.address, nonces.get(signingEntity.address).addn(1));
  }
}

// Attach a signing key to each DID
async function addSigningKeys(api, accounts, dids, signing_accounts) {
  for (let i = 0; i < accounts.length; i++) {
    // 1. Add Signing Item to identity.

    const unsub = await api.tx.identity
      .addAuthorizationAsKey({ AccountKey: signing_accounts[i].publicKey }, { JoinIdentity: dids[i] }, 0)
      .signAndSend(accounts[i], { nonce: nonces.get(accounts[i].address) });

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
    const unsub = await api.tx.identity
      .joinIdentityAsKey([last_auth_id])
      .signAndSend(signing_accounts[i], { nonce: nonces.get(signing_accounts[i].address) });
    nonces.set(signing_accounts[i].address, nonces.get(signing_accounts[i].address).addn(1));
  }

  return dids;
}

// Used to make the functions in scripts more efficient
async function callback(status, events, sectionName, methodName, fail_count) {
  let new_did_ok = false;
  events.forEach(({ phase, event: { data, method, section } }) => {
    if (section == sectionName && method == methodName) {
      new_did_ok = true;
    }
  });

  if (!new_did_ok) {
    fail_count++;
  }

  return fail_count;
}

// Creates a token for a did
async function issueTokenPerDid(api, accounts) {

    const unsub = await api.tx.asset
      .createToken(ticker, ticker, 1000000, true, 0, [], "abc")
      .signAndSend(accounts[0], { nonce: nonces.get(accounts[0].address) });

    nonces.set(accounts[0].address, nonces.get(accounts[0].address).addn(1));
  
}

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

    const unsub = await api.tx.generalTm
      .addActiveRule(ticker, senderRules, receiverRules)
      .signAndSend( accounts[0], { nonce: nonces.get(accounts[0].address) });

      nonces.set(accounts[0].address, nonces.get(accounts[0].address).addn(1));
    
  
}

// Adds claim to did
async function addClaimsToDids(api, accounts, did, claimType, claimValue) {

  // Receieving Rules Claim
  let claim = {[claimType]: claimValue};

      const unsub = await api.tx.identity
      .addClaim(did, claim, null)
      .signAndSend(accounts[1],
        { nonce: nonces.get(accounts[1].address) });

    nonces.set(accounts[1].address, nonces.get(accounts[1].address).addn(1));
    
}

// this object holds the required imports for all the scripts
let reqImports = {
  path,
  ApiPromise,
  WsProvider,
  createIdentities,
  initMain,
  blockTillPoolEmpty,
  generateKeys,
  fs,
  callback,
  nonces,
  transfer_amount,
  fail_count,
  distributePoly,
  addSigningKeys,
  authorizeJoinToIdentities,
  sk_roles,
  prepend,
  issueTokenPerDid,
  senderRules1,
  receiverRules1,
  createClaimRules,
  addClaimsToDids,
  ticker,
  tickerToDid
};

export { reqImports };
