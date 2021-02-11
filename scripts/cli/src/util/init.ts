import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { cryptoWaitReady, blake2AsHex, mnemonicGenerate } from "@polkadot/util-crypto";
import { stringToU8a, u8aConcat, u8aFixLength, u8aToHex } from "@polkadot/util";
import BN from "bn.js";
import assert from "assert";
import fs from "fs";
import path from "path";
import cryptoRandomString from 'crypto-random-string';
import { AccountId, Address, Balance, Moment } from "@polkadot/types/interfaces/runtime";
import { Type } from "@polkadot/types";
import { SubmittableExtrinsic } from "@polkadot/api/types";
import { IKeyringPair, ISubmittableResult } from "@polkadot/types/types";
import { Bytes, Text, u8, u32, u64 } from '@polkadot/types/primitive';
import { KeyringPair } from "@polkadot/keyring/types";
import { some, none, ap, Option } from "fp-ts/lib/Option";
import { pipe } from 'fp-ts/lib/pipeable';
//import type { Option, Vec } from '@polkadot/types/codec';
import type { Codec } from '@polkadot/types/types';
import type { DispatchError, DispatchInfo, EventRecord, ExtrinsicStatus } from '@polkadot/types/interfaces';

enum CountryCode {
    AF,
    AX,
    AL,
    DZ,
    AS,
    AD,
    AO,
    AI,
    AQ,
    AG,
    AR,
    AM,
    AW,
    AU,
    AT,
    AZ,
    BS,
    BH,
    BD,
    BB,
    BY,
    BE,
    BZ,
    BJ,
    BM,
    BT,
    BO,
    BA,
    BW,
    BV,
    BR,
    VG,
    IO,
    BN,
    BG,
    BF,
    BI,
    KH,
    CM,
    CA,
    CV,
    KY,
    CF,
    TD,
    CL,
    CN,
    HK,
    MO,
    CX,
    CC,
    CO,
    KM,
    CG,
    CD,
    CK,
    CR,
    CI,
    HR,
    CU,
    CY,
    CZ,
    DK,
    DJ,
    DM,
    DO,
    EC,
    EG,
    SV,
    GQ,
    ER,
    EE,
    ET,
    FK,
    FO,
    FJ,
    FI,
    FR,
    GF,
    PF,
    TF,
    GA,
    GM,
    GE,
    DE,
    GH,
    GI,
    GR,
    GL,
    GD,
    GP,
    GU,
    GT,
    GG,
    GN,
    GW,
    GY,
    HT,
    HM,
    VA,
    HN,
    HU,
    IS,
    IN,
    ID,
    IR,
    IQ,
    IE,
    IM,
    IL,
    IT,
    JM,
    JP,
    JE,
    JO,
    KZ,
    KE,
    KI,
    KP,
    KR,
    KW,
    KG,
    LA,
    LV,
    LB,
    LS,
    LR,
    LY,
    LI,
    LT,
    LU,
    MK,
    MG,
    MW,
    MY,
    MV,
    ML,
    MT,
    MH,
    MQ,
    MR,
    MU,
    YT,
    MX,
    FM,
    MD,
    MC,
    MN,
    ME,
    MS,
    MA,
    MZ,
    MM,
    NA,
    NR,
    NP,
    NL,
    AN,
    NC,
    NZ,
    NI,
    NE,
    NG,
    NU,
    NF,
    MP,
    NO,
    OM,
    PK,
    PW,
    PS,
    PA,
    PG,
    PY,
    PE,
    PH,
    PN,
    PL,
    PT,
    PR,
    QA,
    RE,
    RO,
    RU,
    RW,
    BL,
    SH,
    KN,
    LC,
    MF,
    PM,
    VC,
    WS,
    SM,
    ST,
    SA,
    SN,
    RS,
    SC,
    SL,
    SG,
    SK,
    SI,
    SB,
    SO,
    ZA,
    GS,
    SS,
    ES,
    LK,
    SD,
    SR,
    SJ,
    SZ,
    SE,
    CH,
    SY,
    TW,
    TJ,
    TZ,
    TH,
    TL,
    TG,
    TK,
    TO,
    TT,
    TN,
    TR,
    TM,
    TC,
    TV,
    UG,
    UA,
    AE,
    GB,
    US,
    UM,
    UY,
    UZ,
    VU,
    VE,
    VN,
    VI,
    WF,
    EH,
    YE,
    ZM,
    ZW
}

interface PortfolioKind {
    Default: string,
    User: PortfolioNumber
}

interface TargetIdentity {
    PrimaryIssuanceAgent: string,
    Specific: IdentityId
}

interface Claim {
    Accredited: Partial<Scope>,
    Affiliate: Partial<Scope>,
    BuyLockup: Partial<Scope>,
    SellLockup: Partial<Scope>,
    CustomerDueDiligence: CddId,
    KnowYourCustomer: Partial<Scope>,
    Jurisdiction: [CountryCode, Partial<Scope>],
    Exempted: Partial<Scope>,
    Blocked: Partial<Scope>,
    InvestorUniqueness: [Partial<Scope>, ScopeId, CddId],
    NoData: string
}

interface ClaimType {
    Accredited: string,
    Affiliate: string,
    BuyLockup: string,
    SellLockup: string,
    CustomerDueDiligence: string,
    KnowYourCustomer: string,
    Jurisdiction: string,
    Exempted: string,
    Blocked: string,
    InvestorUniqueness: string,
    NoData: string
}

interface AuthorizationData {
    AttestPrimaryKeyRotation: IdentityId
    RotatePrimaryKey: IdentityId
    TransferTicker: Ticker
    TransferPrimaryIssuanceAgent: Ticker
    AddMultiSigSigner: AccountId
    TransferAssetOwnership: Ticker
    JoinIdentity: Permissions 
    PortfolioCustody: PortfolioId
    Custom: Ticker
    NoData: string
    TransferCorporateActionAgent: Ticker
}

interface ConditionType  {
    IsPresent: Partial<Claim>
    IsAbsent: Partial<Claim>
    IsAnyOf: Partial<Claim>[]
    IsNoneOf: Partial<Claim>[]
    IsIdentity: Partial<TargetIdentity>
}

interface TrustedFor {
    Any: string
    Specific: Partial<ClaimType>[]
}

//Types
type IdentityId = [u8:  32];
type Ticker = [u8, 12];
type NonceObject = {nonce: string};
type PortfolioNumber = u64;
type ScopeId = [u8, 32];
type CddId = [u8, 32];
type PalletName = string;
type DispatchableName = string;

type Permissions = {
    asset: Option<Ticker[]>,
    extrinsic: Option<PalletPermissions[]>,
    portfolio: Option<PortfolioId[]>
}

type PalletPermissions = {
    pallet_name: PalletName,
    dispatchable_names: Option<DispatchableName[]>
}

type PortfolioId = {
    did: IdentityId,
    kind: Partial<PortfolioKind>
}

type TickerRegistration = {
    owner: IdentityId,
    expiry: Option<Moment>
}

type Authorization = {
    authorization_data: Partial<AuthorizationData>,
    authorized_by: IdentityId,
    expiry: Option<Moment>,
    auth_id: u64
}
type Scope = {
        Identity: IdentityId,
        Ticker: Ticker,
        Custom: u8[]
}

type TrustedIssuer = {
    issuer: IdentityId,
    trusted_for: Partial<TrustedFor>
}

type Condition = {
    condition_type: Partial<ConditionType>,
    issuers: TrustedIssuer[]
}

type ComplianceRequirement = {
    sender_conditions: Condition[],
    receiver_conditions: Condition[],
    id: u32
}

type AssetCompliance = {
    is_paused: Boolean,
    requirements: ComplianceRequirement[]
}

let nonces = new Map();
let totalPermissions: Permissions =
{
  "asset": some([]),
  "extrinsic": some([]),
  "portfolio": some([])
};

let fail_count = 1;
let block_sizes: Number[];
let block_times: Number[];
let synced_block = 0;
let synced_block_ts = 0;

// Amount to seed each key with
let transfer_amount = new BN(25000).mul(new BN(10).pow(new BN(6)));

const senderConditions1 = function (trusted_did: IdentityId, data: Partial<Scope>) {
  return [
    {
      "condition_type": {
        "IsPresent": {
          "Exempted": data
        }
      },
      issuers: [{ "issuer": trusted_did, "trusted_for": { "Any": "" } }]
    },
  ];
};

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
  const filePath = path.join(
    __dirname + "/../../../polymesh_schema.json"
  );
  const { types } = JSON.parse(fs.readFileSync(filePath, "utf8"));

  // Start node instance
  const ws_provider = new WsProvider(process.env.WS_PROVIDER || "ws://127.0.0.1:9944/");
  const api = await ApiPromise.create({
    types,
    provider: ws_provider,
  });
  return api;
};

export async function generateEntity(api: ApiPromise, name: string) {

  await cryptoWaitReady();
  let entity = new Keyring({ type: "sr25519" }).addFromUri(`//${name}`, {
    name: `${name}`,
  });
  let entityRawNonce = (await api.query.system.account(entity.address)).nonce;
  let entity_nonce = new BN(entityRawNonce.toString());
  nonces.set(entity.address, entity_nonce);

  return entity;
};

export async function generateKeys(api: ApiPromise, numberOfKeys: Number, keyPrepend: String) {
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
    let accountRawNonce = (await api.query.system.account(keys[i].address))
      .nonce;
    let account_nonce = new BN(accountRawNonce.toString());
    nonces.set(keys[i].address, account_nonce);
  }
  return keys;
};

export async function generateEntityFromUri(api: ApiPromise, uri: string) {
  await cryptoWaitReady();
  let entity = new Keyring({ type: "sr25519" }).addFromUri(uri);
  let accountRawNonce = (await api.query.system.account(entity.address)).nonce;
  let account_nonce = new BN(accountRawNonce.toString());
  nonces.set(entity.address, account_nonce);
  return entity;
};

export async function generateRandomEntity(api: ApiPromise) {
  let entity = await generateEntityFromUri(api, cryptoRandomString({length: 10}));
  return entity;
}

export async function generateRandomTicker() {
  let ticker = cryptoRandomString({length: 12, type: 'distinguishable'});
  return ticker;
}

export async function generateRandomKey() {
  let ticker = cryptoRandomString({length: 12, type: 'alphanumeric'});
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
        let new_block_ts = parseInt(
          JSON.stringify(timestamp_extrinsic["method"].args[0].toHuman()));
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
};

export async function createIdentities(api: ApiPromise, accounts: KeyringPair[], alice: KeyringPair) {
  return await createIdentitiesWithExpiry(api, accounts, alice, []);
};

async function createIdentitiesWithExpiry(
  api: ApiPromise,
  accounts: KeyringPair[],
  alice: KeyringPair,
  expiries: Moment[]
) {
  let dids = [];

  for (let i = 0; i < accounts.length; i++) {
    let account_did = await keyToIdentityIds(api, accounts[i].publicKey);

    if(account_did == 0) {
        console.log( `>>>> [Register CDD Claim] acc: ${accounts[i].address}`);
        const transaction = await api.tx.identity.cddRegisterDid(accounts[i].address, []);
        await sendTx(alice, transaction);
    }
    else {
        console.log('Identity Already Linked.');
    }
  }
  await blockTillPoolEmpty(api);

  for (let i = 0; i < accounts.length; i++) {
    const d = await api.query.identity.keyToIdentityIds(accounts[i].publicKey);
    dids.push(d.toHuman());
    console.log(`>>>> [Get DID ] acc: ${accounts[i].address} did: ${dids[i]}`);
  }

  // Add CDD Claim with CDD_ID
  for (let i = 0; i < dids.length; i++) {
    const cdd_id_byte = (i + 1).toString(16).padStart(2, "0");
    const claim = {
      CustomerDueDiligence: `0x00000000000000000000000000000000000000000000000000000000000000${cdd_id_byte}`,
    };
    const expiry = expiries.length == 0 ? null : expiries[i];

    console.log(`>>>> [add CDD Claim] did: ${dids[i]}, claim: ${JSON.stringify(claim)}`);
    await api.tx.identity
      .addClaim(dids[i], claim, expiry)
      .signAndSend(alice, { nonce: nonces.get(alice.address) });

    nonces.set(alice.address, nonces.get(alice.address).addn(1));
  }

  return dids;
};

// Fetches DID that belongs to the Account Key
export async function keyToIdentityIds(api: ApiPromise, accountKey: AccountId | KeyringPair["publicKey"]) {
  let account_did = await api.query.identity.keyToIdentityIds(accountKey);
  return account_did.toHuman();
}

// Sends transfer_amount to accounts[] from alice
export async function distributePoly(api: ApiPromise, to: KeyringPair, amount: Balance, from: KeyringPair) {
  // Perform the transfers
  const transaction = api.tx.balances.transfer(to.address, amount);
  await sendTx(from, transaction);
}

export async function distributePolyBatch(api: ApiPromise, to: KeyringPair[], amount: Balance, from: KeyringPair) {
  // Perform the transfers
  for (let i = 0; i < to.length; i++) {
    await distributePoly(api, to[i], amount, from);
  }
}

// Attach a secondary key to each DID
export async function addSecondaryKeys(api: ApiPromise, accounts: KeyringPair[], dids: IdentityId[], secondary_accounts: KeyringPair[]) {
  for (let i = 0; i < accounts.length; i++) {
    // 1. Add Secondary Item to identity.
    const transaction = api.tx.identity.addAuthorization({ Account: secondary_accounts[i].publicKey }, { JoinIdentity: totalPermissions }, null);
    await sendTx(accounts[i], transaction);
  }
}

// Authorizes the join of secondary keys to a DID
export async function authorizeJoinToIdentities(api: ApiPromise, accounts: KeyringPair[], dids: IdentityId[], secondary_accounts: KeyringPair[]) {

  for (let i = 0; i < accounts.length; i++) {
    // 1. Authorize
    const auths = await api.query.identity.authorizations.entries({
      Account: secondary_accounts[i].publicKey
    }) as unknown as Authorization[][];
    let last_auth_id = 0;
    for (let i = 0; i < auths.length; i++) {
      if (auths[i][1].auth_id.toNumber() > last_auth_id) {
        last_auth_id = auths[i][1].auth_id.toNumber();
      }
    }

    const transaction = api.tx.identity.joinIdentityAsKey([last_auth_id]);
    await sendTx(secondary_accounts[i], transaction);
  }

  return dids;
}

// Creates a token for a did
export async function issueTokenPerDid(api: ApiPromise, accounts: KeyringPair[], ticker: Ticker, amount: Balance, fundingRound: string) {
  
  assert(ticker.length <= 12, "Ticker cannot be longer than 12 characters");
  let tickerExist = (await api.query.asset.tickers(ticker)) as unknown as TickerRegistration;
 
  if (tickerExist.owner == [0]) {
    const transaction = api.tx.asset.createAsset(
      ticker, ticker, amount, true, 0, [], fundingRound
    );
    await sendTx(accounts[0], transaction);
  } else {
    console.log("ticker exists already");
  }
}

// Returns the asset did
export function tickerToDid(ticker: Ticker) {
    let tickerString = String.fromCharCode.apply(ticker);
    let tickerUintArray = Uint8Array.from(tickerString, x => x.charCodeAt(0))
    return blake2AsHex(
        u8aConcat(
        stringToU8a("SECURITY_TOKEN:"),
        u8aFixLength(tickerUintArray, 96, true)
        )
    );
}

// Creates claim compliance for an asset
export async function createClaimCompliance(api: ApiPromise, accounts: KeyringPair[], dids: IdentityId[], ticker: Ticker) {

  assert(ticker.length <= 12, "Ticker cannot be longer than 12 characters");

  let senderConditions = senderConditions1(dids[1], { "Ticker": ticker });
  let receiverConditions = receiverConditions1(dids[1], { "Ticker": ticker });

  const transaction = api.tx.complianceManager.addComplianceRequirement(
    ticker,
    senderConditions,
    receiverConditions
  );
  await sendTx(accounts[0], transaction);
}

// TODO Refactor function to deal with all the possible claim types and their values
export async function addClaimsToDids(
  api: ApiPromise,
  accounts: KeyringPair[],
  did: IdentityId,
  claimType: string,
  claimValue: Partial<Scope>,
  expiry: Option<Moment>
) {
  // Receieving Conditions Claim
  let claim = {[claimType]: claimValue};

  some(expiry) ? null : expiry;
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
    let accountRawNonce = (await api.query.system.account(keys[i].address))
      .nonce;
    let account_nonce = new BN(accountRawNonce.toString());
    nonces.set(keys[i].address, account_nonce);
  }
  return keys;
};

export function sendTransaction(transaction: SubmittableExtrinsic<"promise">, signer: KeyringPair, nonceObj: NonceObject) {
  return new Promise((resolve, reject) => {
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
              const dispatchError = failed.event.data[0] as unknown as DispatchError;

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
  let entityKey = entity.publicKey;
  let entityDid = await createIdentities(api, [entity], signer);

  let signatoryObj = {
    Identity: entityDid,
    Account: entityKey,
  };
  return signatoryObj;
}

// Creates a multiSig Key
export async function createMultiSig(api: ApiPromise, signer: KeyringPair, dids: IdentityId[], numOfSigners: u8) {
  const transaction = api.tx.multiSig.createMultisig(dids, numOfSigners);
  await sendTx(signer, transaction);
}

export async function mintingAsset(api: ApiPromise, minter: KeyringPair, did: IdentityId, ticker: Ticker) {
  const transaction = api.tx.asset.issue(ticker, 100);
  await sendTx(minter, transaction);
}

export async function sendTx(signer: KeyringPair, tx: SubmittableExtrinsic<"promise">) {
  let nonceObj = { nonce: nonces.get(signer.address) };
  const result = await sendTransaction(tx, signer, nonceObj) as unknown as ISubmittableResult;
  const passed = result.findRecord("system", "ExtrinsicSuccess");
  if (!passed) return -1;
  nonces.set(signer.address, nonces.get(signer.address).addn(1));
}

export async function addComplianceRequirement(api: ApiPromise, sender: KeyringPair, ticker: Ticker) {

  let assetCompliance = await api.query.complianceManager.assetCompliances(ticker) as unknown as AssetCompliance;

  if (assetCompliance.requirements.length == 0) {
    const transaction = await api.tx.complianceManager.addComplianceRequirement(
      ticker,
      [],
      []
    );

    await sendTx(sender, transaction);
  } else {
    console.log("Asset already has compliance.");
  }
}

export async function createVenue(api: ApiPromise, sender: KeyringPair) {
  let venueCounter = await api.query.settlement.venueCounter();
  let venueDetails = [0];

  const transaction = await api.tx.settlement.createVenue(venueDetails, [
    sender.address,
  ], 0);

  await sendTx(sender, transaction);

  return venueCounter;
}

export function getDefaultPortfolio(did: IdentityId) {
  return { "did": did, "kind": "Default" };
}

export async function affirmInstruction(api: ApiPromise, sender: KeyringPair, instructionCounter: u64, did: IdentityId, leg_counts: u64) {
  const transaction = await api.tx.settlement.affirmInstruction(
    instructionCounter,
    [getDefaultPortfolio(did)],
    leg_counts
  );

  await sendTx(sender, transaction);
}

export async function withdrawInstruction(api: ApiPromise, sender: KeyringPair, instructionCounter: u64, did: IdentityId) {
  const transaction = await api.tx.settlement.withdrawInstruction(
    instructionCounter,
    [getDefaultPortfolio(did)]
  );

  await sendTx(sender, transaction);
}

export async function rejectInstruction(api: ApiPromise, sender: KeyringPair, instructionCounter: u64, did: IdentityId) {
  const transaction = await api.tx.settlement.rejectInstruction(
    instructionCounter,
    [getDefaultPortfolio(did)]
  );

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
    transaction = await api.tx.settlement.addInstruction(
      venueCounter,
      0,
      null,
      null,
      [leg]
    );
  } else {
    transaction = await api.tx.settlement.addInstruction(
      venueCounter,
      0,
      null,
      null,
      [leg, leg2]
    );
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

  const transaction = await api.tx.settlement.claimReceipt(
    instructionCounter,
    receiptDetails
  );

  await sendTx(sender, transaction);
}

