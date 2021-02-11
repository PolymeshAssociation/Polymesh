import {createApi, initMain, generateKeys, generateRandomKey, createIdentities} from "../util/init";

async function main (): Promise<void> {
    const api = await createApi();
    const testEntities = await initMain(api);
    let primary_dev_seed = await generateRandomKey();
    let keys = await generateKeys(api, 2, primary_dev_seed );
    await createIdentities(api, keys, testEntities[0]);
}

main().catch(console.error);