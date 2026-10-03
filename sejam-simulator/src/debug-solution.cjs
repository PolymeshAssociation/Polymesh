const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ALICE_DID = '0x0100000000000000000000000000000000000000000000000000000000000000';
const ASSET = '0x1fb383fc367588cb8de25ca5474caa22';

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });

  console.log('\n=== portfolio.portfolioAssetBalances entries ===');
  try {
    const entries = await api.query.portfolio.portfolioAssetBalances.entries();
    console.log('total entries:', entries.length);
    for (const [key, val] of entries.slice(0, 5)) {
      console.log('key:', JSON.stringify(key.args.map(a => a.toHuman ? a.toHuman() : String(a))));
      console.log('val:', val.toString());
    }
  } catch (e) {
    console.log('error:', e.message);
  }

  console.log('\n=== identity.addAuthorization signature ===');
  const aa = api.tx.identity.addAuthorization;
  if (aa) {
    console.log('args count:', aa.meta.args.length);
    aa.meta.args.forEach((arg, i) => {
      console.log('  arg' + i + ': ' + arg.name + ' : ' + arg.type);
    });
  } else {
    console.log('addAuthorization NOT found');
  }

  console.log('\n=== settlement.addAndAffirmInstruction signature ===');
  const aai = api.tx.settlement.addAndAffirmInstruction;
  if (aai) {
    console.log('args count:', aai.meta.args.length);
    aai.meta.args.forEach((arg, i) => {
      console.log('  arg' + i + ': ' + arg.name + ' : ' + arg.type);
    });
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
