// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
const {assert} = require("chai");
const BN = require("bn.js");
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

async function main() {
    // Schema path
    const filePath = reqImports["path"].join(__dirname + "/../../../polymesh_schema.json");
    const customTypes = JSON.parse(reqImports["fs"].readFileSync(filePath, "utf8"));

    // Start node instance
    const ws_provider = new reqImports["WsProvider"](process.env.WS_PROVIDER || "ws://127.0.0.1:9944/");
    const api = await reqImports["ApiPromise"].create({
        types: customTypes,
        provider: ws_provider
    });

    const testEntities = await reqImports["initMain"](api);

    // And an signer key to the keystore to sign the offchain worker data
    await reqImports["generateOffchainKeys"](api, "cddw"); // Off-chain worker Key type

    const controllers = ["nom1", "nom2", "nom3", "nom4", "nom5"];
    // Create stash accounts for the nominators
    const stash_nominators = await reqImports["generateStashKeys"](api, controllers);

    // Create keys of validators
    const validators_key = await reqImports["generateKeys"](api, 2, "validator");

    // Get the key of cdd provider
    let provider = await getValidCddProvider(api, testEntities[0]);

    // Provide the DID to the stash_nominators
    // Calculate the bonding duration
    let expiryTime = await getExpiries(api, stash_nominators.length);
    let nominator_dids = await reqImports["createIdentitiesWithExpiry"](api, stash_nominators, provider, expiryTime);
    await reqImports["blockTillPoolEmpty"](api);

    // fund stash keys
    await reqImports["distributePolyBatch"]( api, stash_nominators, reqImports["transfer_amount"], testEntities[0] );
    await reqImports["blockTillPoolEmpty"](api);

    // get controller key for nominators
    const controller_keys = await generateEntities(api, controllers);
    // Add as secondary key to the existing did who wants to be a potential nominator
    await reqImports["addSecondaryKeys"]( api, stash_nominators, nominator_dids, controller_keys );
    await reqImports["blockTillPoolEmpty"](api);

    await reqImports["authorizeJoinToIdentities"]( api, stash_nominators, nominator_dids, controller_keys);
    await reqImports["blockTillPoolEmpty"](api);

    await addNominator(api, controller_keys, stash_nominators, testEntities[0], validators_key);
    await reqImports["blockTillPoolEmpty"](api);

    subscribeCddOffchainWorker(api);
}


async function getValidCddProvider(api, alice) {
    let transfer_amount = new BN(1000).mul(new BN(10).pow(new BN(6)));
    // Fetch the cdd providers key and provide them right fuel to spent for
    // cdd creation
    let service_providers = await api.query.cddServiceProviders.activeMembers();
    let service_provider_1_key = await reqImports["generateEntity"](api, "service_provider_1");

    // match the identity within the identity pallet
    const service_provider_1_identity = await api.query.identity.keyToIdentityIds(service_provider_1_key.publicKey);
    assert.equal(JSON.parse(service_provider_1_identity).toString(), service_providers[0].toString());

    // fund the service_provider_1 account key to successfully call the `register_did` dispatchable
    let old_balance = (await api.query.system.account(service_provider_1_key.address)).data.free;

    await reqImports["distributePoly"](api, service_provider_1_key, transfer_amount,  alice);
    await reqImports["blockTillPoolEmpty"](api);

    // check the funds of service_provider_1
    let new_free_balance = (await api.query.system.account(service_provider_1_key.address)).data.free;
    assert.equal(new_free_balance.toString(), (transfer_amount.add(old_balance)).toString());
    return service_provider_1_key;
}

async function generateEntities(api, accounts) {
    let entites = []
    for(let i= 0; i < accounts.length; i++) {
        let entity = await reqImports["generateEntity"](api, accounts[i]);
        entites.push(entity);
    }
    return entites;
}

async function getExpiries(api, length) {
    let blockTime = await api.consts.babe.expectedBlockTime;
    let bondingDuration = await api.consts.staking.bondingDuration;
    let sessionPerEra = await api.consts.staking.sessionsPerEra;
    let session_length = await api.consts.babe.epochDuration;
    const currentBlockTime = await api.query.timestamp.now();

    const bondingTime = bondingDuration * sessionPerEra * session_length;
    let expiryTime = parseInt(currentBlockTime) + parseInt(bondingTime * 1000);

    let expiries = [];
    for (let i = 1; i <= length; i++) {
        // Providing 15 block as the extra time
        let temp = expiryTime + i * 5 * parseInt(blockTime);
        expiries.push(temp);
    }
    return expiries;
}

async function addNominator(api, controller, stash, from, validator) {
    let transfer_amount = new BN(1).mul(new BN(10).pow(new BN(6)));
    let operators = [validator[0].address, validator[1].address];
    let bond_amount = new BN(3).mul(new BN(10).pow(new BN(6)));

    // bond nominator first
    for (let i = 0; i < stash.length; i++) {
        const tx = api.tx.staking.bond(controller[i].address, bond_amount, "Controller");
        await reqImports["signAndSendTransaction"](tx, stash[i]);
    }
    await reqImports["blockTillPoolEmpty"](api);
    // fund controller keys
    await reqImports["distributePolyBatch"](api, controller, transfer_amount, from);
    await reqImports["blockTillPoolEmpty"](api);

    for (let i = 0; i < controller.length; i++) {
        const tx = api.tx.staking.nominate(operators);
        await reqImports["signAndSendTransaction"](tx, controller[i]);
    }
}

async function subscribeCddOffchainWorker(api) {
    let eventCount = 0;
    const unsubscribe = await api.rpc.chain.subscribeNewHeads(async (header) => {
    console.log(`Chain is at block: #${header.number}`);
    let hash = await api.rpc.chain.getBlockHash(header.number);
    let events = await api.query.system.events.at(hash.toString());
    for (let i = 0; i < Object.keys(events).length - 1; i++) {
        try {
            if (events[i].event.data["_section"]== "CddOffchainWorker") {
                let typeList = events[i].event.data["_typeDef"];
                console.log(`EventName - ${events[i].event.data["_method"]} at block number ${header.number}`);
                for (let j = 0; j < typeList.length; j++) {
                    let value = events[i].event.data[j];
                    if (typeList[j].type == "Bytes")
                        value = Utils.hexToString(Utils.bytesToHex(events[i].event.data[j]));
                    console.log(`${typeList[j].type} : ${value}`);
                    eventCount++;
                }
                console.log("***************************************");
            }
        } catch(error) {
            console.log(`Event is not present in this block ${header.number}`);
        }
    }
    if (eventCount >= 5) {
        process.exit(0);
    }
});
}

main().catch(console.error);
