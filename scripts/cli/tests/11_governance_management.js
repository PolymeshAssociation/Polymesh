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
  let govCommittee1 = testEntities[5];
  let govCommittee2 = testEntities[6];

  let proposalId = 0;
  
  await reqImports.createIdentities( api, [bob, govCommittee1, govCommittee2], alice );

  await bondPoly(api, alice, bob);

  await proposePIP( api, bob );

  await amendProposal(api, bob);

  await fastTrackProposal(api, proposalId, alice);
 
  await reqImports.distributePolyBatch( api, [govCommittee1, govCommittee2], reqImports.transfer_amount, alice );

  await voteEnactReferendum(api, proposalId, govCommittee1);
  
  await voteEnactReferendum(api, proposalId, govCommittee2);

  await overrideReferendumEnactmentPeriod(api, proposalId, null, alice);

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function voteEnactReferendum(api, proposalId, signer) {

  let nonceObj = {nonce: reqImports.nonces.get(signer.address)};
  const transaction = await api.tx.polymeshCommittee.voteEnactReferendum(proposalId);
  const result = await reqImports.sendTransaction(transaction, signer, nonceObj);  
  const passed = result.findRecord('system', 'ExtrinsicSuccess');
  if (passed) reqImports.fail_count--;

  reqImports.nonces.set( signer.address, reqImports.nonces.get(signer.address).addn(1));
}

async function overrideReferendumEnactmentPeriod(api, proposalId, until, signer) {

  let nonceObj = {nonce: reqImports.nonces.get(signer.address)};
  const transaction = await api.tx.pips.overrideReferendumEnactmentPeriod(proposalId, until);
  const result = await reqImports.sendTransaction(transaction, signer, nonceObj);  
  const passed = result.findRecord('system', 'ExtrinsicSuccess');
  if (passed) reqImports.fail_count--;

  reqImports.nonces.set( signer.address, reqImports.nonces.get(signer.address).addn(1));
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
