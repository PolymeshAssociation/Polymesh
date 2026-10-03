const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ALICE_DID = '0x0100000000000000000000000000000000000000000000000000000000000000';

function hexToUtf8(hex) {
  const bytes = new Uint8Array((hex.slice(2).match(/.{1,2}/g) || []).map(b => parseInt(b, 16)));
  return new TextDecoder('utf-8').decode(bytes);
}

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });

  const nameEntries = await api.query.asset.assetNames.entries();
  for (const [key, val] of nameEntries) {
    const assetId = key.args[0].toString();
    const name = hexToUtf8(val.toString());
    console.log('\n========== ASSET: ' + name + ' (' + assetId + ') ==========');

    console.log('--- balanceOf (DID-level) entries ---');
    try {
      const b = await api.query.asset.balanceOf.entries(assetId);
      if (b.length === 0) console.log('  (empty)');
      for (const [k, v] of b) {
        console.log('  holder:', JSON.stringify(k.args[1].toHuman ? k.args[1].toHuman() : String(k.args[1])), '=>', v.toString());
      }
    } catch (e) { console.log('  err:', e.message.slice(0, 60)); }

    console.log('--- portfolioAssetBalances (portfolio-level) entries ---');
    try {
      const p = await api.query.portfolio.portfolioAssetBalances.entries();
      const mine = p.filter(([k]) => k.args[1].toString() === assetId);
      if (mine.length === 0) console.log('  (empty)');
      for (const [k, v] of mine) {
        console.log('  portfolio:', JSON.stringify(k.args[0].toHuman ? k.args[0].toHuman() : String(k.args[0])), '=>', v.toString());
      }
    } catch (e) { console.log('  err:', e.message.slice(0, 60)); }

    console.log('--- Alice DID balanceOf ---');
    try {
      const ab = await api.query.asset.balanceOf(assetId, ALICE_DID);
      console.log('  ', ab.toString());
    } catch (e) { console.log('  err:', e.message.slice(0, 60)); }
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
