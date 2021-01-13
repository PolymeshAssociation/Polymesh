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
  
  await reqImports.distributePolyBatch( api, [primary_keys[0]], reqImports.transfer_amount, alice );
  
  await reqImports.issueTokenPerDid(api, [primary_keys[0]], ticker);
  
  await addSecondaryKeys( api, primary_keys, secondary_keys );
  
  await reqImports.authorizeJoinToIdentities( api, primary_keys, issuer_dids, secondary_keys);
  
  await reqImports.distributePolyBatch( api, [secondary_keys[0]], reqImports.transfer_amount, alice );

  let portfolioOutput = await createPortfolio(api, portfolioName, secondary_keys[0]);

  assert.equal(portfolioOutput, false);

  await setPermissionToSigner(api, primary_keys, secondary_keys, "Portfolio", "create_portfolio");

  portfolioOutput = await createPortfolio(api, portfolioName, secondary_keys[0]);

  assert.equal(portfolioOutput, true);

  await setPermissionToSigner(api, primary_keys, secondary_keys, "Portfolio", "move_portfolio_funds");

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function setPermissionToSigner(api, accounts, secondary_accounts, pallet_name, dispatchable_name) {

  const permissions = {
    "asset": null,
    "extrinsic": [
      {
        "pallet_name": pallet_name,
        "total": true,
        "dispatchable_names": [ dispatchable_name ]
      }
    ],
    "portfolio": null
  };

  for (let i = 0; i < accounts.length; i++) {
    let signer = { Account: secondary_accounts[i].publicKey };
    let transaction = api.tx.identity.setPermissionToSigner(signer, permissions);
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

main().catch(console.error);
