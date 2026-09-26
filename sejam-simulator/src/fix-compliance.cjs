const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const CHAIN_URL = process.env.SETUP_CHAIN_URL || 'ws://127.0.0.1:9945';
const ASSET_ID = '0x1fb383fc367588cb8de25ca5474caa22'; // AssetId واقعی که پیدا کردیم
const DEFAULT_CDD_ID = '0x' + '00'.repeat(32);

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider(CHAIN_URL) });
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  console.log('Using AssetId: ' + ASSET_ID);

  // استخراج DID آلیس
  const rec = await api.query.identity.keyRecords(alice.address);
  let aliceDid = null;
  if (rec.isSome) {
    const v = rec.unwrap();
    if (typeof v.type === 'string') {
      if (v.type === 'PrimaryKey' || v.type === 'SecondaryKey') {
        aliceDid = v.value ? v.value.toString() : null;
      }
    } else if (v.primaryKey) {
      aliceDid = v.primaryKey.toString();
    } else if (v.secondaryKey) {
      aliceDid = v.secondaryKey.toString();
    }
  }
  console.log('Alice DID: ' + aliceDid);
  
  if (!aliceDid) { 
    console.error('Alice DID not resolved!');
    process.exit(1); 
  }

  const condition = {
    conditionType: { IsPresent: { CustomerDueDiligence: DEFAULT_CDD_ID } },
    issuers: [{ issuer: aliceDid, trustedFor: 'Any' }],
  };

  console.log('\nSetting Compliance Rule...');
  await new Promise((resolve, reject) => {
    api.tx.complianceManager.addComplianceRequirement(ASSET_ID, [], [condition])
      .signAndSend(alice, ({ status, dispatchError }) => {
        if (dispatchError) {
          let msg = dispatchError.toString();
          if (dispatchError.isModule) {
            try { 
              const d = api.registry.findMetaError(dispatchError.asModule); 
              msg = d.section + '.' + d.name; 
            } catch {}
          }
          reject(new Error(msg));
        } else if (status.isInBlock || status.isFinalized) {
          console.log('✅ Compliance rule set successfully!');
          resolve();
        }
      });
  });

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
