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
import { createIdentities } from "../helpers/identity_helper";
import { distributePoly } from "../helpers/poly_helper";

import {
  create_account,
  create_mediator_account,
  mint_asset,
  create_transaction,
  finalize_transaction,
  justify_transaction,
  Account,
  PubAccount,
  decrypt,
} from "../../mercat/";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("17 - Confidential Asset Unit Test", () => {
  test("Basic transfer", async () => {
    // Seeds needs to be at least 32 bytes.  These are bad seeds, please use a crypt-rng.
    const seed_bob = "Confidential Asset Test Seed:  1";
    const seed_charlie = "Confidential Asset Test Seed:  2";
    const seed_dave = "Confidential Asset Test Seed:  3";
    const seed4 = "Confidential Asset Test Seed:  4";
    const seed5 = "Confidential Asset Test Seed:  5";

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
    const daveMercatInfo = create_account(seed_dave);
    const bobMercatInfo = create_account(seed_bob);

    // Validate Dave and Bob's Mercat Accounts
    console.log("-----------> Submitting dave mercat account proofs.");
    await validateMercatAccount(dave, daveMercatInfo.account_tx);
    console.log("-----------> Submitting bob mercat account proofs.");
    await validateMercatAccount(bob, bobMercatInfo.account_tx);

    // Charlie creates his mediator Mercat Account
    console.log("-----------> Creating Charlie's account.");
    const charlieMercatAccount = createMercatMediatorAccount(seed_charlie).account();
    const charliePublicKey = charlieMercatAccount.public_account;
    const charlieAccount = charlieMercatAccount.secret_account;

    // Validate Charlie's Mercat Account
    console.log("-----------> Submitting Charlie's account.");
    await addMediatorMercatAccount(charlie, charliePublicKey);

  });
});

function createMercatMediatorAccount() {
  const charlieMercatInfo = create_mediator_account();

  return charlieMercatInfo;
}

async function validateMercatAccount(signer: KeyringPair, proof) {
  const api = await ApiSingleton.getInstance();
  const transaction = await api.tx.confidentialAsset.validateMercatAccount(proof);

  await sendTx(signer, transaction);
}

async function addMediatorMercatAccount(signer: KeyringPair, public_key) {
  const api = await ApiSingleton.getInstance();
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

async function createVenue(signer: KeyringPair): Promise<number> {
  const api = await ApiSingleton.getInstance();
  let venueCounter = (await api.query.confidentialAsset.venueCounter()).toNumber();
  const transaction = api.tx.settlement.createVenue();
  await sendTx(signer, transaction);
  return venueCounter;
}

/*
async function main() {
  // Dave and Bob create their Mercat account locally and submit the proof to the chain
  console.log("-----------> Creating Dave and Bob's mercat accounts.");
  const daveMercatInfo = create_account(tickersHexList, tickerHex);
  const bobMercatInfo = create_account(tickersHexList, tickerHex);

  // Validate Dave and Bob's Mercat Accounts
  console.log("-----------> Submitting dave mercat account proofs.");
  await validateMercatAccount(api, dave, daveMercatInfo.account_tx);
  console.log("-----------> Submitting bob mercat account proofs.");
  await validateMercatAccount(api, bob, bobMercatInfo.account_tx);

  // Charlie creates his mediator Mercat Account
  console.log("-----------> Creating Charlie's account.");
  const charlieMercatAccount = createMercatMediatorAccount();
  const charliePublicKey = charlieMercatAccount.public_key;
  const charlieAccount = charlieMercatAccount.secret_account;

  // Validate Charlie's Mercat Account
  console.log("-----------> Submitting Charlie's account.");
  await addMediatorMercatAccount(api, charlie, charliePublicKey);

  await displayBalance(api, daveDid, daveMercatInfo, "Dave initial balance");
  await displayBalance(api, bobDid, bobMercatInfo, "Bob initial balance");

  // Mint Tokens
  console.log("-----------> Minting assets.");
  await mintTokens(api, dave, ticker, 1000, daveMercatInfo);

  await displayBalance(api, daveDid, daveMercatInfo, "Dave balance after minting");

  // Create Venue
  console.log("-----------> Creating venue.");
  const venueCounter = await createVenue(api, charlie);

  // Create Confidential Instruction
  console.log("-----------> Creating confidential instruction.");
  const instructionCounter = await addConfidentialInstruction(
    api,
    venueCounter,
    charlie,
    daveDid,
    bobDid,
    charlieDid,
    daveMercatInfo.account_id,
    bobMercatInfo.account_id
  );

  console.log("-----------> Initializing confidential transaction.");
  let bobPublicAccount = new PubAccount(bobMercatInfo.account_id, bobMercatInfo.public_key);
  let daveEncryptedBalance = await getEncryptedBalance(api, daveDid, daveMercatInfo.account_id);
  const initTransactionProof = createTransaction(
    100,
    daveMercatInfo,
    bobPublicAccount,
    charliePublicKey,
    daveEncryptedBalance
  );

  console.log("-----------> Submitting initial confidential transaction proof.");
  await affirmConfidentialInstruction(api, instructionCounter, {InitializedTransfer: initTransactionProof}, dave, daveDid);

  console.log("-----------> Finalizing confidential transaction.");
  const finalizeTransactionProof = finalizeTransaction(
    100,
    initTransactionProof,
    bobMercatInfo,
  );

  console.log("-----------> Submitting finalize confidential transaction proof.");
  bobPublicAccount = new PubAccount(bobMercatInfo.account_id, bobMercatInfo.public_key);
  await affirmConfidentialInstruction(api, instructionCounter, {FinalizedTransfer: finalizeTransactionProof}, bob, bobDid);

  const davePublicAccount = new PubAccount(daveMercatInfo.account_id, daveMercatInfo.public_key);

  console.log("-----------> Justifying confidential transaction.");
  const justifiedTransactionProof = (justify_transaction(finalizeTransactionProof, charlieAccount, davePublicAccount, daveEncryptedBalance, bobPublicAccount, tickerHex)).justified_tx;

  console.log("-----------> Submitting justify confidential transaction proof.");
  await affirmConfidentialInstruction(api, instructionCounter, {JustifiedTransfer: justifiedTransactionProof}, charlie, charlieDid);

  await displayBalance(api, daveDid, daveMercatInfo, "Dave balance after giving tokens to Bob");
  await displayBalance(api, bobDid, bobMercatInfo, "Bob balance after getting tokens from Dave");

  daveEncryptedBalance = await getEncryptedBalance(api, daveDid, daveMercatInfo.account_id);
  const bobEncryptedBalance = await getEncryptedBalance(api, bobDid, bobMercatInfo.account_id);

  assert.equal(decryptBalances(daveEncryptedBalance, daveMercatInfo), 900);
  assert.equal(decryptBalances(bobEncryptedBalance, bobMercatInfo), 100);

  if (fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

function finalizeTransaction(amount, initializeProof, receiverMercatAccountInfo) {
  const receiverPublicAccount = new PubAccount(receiverMercatAccountInfo.account_id, receiverMercatAccountInfo.public_key);
  const receiverAccount = new Account(receiverMercatAccountInfo.secret_account, receiverPublicAccount);

  let tx = finalize_transaction(amount, initializeProof, receiverAccount);

  return tx.finalized_tx;
}

async function affirmConfidentialInstruction(api, instruction_id, proof, signer, signerDid) {
    const portfolio = getDefaultPortfolio(signerDid);
    const transaction = await api.tx.settlement.affirmConfidentialInstruction(instruction_id, proof, [portfolio], 1);
    let tx = await sendTx(signer, transaction);
    if(tx !== -1) fail_count--;
}

function createTransaction(amount, senderMercatAccountInfo, receiverPubAccount, mediatorPublicKey, encrypted_pending_balance) {
  const senderPublicAccount = new PubAccount(senderMercatAccountInfo.account_id, senderMercatAccountInfo.public_key);
  const senderAccount = new Account(senderMercatAccountInfo.secret_account, senderPublicAccount);

  let tx = create_transaction(amount, senderAccount, encrypted_pending_balance, receiverPubAccount, mediatorPublicKey);

  return tx.init_tx;
}

async function displayBalance(api, did, mercatAccountInfo, message) {
    // Get encrypted balance
    const accountEncryptedBalance = await getEncryptedBalance(api, did, mercatAccountInfo.account_id);
    // Decrypt balance
    const accountBalance = decryptBalances(accountEncryptedBalance, mercatAccountInfo);
    console.log(`${message}: ${accountBalance}`);

    return accountEncryptedBalance;
}

async function mintTokens(api, signer, ticker, amount, mercatAccountInfo) {
  const publicAccount = new PubAccount(mercatAccountInfo.account_id,  mercatAccountInfo.public_key);
  const account = new Account(mercatAccountInfo.secret_account, publicAccount);
  const mintTxInfo = new mint_asset(amount, account);
  const mintProof = mintTxInfo.asset_tx;

  const transaction = await api.tx.confidentialAsset.mintConfidentialAsset(ticker, amount, mintProof);
  let tx = await sendTx(signer, transaction);
  if(tx !== -1) fail_count--;
}

async function getEncryptedBalance(api, did, mercatAccountID){
    return String(await api.query.confidentialAsset.mercatAccountBalance(did, mercatAccountID));
}

function decryptBalances(encryptedBalance, mercatAccountInfo) {
  const publicAccount = new PubAccount(mercatAccountInfo.account_id,  mercatAccountInfo.public_key);
  const account = new Account(mercatAccountInfo.secret_account, publicAccount);

  return decrypt(encryptedBalance, account);
}

*/
