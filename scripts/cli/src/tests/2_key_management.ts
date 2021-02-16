import {
    createApi,
    initMain,
    generateRandomEntity,
    generateRandomKey,
    generateKeys,
    transferAmount,
    signatory
  } from "../util/init";
  import { createIdentities } from "../helpers/identity_helper";
  import { distributePolyBatch } from "../helpers/poly_helper";
  import { addSecondaryKeys, createMultiSig } from "../helpers/key_management_helper";
  
  async function main(): Promise<void> {
      try {
          const api = await createApi();
          const testEntities = await initMain(api.api);
          const alice = testEntities[0];
          const bob = await generateRandomEntity(api.api);
          const charlie = await generateRandomEntity(api.api);
          const dave = await generateRandomEntity(api.api);
          const primary_dev_seed = await generateRandomKey();
          const secondary_dev_seed = await generateRandomKey();
          const primary_keys = await generateKeys(api.api, 2, primary_dev_seed );
          const secondary_keys = await generateKeys(api.api, 2, secondary_dev_seed );
          await createIdentities(api.api, primary_keys, alice);
          await distributePolyBatch( api.api, primary_keys, transferAmount.toNumber(), alice );
          await addSecondaryKeys( api.api, secondary_keys, primary_keys);
          const bob_signatory = await signatory(api.api, bob, alice);
          const charlie_signatory = await signatory(api.api, charlie, alice);
          const dave_signatory = await signatory(api.api, dave, alice);
          const signatory_array = [bob_signatory, charlie_signatory, dave_signatory];
          await createMultiSig( api.api, alice, signatory_array, 2 );
          await api.ws_provider.disconnect();     
      }
      catch(err) {
          console.log(err);
      }
  }
  
  main();
  