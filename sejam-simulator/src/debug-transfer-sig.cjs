const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });

  console.log('\n=== settlement.transferFunds signature ===');
  const tf = api.tx.settlement.transferFunds;
  if (tf) {
    console.log('args count:', tf.meta.args.length);
    tf.meta.args.forEach((arg, i) => {
      console.log('  arg' + i + ': ' + arg.name + ' : ' + arg.type);
    });
  }

  console.log('\n=== asset.transferAsset signature ===');
  const ta = api.tx.asset.transferAsset;
  if (ta) {
    console.log('args count:', ta.meta.args.length);
    ta.meta.args.forEach((arg, i) => {
      console.log('  arg' + i + ': ' + arg.name + ' : ' + arg.type);
    });
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
