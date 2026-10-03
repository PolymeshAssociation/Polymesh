const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ASSET = '0x1fb383fc367588cb8de25ca5474caa22';

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });

  console.log('=== ALL event types in relevant pallets ===');
  for (const section of ['asset', 'settlement', 'balances', 'identity', 'complianceManager']) {
    const events = api.events[section];
    if (!events) { console.log('  ' + section + ': NOT FOUND'); continue; }
    console.log('\n  [' + section + ']:');
    for (const method of Object.keys(events)) {
      console.log('    - ' + section + '.' + method);
    }
  }

  // انجام یک تراکنش تستی برای ایجاد event واقعی
  console.log('\n=== Performing test transfer to generate events ===');
  const keyring = new Keyring({ type: 'sr25519' });
  const reza = keyring.addFromUri('//TehranInv//1');
  const sara = keyring.addFromUri('//TehranInv//2');

  await new Promise((resolve, reject) => {
    api.tx.asset.transferAsset(ASSET, sara.address, 1000000n, null).signAndSend(reza, ({ status, dispatchError, events }) => {
      if (dispatchError) { reject(new Error(dispatchError.toString())); return; }
      if (status.isInBlock || status.isFinalized) {
        console.log('\n=== Events fired by this transfer ===');
        events.forEach(({ event }) => {
          const key = event.section + '.' + event.method;
          const data = event.data.toHuman ? event.data.toHuman() : event.data;
          console.log('  ' + key + ':');
          console.log('    ' + JSON.stringify(data, null, 2).split('\n').join('\n    '));
        });
        resolve();
      }
    });
  });

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
