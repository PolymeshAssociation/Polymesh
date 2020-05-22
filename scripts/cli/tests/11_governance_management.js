// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

const testKeyring = require('@polkadot/keyring/testing');

const keyring = testKeyring.default();

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {
  
  const api = await reqImports.createApi();
  
  await proposePIP( api, keyring );
  
  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}


async function proposePIP(api, keyring) {

  // Get the current sudo key in the system
  const sudoKey = await api.query.sudo.key();

  // Lookup from keyring (assuming we have added all, on --dev this would be `//Alice`)
  const sudoPair = keyring.getPair(sudoKey.toString());

  let proposal = await api.tx.pips.setProposalDuration(10);
  let deposit = 10;
  let url = "www.google.com";
  let description = "test proposal";

  // Send the actual sudo transaction
  const unsub = await api.tx.sudo.sudo(
    await api.tx.pips.propose(proposal, deposit, url, description, null)
    );

    const result = await reqImports.sendTransaction(unsub, sudoPair, 0);  
    const passed = result.findRecord('system', 'ExtrinsicSuccess');
    if (passed) {
      reqImports.fail_count--;
    }
  
}

main().catch(console.error);
