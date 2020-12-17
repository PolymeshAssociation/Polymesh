// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

const util = require('util');
const exec = util.promisify(require('child_process').exec);

const {
  create_account,
  create_mediator_account,
  mint_asset,
  create_transaction,
  finalize_transaction,
  Account,
  PubAccount
} = require("../../../target/release/build/pkg/mercat_wasm");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {
  const api = await reqImports.createApi();

  const ticker = await reqImports.generateRandomTicker(api);
  const ticker2 = await reqImports.generateRandomTicker(api);
  const tickerHex = reqImports.stringToHex(ticker);
  const ticker2Hex = reqImports.stringToHex(ticker2); 
  const tickerHexSubStr = tickerHex.substr(2);
  const ticker2HexSubStr = ticker2Hex.substr(2);
  const testEntities = await reqImports.initMain(api);
  const CHAIN_DIR = 'chain_dir';

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

  // Alice creates Confidential Assets 
  await createConfidentialAsset(api, tickerHex, alice);
  await createConfidentialAsset(api, ticker2Hex, alice);

  // Alice and Bob create their Mercat account locally and submit the proof to the chain
  const aliceMercatInfo = create_account([tickerHexSubStr, ticker2HexSubStr], tickerHexSubStr);
  const bobMercatInfo = create_account([tickerHexSubStr, ticker2HexSubStr], tickerHexSubStr);

  // Validate Alice and Bob's Mercat Accounts
  await validateMercatAccount(api, alice, aliceMercatInfo.account_tx);
  await validateMercatAccount(api, bob, bobMercatInfo.account_tx);

  // Charlie creates his mediator Mercat Account 
  const charlieMercatInfo = create_mediator_account();

  // Validate Charlie's Mercat Account 
  await addMediatorMercatAccount(api, charlie, charlieMercatInfo.public_account);

  let aliceBalance = await getEncryptedBalance(api, alice_did, aliceMercatInfo.account_id);
  let bobBalance = await getEncryptedBalance(api, bob_did, bobMercatInfo.account_id);
  
  // Mint Tokens 
  const alicePublicAccount = new PubAccount(aliceMercatInfo.account_id, aliceMercatInfo.public_key);
  const aliceAccount = new Account(aliceMercatInfo.secret_account, alicePublicAccount);

  const mintTxInfo = mint_asset(1000, aliceAccount);
 
  await mintTokens(api, alice, tickerHexSubStr, 1000, mintTxInfo.asset_tx);

  aliceBalance = await getEncryptedBalance(api, alice_did, aliceMercatInfo.account_id);
  bobBalance = await getEncryptedBalance(api, bob_did, bobMercatInfo.account_id);

  // Create Venue
  const venueCounter = await reqImports.createVenue(api, charlie);
 
  // Create Confidential Instruction
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
  
  const bobPublicAccount = new PubAccount(bobMercatInfo.account_id, bobMercatInfo.public_key);
  const bobAccount = new Account(bobMercatInfo.secret_account, bobPublicAccount);
  
  const transactionProof = create_transaction(100, aliceAccount, aliceBalance, bobPublicAccount, charlieMercatInfo.public_key);
  
  await affirmConfidentialInstruction(api, instructionCounter, transactionProof.init_tx, alice, alice_did);
  
  const finalizedProof = finalize_transaction(100, transactionProof.init_tx, bobAccount);
  
  // Removes the Chain_Dir
  await removeChainDir(CHAIN_DIR);
  
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

async function finalizeTransaction(account, tickerHex, amount, proof, dbDir) {
    const { stdout, stderr } = await exec(
        `mercat-interactive finalize-transaction --db-dir ${dbDir} --account-id-from-ticker ${tickerHex} --amount ${amount} --receiver ${account} --init-tx ${proof}`
      );
   
}

async function affirmConfidentialInstruction(api, instruction_id, proof, signer, signer_did) {
    const portfolio = reqImports.getDefaultPortfolio(signer_did);
    const transaction = await api.tx.settlement.affirmConfidentialInstruction(instruction_id, {InitializedTransfer: proof}, [portfolio]);
    let tx = await reqImports.sendTx(signer, transaction);
    if(tx !== -1) reqImports.fail_count--;
}

async function createTransaction(account, dbDir, tickerHex, amount, receiver, mediator, balance) {

    let base64Receiver = encodeToBase64(receiver.publicKey);
    let base64Mediator = encodeToBase64(mediator.publicKey);

    const { stdout, stderr } = await exec(
      `mercat-interactive create-transaction --db-dir ${dbDir} --account-id-from-ticker ${tickerHex} --amount ${amount} --sender ${account} \
      --receiver ${receiver.encryptedAssetId} ${base64Receiver} \
      --mediator ${base64Mediator} \
      --pending-balance ${balance}`
    );
    const splitOutput = stderr.split('\n');
    const transactionProof = splitOutput[22].trim();

    return transactionProof;
}

async function displayBalance(api, did, mercatAccountID, account, tickerHex, dbDir) {
    // Get encrypted balance
    const accountEncryptedBalance = await getEncryptedBalance(api, did, mercatAccountID);
    // Decrypt balance
    const accountBalance = await decryptBalances(account, tickerHex, accountEncryptedBalance, dbDir);
    console.log(`${account}'s Balance: ${accountBalance}`);

    return accountEncryptedBalance;
}

async function mintTokens(api, signer, tickerHex, amount, mintProof) {
    const transaction = await api.tx.confidentialAsset.mintConfidentialAsset(tickerHex, amount, mintProof);
    let tx = await reqImports.sendTx(signer, transaction);
    if(tx !== -1) reqImports.fail_count--;
}

async function getEncryptedBalance(api, did, mercatAccountID){
    return await api.query.confidentialAsset.mercatAccountBalance(did, mercatAccountID);
}

async function decryptBalances(account, tickerHex, encryptedBalance, dbDir) {
    const { stdout, stderr } = await exec(
      `mercat-interactive decrypt --db-dir ${dbDir} --ticker ${tickerHex} --user ${account} --encrypted-value ${encryptedBalance}`
    );
    const splitOutput = stderr.split('\n');
    return splitOutput[11].substr(65).trim();
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

async function createMercatUserAccount(account, tickerHex, ticker2Hex, dbDir) {
  const { stdout, stderr } = await exec(
    `mercat-interactive create-user-account --user ${account} --db-dir ${dbDir} --ticker ${tickerHex} --valid-ticker-names ${tickerHex} ${ticker2Hex}`
  );

  const splitOutput = stderr.split('\n');
  const mercatAccountID = splitOutput[21].trim();
  const mercatAccountProof = splitOutput[24].trim();

  return {mercatAccountID, mercatAccountProof};
}

async function createMercatMediatorAccount(account, dbDir) {
    const { stdout, stderr } = await exec(
      `mercat-interactive create-mediator-account  --db-dir ${dbDir} --user ${account}`
    );
    const splitOutput = stderr.split('\n');
    
    return splitOutput[15].trim();
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
