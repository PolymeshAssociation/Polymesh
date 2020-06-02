// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Import the test keyring (already has dev keys for Alice, Bob, Charlie, Eve & Ferdie)
const testKeyring = require('@polkadot/keyring/testing');

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {
  
  const api = await reqImports.createApi();
  
  const testEntities = await reqImports.initMain(api);
  
  let alice = testEntities[0];
  let bob = testEntities[1];
  
  await reqImports.createIdentities( api, [bob], alice );

  await bondPoly(api, alice, bob);

  await proposePIP( api, bob );

  await amendProposal(api, bob);

  await fastTrackProposal(api, 0, alice);

  await enactReferendum(api, 0);

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function enactReferendum(api, proposalId) {

  // Get the current sudo key in the system
  const sudoKey = await api.query.sudo.key();

  const keyring = testKeyring.default();

  // Lookup from keyring (assuming we have added all, on --dev this would be `//Alice`)
  const sudoPair = keyring.getPair(sudoKey.toString());
  
  // Send the actual sudo transaction
   const transaction = await api.tx.sudo
   .sudo(
     await api.tx.pips.enactReferendum(proposalId)
   );

   const result = await reqImports.sendTransaction(transaction, sudoPair, 0);  
   const passed = result.findRecord('system', 'ExtrinsicSuccess');
   if (passed) reqImports.fail_count--;

}

async function fastTrackProposal(api, proposalId, signer) {

  let nonceObj = {nonce: reqImports.nonces.get(signer.address)};
  const transaction = await api.tx.pips.fastTrackProposal(proposalId);
  const result = await reqImports.sendTransaction(transaction, signer, nonceObj);  
  const passed = result.findRecord('system', 'ExtrinsicSuccess');
  if (passed) reqImports.fail_count--;

  reqImports.nonces.set( signer.address, reqImports.nonces.get(signer.address).addn(1));
}

async function amendProposal(api, signer) {

  let nonceObj = {nonce: reqImports.nonces.get(signer.address)};
  const transaction = await api.tx.pips.amendProposal(0, "www.facebook.com", null);
  const result = await reqImports.sendTransaction(transaction, signer, nonceObj);  
  const passed = result.findRecord('system', 'ExtrinsicSuccess');
  if (passed) reqImports.fail_count--;

  reqImports.nonces.set( signer.address, reqImports.nonces.get(signer.address).addn(1));
}


async function bondPoly(api, signer, bob) {

  let nonceObj = {nonce: reqImports.nonces.get(signer.address)};
  const transaction = await api.tx.staking.bond(bob.publicKey, 20_000, "Staked");
  const result = await reqImports.sendTransaction(transaction, signer, nonceObj);  
  const passed = result.findRecord('system', 'ExtrinsicSuccess');
  if (passed) reqImports.fail_count--;

  reqImports.nonces.set( signer.address, reqImports.nonces.get(signer.address).addn(1));
}


async function proposePIP(api, signer) {

  let proposal = await api.tx.pips.setProposalDuration(10);
  let deposit = 10_000_000_000;
  let url = "www.google.com";
  let description = "test proposal";

  let nonceObj = {nonce: reqImports.nonces.get(signer.address)};
  const transaction = await api.tx.pips.propose(proposal, deposit, url, description, null);
  const result = await reqImports.sendTransaction(transaction, signer, nonceObj);  
  const passed = result.findRecord('system', 'ExtrinsicSuccess');
  if (passed) reqImports.fail_count--;

  reqImports.nonces.set( signer.address, reqImports.nonces.get(signer.address).addn(1));
  
}

main().catch(console.error);
