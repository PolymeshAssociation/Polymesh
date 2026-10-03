const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ALICE_DID = '0x0100000000000000000000000000000000000000000000000000000000000000';
const ASSET = '0x1fb383fc367588cb8de25ca5474caa22';

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });

  console.log('\n=== api.tx.portfolio (all methods) ===');
  for (const name of Object.keys(api.tx.portfolio || {})) {
    console.log('  portfolio.' + name);
  }

  console.log('\n=== portfolioCustodian query for Alice Default ===');
  try {
    const cust = await api.query.portfolio.portfolioCustodian({ did: ALICE_DID, kind: 'Default' });
    console.log('custodian:', cust.toHuman ? JSON.stringify(cust.toHuman()) : String(cust));
  } catch (e) {
    console.log('error:', e.message);
  }

  console.log('\n=== Testing AssetHolder with Identity ===');
  try {
    const bal = await api.query.asset.balanceOf(ASSET, { Identity: ALICE_DID });
    console.log('balance with Identity:', bal.toString());
  } catch (e) {
    console.log('Identity error:', e.message);
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
