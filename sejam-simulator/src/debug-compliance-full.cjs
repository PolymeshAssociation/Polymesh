const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });

  console.log('=== ALL complianceManager methods ===');
  const methods = Object.keys(api.tx.complianceManager || {}).sort();
  methods.forEach(m => console.log('  ' + m));

  console.log('\n=== ALL complianceManager storage items ===');
  const storage = Object.keys(api.query.complianceManager || {}).sort();
  storage.forEach(s => console.log('  ' + s));

  console.log('\n=== Signature of addDefaultTrustedClaimIssuer ===');
  const m = api.tx.complianceManager.addDefaultTrustedClaimIssuer;
  if (m) {
    console.log('args count:', m.meta.args.length);
    m.meta.args.forEach((arg, i) => {
      console.log('  arg' + i + ': ' + arg.name + ' : ' + arg.type);
    });
  }

  console.log('\n=== Testing trustedClaimIssuer storage with different keys ===');
  const ASSET = '0x1fb383fc367588cb8de25ca5474caa22';
  const claimTypes = ['CustomerDueDiligence', 'InvestorUniqueness', 'BuyLockup', 'SellLockup'];
  
  for (const ct of claimTypes) {
    try {
      const val = await api.query.complianceManager.trustedClaimIssuer(ASSET, ct);
      console.log('  ' + ct + ':', val.toHuman ? JSON.stringify(val.toHuman()) : String(val));
    } catch (e) {
      console.log('  ' + ct + ' error:', e.message.slice(0, 80));
    }
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
