// This script prepares the bridge multisig account on the `--dev` chain.

require = require("esm")(module);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");
let relayerUri = "bottom drive obey lake curtain smoke basket hold race lonely fit walk//relay_1";

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {
    console.log(" ***** Creating the API");
    let api = await reqImports.createApi();
    console.log(" ***** Setting Alice up");
    let result = await initAlice(api);
    if (result == []) {
        console.error("initAlice failed");
        process.exit();
    }
    let {
        alice: alice_account,
        did: alice_did
    } = result[0];
    console.log(" ***** Accepting the authorization for the relayer to join the multisig");
    let resultRelay = await initRelay(api);
    if (resultRelay == []) {
        console.error("initRelay failed");
        process.exit();
    }
    console.log(" ***** Success");
    process.exitCode = 0;
    process.exit();
}

// Returns a singleton list with Alice's account and DID if successful and the empty list otherwise.
async function initAlice(api) {
    // Find the account and its DID.
    let alice = await reqImports.generateEntity(api, "Alice");
    let didObj = await api.query.identity.keyToIdentityIds(alice.publicKey);
    let did = didObj.raw.asUnique;
    // Top up Alice's identity balance.
    let didBalance = 1000 * 10**6;
    let nonceObj = {nonce: reqImports.nonces.get(alice.address)};
    let tx = api.tx.balances.topUpIdentityBalance(did, didBalance);
    let result = await reqImports.sendTransaction(tx, alice, nonceObj);
    if (!result.findRecord('system', 'ExtrinsicSuccess')) {
        return []
    }
    reqImports.nonces.set(alice.address, reqImports.nonces.get(alice.address).addn(1));
    return [{
        account: alice,
        did: did
    }];
}

async function initRelay(api) {
    let relayer = await reqImports.generateEntityFromUri(api, relayerUri);
    // accept authorization 9
    let nonceObj = {nonce: reqImports.nonces.get(relayer.address)};
    let tx = api.tx.multiSig.acceptMultisigSignerAsKey(9);
    let result = await reqImports.sendTransaction(tx, relayer, nonceObj);
    if (!result.findRecord('system', 'ExtrinsicSuccess')) {
        return []
    }
    return [{
        account: relayer
    }];
}

main().catch(console.error);
