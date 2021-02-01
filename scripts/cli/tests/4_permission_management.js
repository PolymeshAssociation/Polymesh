// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");
const {assert} = require("chai");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {

  const api = await reqImports.createApi();

  const ticker = await reqImports.generateRandomTicker(api);

  const portfolioName = await reqImports.generateRandomTicker(api);

  let primary_dev_seed = await reqImports.generateRandomKey(api);
  
  let secondary_dev_seed = await reqImports.generateRandomKey(api);

  const testEntities = await reqImports.initMain(api);

  const alice = testEntities[0];

  let primary_keys = await reqImports.generateKeys(api, 1, primary_dev_seed );

  let secondary_keys = await reqImports.generateKeys(api, 1, secondary_dev_seed );
  
  let issuer_dids = await reqImports.createIdentities(api, primary_keys, alice);

  let extrinsics = [];
  let portfolios = [];
  let assets = [];
  let documents = [];
  
  await reqImports.distributePolyBatch( api, [primary_keys[0]], reqImports.transfer_amount, alice );
  
  await reqImports.issueTokenPerDid(api, [primary_keys[0]], ticker, 1000000, null);
  
  await addSecondaryKeys( api, primary_keys, secondary_keys );
  
  await reqImports.authorizeJoinToIdentities( api, primary_keys, issuer_dids, secondary_keys);
  
  await reqImports.distributePolyBatch( api, [secondary_keys[0]], reqImports.transfer_amount, alice );

  let portfolioOutput = await createPortfolio(api, portfolioName, secondary_keys[0]);

  assert.equal(portfolioOutput, false);

  setExtrinsic(extrinsics, "Portfolio", "create_portfolio");

  await setPermissionToSigner(api, primary_keys, secondary_keys, extrinsics, portfolios, assets);

  portfolioOutput = await createPortfolio(api, portfolioName, secondary_keys[0]);

  assert.equal(portfolioOutput, true);

  setExtrinsic(extrinsics, "Portfolio", "move_portfolio_funds");

  await setPermissionToSigner(api, primary_keys, secondary_keys, extrinsics, portfolios, assets);

  let portfolioFundsOutput = await movePortfolioFunds(api, primary_keys[0], secondary_keys[0], ticker, 100);

  assert.equal(portfolioFundsOutput, false);

  await setPortfolio(api, portfolios, primary_keys[0], null);

  await setPortfolio(api, portfolios, secondary_keys[0], "user");

  await setPermissionToSigner(api, primary_keys, secondary_keys, extrinsics, portfolios, assets);

  portfolioFundsOutput = await movePortfolioFunds(api, primary_keys[0], secondary_keys[0], ticker, 100);

  assert.equal(portfolioFundsOutput, true);

  setExtrinsic(extrinsics, "Asset", "add_documents");
  
  await setPermissionToSigner(api, primary_keys, secondary_keys, extrinsics, portfolios, assets);

  setDoc(documents, "www.google.com", 0, "google", null, null);

  let addDocsOutput = await addDocuments(api, ticker, documents, secondary_keys[0]);

  assert.equal(addDocsOutput, false);

  setAsset(ticker, assets);

  await setPermissionToSigner(api, primary_keys, secondary_keys, extrinsics, portfolios, assets);

  addDocsOutput = await addDocuments(api, ticker, documents, secondary_keys[0]);

  assert.equal(addDocsOutput, true);

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

function setDoc(docArray, uri, contentHash, name, docType, filingDate) {
  docArray.push({
    "uri": uri,
    "content_hash": contentHash,
    "name": name,
    "doc_type": docType,
    "filing_date": filingDate
  });
}

async function addDocuments(api, ticker, docs, signer) {
  try {
    const transaction = api.tx.asset.addDocuments(docs, ticker);
    await reqImports.sendTx(signer, transaction);
    return true;
  } catch(err) {
    return false;
  }
}

async function movePortfolioFunds(api, primary_key, secondary_key, ticker, amount) {
  try {
    
    
    const primaryKeyDid = await reqImports.getDid(api, primary_key);
    
    const secondaryKeyDid = await reqImports.getDid(api, secondary_key);
    
    const portfolioNum = (await nextPortfolioNumber(api, secondaryKeyDid)) - 1;

    const from = {
      did: primaryKeyDid,
      kind: 'Default'
    }
    
    const to = {
      did: secondaryKeyDid,
      kind: {User: portfolioNum}
    }
    
    const items = [
      {
        ticker,
        amount
      }
    ]
    
    const transaction = api.tx.portfolio.movePortfolioFunds(from, to, items);
    
    await reqImports.sendTx(secondary_key, transaction);
    
    return true;
    } catch (err) {
      return false;
    }
}

function setAsset(ticker, assetArray) {
  assetArray.push(ticker);
}

async function setPortfolio(api, portfolioArray, key, type) {
  let keyDid = await reqImports.getDid(api, key);

  switch(type) {

    case 'user':
      
      const portfolioNum = (await nextPortfolioNumber(api, keyDid)) - 1;

      let userPortfolio = {
        did: keyDid,
        kind: {User: portfolioNum}
      };

      portfolioArray.push(userPortfolio);
    break;

    default:

      let defaultPortfolio = {
        did: keyDid,
        kind: 'Default'
      };

      portfolioArray.push(defaultPortfolio);
    break;

  }
}

function setExtrinsic(extrinsicArray, palletName, dispatchName) {
  extrinsicArray.push({
    "pallet_name": palletName,
    "total": false,
    "dispatchable_names": [ dispatchName ]
  });
}

async function setPermissionToSigner(api, accounts, secondary_accounts, extrinsic, portfolio, asset) {

  const permissions = {
    "asset": asset,
    "extrinsic": extrinsic,
    "portfolio": portfolio
  };

  for (let i = 0; i < accounts.length; i++) {
    let signer = { Account: secondary_accounts[i].publicKey };
    let transaction = api.tx.identity.legacySetPermissionToSigner(signer, permissions);
    let tx = await reqImports.sendTx(accounts[i], transaction);
    if(tx !== -1) reqImports.fail_count--;
  }
}

// Attach a secondary key to each DID
async function addSecondaryKeys(api, accounts, secondary_accounts) {

  const emptyPermissions =
  {
    "asset": [],
    "extrinsic": [],
    "portfolio": []
  };

  for (let i = 0; i < accounts.length; i++) {
    // 1. Add Secondary Item to identity.

    const transaction = api.tx.identity.addAuthorization({ Account: secondary_accounts[i].publicKey }, { JoinIdentity: emptyPermissions }, null);
    let tx = await reqImports.sendTx(accounts[i], transaction);
    if(tx !== -1) reqImports.fail_count--;
  }
}

async function createPortfolio(api, name, signer) {

  try {
  const transaction = api.tx.portfolio.createPortfolio(name);
  await reqImports.sendTx(signer, transaction);
  return true;
  } catch (err) {
    return false;
  }

}

async function nextPortfolioNumber(api, did) {
  return await api.query.portfolio.nextPortfolioNumber(did);
}

main().catch(console.error);
