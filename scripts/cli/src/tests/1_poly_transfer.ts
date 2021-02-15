import {
    createApi,
    initMain,
    generateRandomKey,
    generateKeys,
    transferAmount
  } from "../util/init";
  import { createIdentities } from "../helpers/identity_helper";
  import { distributePolyBatch } from "../helpers/poly_helper";
  
  async function main(): Promise<void> {
      try {
          const api = await createApi();
          const testEntities = await initMain(api.api);
          const primary_dev_seed = await generateRandomKey();
          const keys = await generateKeys(api.api, 2, primary_dev_seed);
          await createIdentities(api.api, keys, testEntities[0]);
          await distributePolyBatch(api.api, keys, transferAmount.toNumber(), testEntities[0]);
          await api.ws_provider.disconnect();     
      }
      catch(err) {
          console.log(err);
      }
  }
  
  main();
  