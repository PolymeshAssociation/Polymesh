// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

const util = require('util');
const exec = util.promisify(require('child_process').exec);
// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {
  const api = await reqImports.createApi();

  const ticker = await reqImports.generateRandomTicker(api);
  const ticker2 = await reqImports.generateRandomTicker(api);
  const tickerHex = reqImports.stringToHex(ticker); 
  const ticker2Hex = reqImports.stringToHex(ticker2); 
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
  const bobMercatInfo = await createMercatUserAccount('bob', tickerHex, ticker2Hex, CHAIN_DIR);
  const aliceMercatInfo = await createMercatUserAccount('alice', tickerHex, ticker2Hex, CHAIN_DIR);

  // Validate Alice and Bob's Mercat Accounts
  await validateMercatAccount(api, alice, aliceMercatInfo.mercatAccountProof);
  await validateMercatAccount(api, bob, bobMercatInfo.mercatAccountProof);

  // Charlie creates his mediator Mercat Account 
  const charlieMercatInfo = await createMercatMediatorAccount('charlie', CHAIN_DIR);

  // Validate Charlie's Mercat Account 
  await addMediatorMercatAccount(api, charlie, charlieMercatInfo);

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
  const { stdout, stderr } = await exec(`mercat-interactive create-user-account --user ${account} --db-dir ${dbDir} --ticker ${tickerHex.substr(2)} --valid-ticker-names ${tickerHex.substr(2)} ${ticker2Hex.substr(2)}`);

  const splitOutput = stderr.split('\n');
  const mercatAccountID = splitOutput[21].trim();
  const mercatAccountProof = splitOutput[24].trim();

  return {mercatAccountID, mercatAccountProof};
}

async function createMercatMediatorAccount(account, dbDir) {
    const { stdout, stderr } = await exec(`mercat-interactive create-mediator-account  --db-dir ${dbDir} --user ${account}`);
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
