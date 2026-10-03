const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const CHAIN_URL = process.env.SETUP_CHAIN_URL || 'ws://127.0.0.1:9945';

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider(CHAIN_URL) });
  const targets = [
    ['asset', 'createAssetWithCustomType'],
    ['asset', 'createAsset'],
    ['asset', 'registerCustomAssetType'],
    ['settlement', 'createVenue'],
    ['complianceManager', 'addComplianceRequirement'],
  ];
  for (const [p, m] of targets) {
    const tx = api.tx[p] && api.tx[p][m];
    if (tx) {
      const args = tx.meta.args.map(a => a.name.toString() + ': ' + a.type.toString());
      console.log('\n' + p + '.' + m + ' (' + args.length + ' args):');
      args.forEach((a, i) => console.log('  ' + i + '. ' + a));
    } else {
      console.log('\n' + p + '.' + m + ': NOT AVAILABLE');
    }
  }
  await api.disconnect();
}

main().catch(e => { console.error(e); process.exit(1); });
