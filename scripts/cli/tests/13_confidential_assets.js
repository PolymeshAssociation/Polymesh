// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");
let { reqImports } = require("../util/init.js");

const {
  create_account,
  create_mediator_account,
  mint_asset,
  create_transaction,
  finalize_transaction,
  justify_transaction,
  Account,
  PubAccount,
  decrypt,
} = require("../../../target/release/build/pkg/mercat_wasm");
// TODO This is based on my directory structure.
//} = require("../../../../cryptography/mercat/wasm/pkg/mercat_wasm");


async function main() {
  const api = await reqImports.createApi();

  const ticker = await reqImports.generateRandomTicker(api);
  const ticker2 = await reqImports.generateRandomTicker(api);
  const tickerHexList = ["01", "02"];
  const testEntities = await reqImports.initMain(api);

  let alice = testEntities[0];
  let bob = await reqImports.generateRandomEntity(api);
  let charlie = await reqImports.generateRandomEntity(api);

  let alice_did = await reqImports.keyToIdentityIds(api, alice.publicKey);

  let dids = await reqImports.createIdentities(api, [bob, charlie], alice);
  let bob_did = dids[0];
  let charlie_did = dids[1];
  await reqImports.distributePolyBatch(
    api,
    [bob, charlie],
    reqImports.transfer_amount,
    alice
  );

  console.log("Alice: ", alice_did);
  console.log("Bob: ", bob_did);
  console.log("Charlie: ", charlie_did);

  
  // Alice creates Confidential Assets 
  console.log("-----------> Creating confidential assets.");
  await createConfidentialAsset(api, "0x01", alice);
  await createConfidentialAsset(api, "0x02", alice);

  // Alice and Bob create their Mercat account locally and submit the proof to the chain
  console.log("-----------> Creating Alice and Bob's mercat accounts.");
  const aliceMercatInfo = await create_account(tickerHexList, tickerHexList[0]);
  const bobMercatInfo = await create_account(tickerHexList, tickerHexList[0]);

  // Validate Alice and Bob's Mercat Accounts
  console.log("-----------> Submitting alice mercat account proofs.");
  await validateMercatAccount(api, alice, aliceMercatInfo.account_tx);
  console.log("-----------> Submitting bob mercat account proofs.");
  await validateMercatAccount(api, bob, bobMercatInfo.account_tx);

  // Charlie creates his mediator Mercat Account 
  console.log("-----------> Creating Charlie's account.");
  const charliePublicKey = await createMercatMediatorAccount();

  // Validate Charlie's Mercat Account 
  console.log("-----------> Submitting Charlie's account.");
  await addMediatorMercatAccount(api, charlie, charliePublicKey);

  let aliceBalance = await displayBalance(api, alice_did, aliceMercatInfo, "Alice initial balance");
  let bobBalance = await displayBalance(api, bob_did, bobMercatInfo, "Bob initial balance");
  
  // Mint Tokens 
  console.log("-----------> Minting assets.");
  await mintTokens(api, alice, "0x01", 1000, aliceMercatInfo);

  await displayBalance(api, alice_did, aliceMercatInfo, "Alice balance after minting");

  // Create Venue
  console.log("-----------> Creating venue.");
  const venueCounter = await reqImports.createVenue(api, charlie);
 
  // Create Confidential Instruction
  console.log("-----------> Creating confidential instruction.");
  const instructionCounter = await reqImports.addConfidentialInstruction(
    api,
    venueCounter,
    charlie,
    alice_did,
    bob_did,
    charlie_did,
    aliceMercatInfo.account_id,
    bobMercatInfo.account_id
  );
  
  console.log("-----------> Initializing confidential transaction.");
  const bobPublicAccount = new PubAccount(bobMercatInfo.account_id, bobMercatInfo.public_key);
  const aliceEncryptedBalance = await getEncryptedBalance(api, alice_did, aliceMercatInfo.account_id);
  const initTransactionProof = await createTransaction(
    100,
    aliceMercatInfo,
    bobPublicAccount,
    charliePublicKey,
    aliceEncryptedBalance
  );

  console.log("-----------> Submitting initial confidential transaction proof.");
  await affirmConfidentialInstruction(api, instructionCounter, {InitializedTransfer: initTransactionProof}, alice, alice_did);

  console.log("-----------> Finalizing confidential transaction.");
  const finalizeTransactionProof = await finalizeTransaction(
    100,
    initTransactionProof,
    bobMercatInfo,
  );

  console.log("-----------> Submitting finalize confidential transaction proof.");
  await affirmConfidentialInstruction(api, instructionCounter, {FinalizedTransfer: finalizeTransactionProof}, bob, bob_did);


  // TODO: call justify_transaction similar to the way I called create and finalized tranasctions.
  // TODO: similarly affirm the justified_tx proof.
  // TODO: assert that the plain balances are correct after decrypting them.

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

function encodeToBase64(data) {
    let buffer = Buffer.from(data);
    return buffer.toString('base64');
}

async function finalizeTransaction(amount, initializeProof, receiverMercatAccountInfo) {
  const receiverPublicAccount = new PubAccount(receiverMercatAccountInfo.account_id, receiverMercatAccountInfo.public_key);
  const receiverAccount = new Account(receiverMercatAccountInfo.secret_account, receiverPublicAccount);

  let tx = finalize_transaction(amount, initializeProof, receiverAccount);

  return tx.finalized_tx;
}

async function affirmConfidentialInstruction(api, instruction_id, proof, signer, signer_did) {
    const portfolio = reqImports.getDefaultPortfolio(signer_did);
    const transaction = await api.tx.settlement.affirmConfidentialInstruction(instruction_id, proof, [portfolio]);
    let tx = await reqImports.sendTx(signer, transaction);
    if(tx !== -1) reqImports.fail_count--;
}

async function createTransaction(amount, senderMercatAccountInfo, receiverPubAccount, mediatorPublicKey, encrypted_pending_balance) {
  const senderPublicAccount = new PubAccount(senderMercatAccountInfo.account_id, senderMercatAccountInfo.public_key);
  const senderAccount = new Account(senderMercatAccountInfo.secret_account, senderPublicAccount);

  let tx = await create_transaction(amount, senderAccount, encrypted_pending_balance, receiverPubAccount, mediatorPublicKey);

  return tx.init_tx;
}

async function displayBalance(api, did, mercatAccountInfo, message) {
    // Get encrypted balance
    const accountEncryptedBalance = await getEncryptedBalance(api, did, mercatAccountInfo.account_id);
    // Decrypt balance
    const accountBalance = await decryptBalances(accountEncryptedBalance, mercatAccountInfo);
    console.log(`${message}: ${accountBalance}`);

    return accountEncryptedBalance;
}

async function mintTokens(api, signer, tickerHex, amount, mercatAccountInfo) {
  const publicAccount = new PubAccount(mercatAccountInfo.account_id,  mercatAccountInfo.public_key);
  const account = new Account(mercatAccountInfo.secret_account, publicAccount);
  const mintTxInfo = new mint_asset(amount, account);
  const mintProof = mintTxInfo.asset_tx;

  const transaction = await api.tx.confidentialAsset.mintConfidentialAsset(tickerHex, amount, mintProof);
  let tx = await reqImports.sendTx(signer, transaction);
  if(tx !== -1) reqImports.fail_count--;
}

async function getEncryptedBalance(api, did, mercatAccountID){
    return await api.query.confidentialAsset.mercatAccountBalance(did, mercatAccountID);
}

async function decryptBalances(encryptedBalance, mercatAccountInfo) {
  const publicAccount = new PubAccount(mercatAccountInfo.account_id,  mercatAccountInfo.public_key);
  const account = new Account(mercatAccountInfo.secret_account, publicAccount);

  return decrypt(encryptedBalance, account);
}

async function removeChainDir(chain_dir) {
    await exec(`rm -rf ${chain_dir}`);
}

async function addMediatorMercatAccount(api, signer, public_key) {
    const transaction = await api.tx.confidentialAsset.addMediatorMercatAccount(public_key);
    
    let tx = await reqImports.sendTx(signer, transaction);
    if(tx !== -1) reqImports.fail_count--;
}

async function validateMercatAccount(api, signer, proof) {
    const transaction = await api.tx.confidentialAsset.validateMercatAccount(proof);
    
    let tx = await reqImports.sendTx(signer, transaction);
    if(tx !== -1) reqImports.fail_count--;
}

async function createMercatMediatorAccount() {
  const charlieMercatInfo = create_mediator_account();
  
  return charlieMercatInfo.public_key;
}

async function createConfidentialAsset(api, ticker, signer) {

    const transaction = await api.tx.confidentialAsset.createConfidentialAsset(
        ticker,
        ticker,
        true,
        0,
        [],
        null
      );
    
      let tx = await reqImports.sendTx(signer, transaction);
      if(tx !== -1) reqImports.fail_count--;
}

main().catch(console.error);
