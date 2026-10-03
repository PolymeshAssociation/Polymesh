const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });

  for (const name of ['createAsset', 'createAssetWithCustomType', 'issue', 'redeem']) {
    const m = api.tx.asset[name];
    console.log('\n=== asset.' + name + ' ===');
    if (!m) { console.log('  NOT FOUND'); continue; }
    m.meta.args.forEach((arg, i) => console.log('  arg' + i + ': ' + arg.name + ' : ' + arg.type));
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
