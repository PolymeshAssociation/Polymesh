import {
  initMain,
  generateEntityFromUri,
  keyToIdentityIds,
  disconnect,
  transferAmount,
  padTicker,
  waitBlocks,
  sendTx,
  ApiSingleton,
} from "../util/init";
import type { Ticker } from "../types";
import { createIdentities } from "../helpers/identity_helper";
import { distributePoly } from "../helpers/poly_helper";
import { IdentityId } from "../interfaces";

import { assert } from "chai";
import type { KeyringPair } from "@polkadot/keyring/types";
import { stringToU8a, u8aConcat, compactToU8a } from "@polkadot/util";

import {
  create_account,
  create_mediator_account,
  mint_asset,
  create_transaction,
  finalize_transaction,
  justify_transaction,
  CreateAccountOutput,
  Account,
  PubAccount,
  decrypt,
} from "mercat-wasm";

type TransactionLegId = number;

interface AffirmParty {
  Sender?: Uint8Array,
  Receiver?: string,
  Mediator?: string,
}

type AffirmLeg = {
  leg_id: TransactionLegId,
  party: AffirmParty,
};

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

// Seeds needs to be at least 32 bytes.  These are bad seeds, please use a crypt-rng.
function makeSeed(name: string): Uint8Array {
  const seed = `${name}::Confidential Asset Test Seed         `;
  return stringToU8a(seed.substr(0, 32));
}

describe("17 - Confidential Asset Unit Test", () => {
  test("Basic transfer", async () => {
    const ticker = padTicker("17ATICKER");
    const testEntities = await initMain();
    const alice = testEntities[0];
    const bob = await generateEntityFromUri("17_bob");
    const bobDid = (await createIdentities(alice, [bob]))[0];
    expect(bobDid).toBeTruthy();
    const charlie = await generateEntityFromUri("17_charlie");
    const charlieDid = (await createIdentities(alice, [charlie]))[0];
    const dave = await generateEntityFromUri("17_dave");
    const daveDid = (await createIdentities(alice, [dave]))[0];
    const aliceDid = await keyToIdentityIds(alice.publicKey);
    await expect(
      distributePoly(alice, bob, transferAmount)
    ).resolves.not.toThrow();
    await expect(
      distributePoly(alice, charlie, transferAmount)
    ).resolves.not.toThrow();
    await expect(
      distributePoly(alice, dave, transferAmount)
    ).resolves.not.toThrow();

    console.log("Alice: ", aliceDid);
    console.log("Bob: ", bobDid);
    console.log("Charlie: ", charlieDid);
    console.log("Dave: ", daveDid);

    // Dave creates Confidential Assets
    console.log("-----------> Creating confidential assets.");
    await createConfidentialAsset(dave, ticker);

    // Dave and Bob create their Mercat account locally and submit the proof to the chain
    console.log("-----------> Creating Dave and Bob's mercat accounts.");
    const daveMercat = create_account(makeSeed("17_dave"));
    const daveMercatAccount = daveMercat.account;
    const davePubAccount = daveMercatAccount.public_account;
    const bobMercat = create_account(makeSeed("17_bob"));
    const bobMercatAccount = bobMercat.account;
    const bobPubAccount = bobMercatAccount.public_account;

    // Validate Dave and Bob's Mercat Accounts
    console.log("-----------> Submitting dave mercat account proofs.");
    await validateMercatAccount(dave, daveMercat.account_tx, ticker);
    console.log("-----------> Submitting bob mercat account proofs.");
    await validateMercatAccount(bob, bobMercat.account_tx, ticker);

    // Charlie creates his mediator Mercat Account
    console.log("-----------> Creating Charlie's account.");
    const charlieMercat = create_mediator_account(makeSeed("17_charlie"));

    // Validate Charlie's Mercat Account
    console.log("-----------> Submitting Charlie's account.");
    await addMediatorMercatAccount(charlie, charlieMercat.public_account);

    await displayBalance(daveMercatAccount, ticker, "Dave initial balance");
    await displayBalance(bobMercatAccount, ticker, "Bob initial balance");

    // Mint Tokens
    console.log("-----------> Minting assets.");
    await mintTokens(makeSeed("17_mint"), dave, ticker, 1000, daveMercatAccount);

    await displayBalance(daveMercatAccount, ticker, "Dave balance after minting");

    // Create Venue
    console.log("-----------> Creating venue.");
    const venueId = await createVenue(charlie);

    // Create Confidential Instruction
    console.log("-----------> Creating confidential instruction.");
    const transactionId = await addConfidentialInstruction(
      charlie,
      venueId,
      ticker,
      davePubAccount,
      bobPubAccount,
      charlieDid,
    );
    let legId = 0;

    console.log("-----------> Initializing confidential transaction.");
    let daveEncryptedBalance = await getEncryptedBalance(davePubAccount, ticker);
    const pending_balance = decryptBalances(daveEncryptedBalance, daveMercatAccount);
    let senderProof = create_transaction(
      makeSeed("Dave_Tx_1"),
      100,
      daveMercatAccount,
      daveEncryptedBalance,
      pending_balance,
      bobPubAccount,
      undefined
    ).init_tx;

    console.log("-----------> Submitting initial confidential transaction proof.");
    await affirmTransaction(dave, transactionId, { leg_id: legId, party: { Sender: wrapTX(senderProof) } });

    console.log("-----------> Finalizing confidential transaction.");
    const finalizeTransactionProof = finalizeTransaction(
      100,
      senderProof,
      bobMercatAccount,
    );

    console.log("-----------> Receiver affirms transaction..");
    await affirmTransaction(bob, transactionId, { leg_id: legId, party: { Receiver: "" } });

    console.log("-----------> Justifying confidential transaction.");
    justifyTransaction(
      makeSeed("Charlie_tx_1"),
      senderProof,
      charlieMercat, davePubAccount, daveEncryptedBalance,
      bobPubAccount, 100);

    console.log("-----------> Mediator affirms transaction..");
    await affirmTransaction(charlie, transactionId, { leg_id: legId, party: { Mediator: "" } });

    console.log("-----------> execute transaction.");
    await executeTransaction(charlie, transactionId, 1);

    console.log("-----------> Receiver undates their balance");
    await applyIncomingBalance(bob, bobPubAccount, ticker);

    await displayBalance(daveMercatAccount, ticker, "Dave balance after giving tokens to Bob");
    await displayBalance(bobMercatAccount,  ticker, "Bob balance after getting tokens from Dave");

    daveEncryptedBalance = await getEncryptedBalance(davePubAccount, ticker);
    const bobEncryptedBalance = await getEncryptedBalance(bobPubAccount, ticker);

    assert.equal(decryptBalances(daveEncryptedBalance, daveMercatAccount), 900);
    assert.equal(decryptBalances(bobEncryptedBalance, bobMercatAccount), 100);
  });
});

function wrapTX(tx: Uint8Array): Uint8Array {
  return u8aConcat(
    compactToU8a(tx.length),
    tx
  );
}

async function validateMercatAccount(signer: KeyringPair, proof: Uint8Array, ticker: Ticker) {
  const api = await ApiSingleton.getInstance();
  const wrapped = wrapTX(proof);
  const transaction = await api.tx.confidentialAsset.validateMercatAccount(ticker, wrapped);

  await sendTx(signer, transaction);
}

async function addMediatorMercatAccount(signer: KeyringPair, account: PubAccount) {
  const api = await ApiSingleton.getInstance();
  const public_key = account.public_key;
  const transaction = await api.tx.confidentialAsset.addMediatorMercatAccount(public_key);

  await sendTx(signer, transaction);
}

async function createConfidentialAsset(signer: KeyringPair, ticker: Ticker) {
  const api = await ApiSingleton.getInstance();
  const transaction = await api.tx.confidentialAsset.createConfidentialAsset(
    ticker,
    ticker,
    { EquityCommon: "" }
  );

  await sendTx(signer, transaction);
}

async function mintTokens(
  seed: Uint8Array, signer: KeyringPair, ticker: Ticker, amount: number, account: Account
) {
  const api = await ApiSingleton.getInstance();
  const mintTxInfo = mint_asset(seed, amount, account);
  const mintProof = wrapTX(mintTxInfo.asset_tx);

  const transaction = await api.tx.confidentialAsset.mintConfidentialAsset(ticker, amount, mintProof);
  await sendTx(signer, transaction);
}

async function createVenue(signer: KeyringPair): Promise<number> {
  const api = await ApiSingleton.getInstance();
  let venueCounter = (await api.query.confidentialAsset.venueCounter()).toNumber();
  const transaction = api.tx.confidentialAsset.createVenue();
  await sendTx(signer, transaction);
  return venueCounter;
}

async function displayBalance(account: Account, ticker: Ticker, message: String): Promise<Uint8Array> {
  // Get encrypted balance
  const accountEncryptedBalance = await getEncryptedBalance(account.public_account, ticker);
  // Decrypt balance
  const accountBalance = decryptBalances(accountEncryptedBalance, account);
  console.log(`${message}: ${accountBalance}`);

  return accountEncryptedBalance;
}

async function getEncryptedBalance(account: PubAccount, ticker: Ticker): Promise<Uint8Array> {
  const api = await ApiSingleton.getInstance();
  return (await api.query.confidentialAsset.mercatAccountBalance(account.public_key, ticker)).unwrap().toU8a();
}

function decryptBalances(encryptedBalance: Uint8Array, account: Account): number {
  return decrypt(encryptedBalance, account);
}

async function addConfidentialInstruction(
  signer: KeyringPair,
  venueId: number,
  ticker: Ticker,
  senderAccount: PubAccount,
  receiverAccount: PubAccount,
  mediator: IdentityId,
): Promise<number> {
  const api = await ApiSingleton.getInstance();
  let transactionCounter = (await api.query.confidentialAsset.transactionCounter()).toNumber();
  let leg = {
    ticker: ticker,
    sender: senderAccount.public_key,
    receiver: receiverAccount.public_key,
    mediator: mediator,
  };
  const transaction = api.tx.confidentialAsset.addTransaction(
    venueId,
    [leg],
    null,
  );
  await sendTx(signer, transaction);
  return transactionCounter;
}

function finalizeTransaction(
  amount: number, initializeProof: Uint8Array, receiverAccount: Account
) {
  finalize_transaction(amount, initializeProof, receiverAccount);
}

function justifyTransaction(
  seed: Uint8Array,
  initializeProof: Uint8Array,
  mediatorAccount: Account,
  senderAccount: PubAccount,
  senderEncryptedBalance: Uint8Array,
  receiverAccount: PubAccount,
  amount: number
) {
  justify_transaction(
    seed,
    initializeProof,
    mediatorAccount,
    senderAccount,
    senderEncryptedBalance,
    receiverAccount,
    amount);
}

async function affirmTransaction(signer: KeyringPair, transaction_id: number, affirm: AffirmLeg) {
  const api = await ApiSingleton.getInstance();
  const transaction = await api.tx.confidentialAsset.affirmTransaction(transaction_id, affirm);
  await sendTx(signer, transaction);
}

async function executeTransaction(signer: KeyringPair, transaction_id: number, legs: number) {
  const api = await ApiSingleton.getInstance();
  const transaction = await api.tx.confidentialAsset.executeTransaction(transaction_id, legs);
  await sendTx(signer, transaction);
}

async function applyIncomingBalance(signer: KeyringPair, account: PubAccount, ticker: Ticker) {
  const api = await ApiSingleton.getInstance();
  const transaction = await api.tx.confidentialAsset.applyIncomingBalance(account.public_key, ticker);
  await sendTx(signer, transaction);
}
