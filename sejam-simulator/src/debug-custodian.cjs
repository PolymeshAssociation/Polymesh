const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ALICE_DID = '0x0100000000000000000000000000000000000000000000000000000000000000';
const ASSET = '0x1fb383fc367588cb8de25ca5474caa22';

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });

  console.log('\n=== portfolio storage items ===');
  for (const name of Object.keys(api.query.portfolio || {})) {
    console.log('  portfolio.' + name);
  }

  console.log('\n=== alice default portfolio custodian ===');
  try {
    const cust = await api.query.portfolio.custodians({ did: ALICE_DID, kind: 'Default' });
    console.log('custodian:', cust.toHuman ? JSON.stringify(cust.toHuman()) : String(cust));
  } catch (e) {
    console.log('custodians query error:', e.message);
  }

  console.log('\n=== alice default portfolio balance (AssetHolder level) ===');
  try {
    const bal = await api.query.asset.balanceOf(ASSET, { Portfolio: { did: ALICE_DID, kind: 'Default' } });
    console.log('balance:', bal.toString());
  } catch (e) {
    console.log('balanceOf error:', e.message);
  }

  console.log('\n=== portfolio.setCustodian signature ===');
  const sc = api.tx.portfolio.setCustodian;
  if (sc) {
    console.log('args count:', sc.meta.args.length);
    sc.meta.args.forEach((arg, i) => {
      console.log('  arg' + i + ': ' + arg.name + ' : ' + arg.type);
    });
  } else {
    console.log('setCustodian NOT found');
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
