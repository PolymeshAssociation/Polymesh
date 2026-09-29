const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ASSET = '0x1fb383fc367588cb8de25ca5474caa22';
const TEST_DID = '0x5939247ff3f43a16fd0085f4f651a89ff3de8c756c3fd2dcd3a7b062e53aed42';
const HALF = 500000000n;

function tx(api, signer, t, label) {
  return new Promise((resolve, reject) => {
    t.signAndSend(signer, ({ status, dispatchError, events }) => {
      if (dispatchError) {
        let m = dispatchError.toString();
        if (dispatchError.isModule) {
          try {
            const d = api.registry.findMetaError(dispatchError.asModule);
            m = d.section + '.' + d.name + ': ' + (d.docs || []).join(' ');
          } catch {}
        }
        reject(new Error(label + ': ' + m));
      } else if (status.isInBlock || status.isFinalized) {
        console.log('OK ' + label);
        resolve(events || []);
      }
    });
  });
}

async function getPrimaryAddress(api, did) {
  try {
    const records = await api.rpc.identity.getDidRecords(did);
    if (records && records.primaryKey) {
      return records.primaryKey.toString();
    }
  } catch {}
  const entries = await api.query.identity.didRecords.entries();
  for (const [key, val] of entries) {
    if (key.args[0].toString().toLowerCase() === did.toLowerCase()) {
      let rec = val;
      if (rec.isSome) rec = rec.unwrap();
      const pk = rec.primaryKey || rec.primary_key;
      if (pk) return pk.toString();
    }
  }
  return null;
}

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  console.log('Finding test user primary address...');
  const testUserAddr = await getPrimaryAddress(api, TEST_DID);
  if (!testUserAddr) throw new Error('Could not find test user address');
  console.log('Test user address:', testUserAddr);

  console.log('\nAttempting transfer with asset.transferAsset...');
  console.log('This method should check Compliance rules.');
  
  await tx(api, alice, 
    api.tx.asset.transferAsset(ASSET, testUserAddr, HALF, null), 
    'transfer 500 to test user via transferAsset'
  );

  console.log('\nVerifying transfer...');
  const balance = await api.query.asset.balanceOf(ASSET, TEST_DID);
  console.log('Test user balance after transfer:', balance.toString());

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
