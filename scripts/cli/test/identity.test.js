require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

describe('Halva test', () => {
 
  describe('Creating Identities', () => {

    it('Create Identity for Key(eventNotEmitted)', async () => {
      let {api, testEntities, _primary_dev_seed, keys} = await getSetup();
      const tx = api.tx.identity.cddRegisterDid(keys[0].address, []);
      await eventNotEmitted(tx,  'ExtrinsicFailed', 'system', '', testEntities[0]);
    });

    it('Create Identity for Key(eventEmitted)', async () => {
      let {api, testEntities, _primary_dev_seed, keys} = await getSetup();
      const tx = api.tx.identity.cddRegisterDid(keys[0].address, []);
      await eventEmitted(tx,  'ExtrinsicSuccess', 'system', '', testEntities[0]);
    });

    it('Create Identity for Key(passes)', async () => {
      let {api, testEntities, _primary_dev_seed, keys} = await getSetup();
      const tx = api.tx.identity.cddRegisterDid(keys[0].address, []);
      await passes(tx, '', testEntities[0]);
     
    });

  });
});

async function getSetup() {
  let api = await reqImports.createApi();
  let testEntities = await reqImports.initMain(api);
  let primary_dev_seed = await reqImports.generateRandomKey(api);
  let keys = await reqImports.generateKeys(api, 2, primary_dev_seed );

  return {api, testEntities, primary_dev_seed, keys};
}
