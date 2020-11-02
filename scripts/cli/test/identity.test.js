require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

describe('Halva test', () => {
 
  describe('Creating Identities', () => {

    it('Create Identity for Key(eventNotEmitted)', async () => {
      const api = await reqImports.createApi();
      const testEntities = await reqImports.initMain(api);
      const primary_dev_seed = await reqImports.generateRandomKey(api);
      const keys = await reqImports.generateKeys(api, 2, primary_dev_seed );

      const tx = api.tx.identity.cddRegisterDid(keys[0].address, []);
      await eventNotEmitted(tx,  'ExtrinsicFailed', 'system', '', testEntities[0]);
    });

    it('Create Identity for Key(eventEmitted)', async () => {
      const api = await reqImports.createApi();
      const testEntities = await reqImports.initMain(api);
      const primary_dev_seed = await reqImports.generateRandomKey(api);
      const keys = await reqImports.generateKeys(api, 2, primary_dev_seed );

      const tx = api.tx.identity.cddRegisterDid(keys[0].address, []);
      await eventEmitted(tx,  'ExtrinsicSuccess', 'system', '', testEntities[0]);
    });

    it('Create Identity for Key(passes)', async () => {
      const api = await reqImports.createApi();
      const testEntities = await reqImports.initMain(api);
      const primary_dev_seed = await reqImports.generateRandomKey(api);
      const keys = await reqImports.generateKeys(api, 2, primary_dev_seed );

      const tx = api.tx.identity.cddRegisterDid(keys[0].address, []);
      await passes(tx, '', testEntities[0]);
     
    });

  });
});
