// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
const {assert} = require("chai");
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

async function main() {
    // Schema path
    const filePath = reqImports["path"].join(__dirname + "/../../../polymesh_schema.json");
    const customTypes = JSON.parse(reqImports["fs"].readFileSync(filePath, "utf8"));

    // Start node instance
    const ws_provider = new reqImports["WsProvider"]("ws://127.0.0.1:9944/");
    const api = await reqImports["ApiPromise"].create({
        types: customTypes,
        provider: ws_provider
    });

    const testEntities = await reqImports["initMain"](api);

    const controllers = ["nom1", "nom2", "nom3", "nom4", "nom5"];
    // Create stash accounts for the nominators
    const stash_nominators = await reqImports["generateStashKeys"](api, controllers);

    // Get the key of cdd provider
    let provider = await getValidCddProvider(api, testEntities[0]);

    // fund stash keys
    await reqImports["distributePoly"]( api, stash_nominators, reqImports["transfer_amount"], testEntities[0] );

    await reqImports["blockTillPoolEmpty"](api);

    // Provide the DID to the stash_nominators
    // Calculate the bonding duration
    let expiryTime = await getExpiries(api, stash_nominators.length);
    
    let nominator_dids = await reqImports["createIdentitiesWithExpiry"](api, stash_nominators, provider, expiryTime);

    await reqImports["blockTillPoolEmpty"](api);

    // get controller key for nominators
    const controller_keys = await generateEntities(api, controllers);

    // Add as signing key to the existing did who wants to be a potential nominator

    await reqImports["addSigningKeys"]( api, stash_nominators, nominator_dids, controller_keys );

    await reqImports["blockTillPoolEmpty"](api);

    await reqImports["authorizeJoinToIdentities"]( api, stash_nominators, nominator_dids, controller_keys);

    await reqImports["blockTillPoolEmpty"](api);

    //await addNominator(api, controller_keys, stash_nominators);

    process.exit();
    }


    async function getValidCddProvider(api, alice) {
        let transfer_amount = 100000 * 10 ** 12;
        // Fetch the cdd providers key and provide them right fuel to spent for
        // cdd creation
        let service_providers = await api.query.cddServiceProviders.activeMembers();
        let service_provider_1_key = await reqImports["generateEntity"](api, "service_provider_1");

        // match the identity within the identity pallet
        const service_provider_1_identity = await api.query.identity.keyToIdentityIds(service_provider_1_key.publicKey);
        assert.equal((JSON.parse(service_provider_1_identity).Unique).toString(), service_providers[0].toString());

        // fund the service_provider_1 account key to successfully call the `register_did` dispatchable
        await reqImports["distributePoly"](api, service_provider_1_key, transfer_amount,  alice);
        await reqImports["blockTillPoolEmpty"](api);
        // check the funds of service_provider_1
        assert.equal((await api.query.balances.account(service_provider_1_key.address)).free.toString(), transfer_amount);
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
        for (let i = 0; i < length; i++) {
            let temp = expiryTime + i * 5 * parseInt(blockTime) * 1000;
            expiries.push(temp);
        }
        return expiries;
    }

    async function addNominator(api, controller, stash) {
        // bond nominator first
        for (let i = 0; i< controller.length; i++) {
            
        }
    }
    
main().catch(console.error);
