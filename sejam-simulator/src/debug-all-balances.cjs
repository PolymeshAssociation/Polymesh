const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ALICE_DID = '0x0100000000000000000000000000000000000000000000000000000000000000';
const ASSET = '0x1fb383fc367588cb8de25ca5474caa22';

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });

  console.log('\n=== 1) Where are the issued tokens actually sitting? ===');
  console.log('Scanning ALL entries of asset.balanceOf for our asset...');
  try {
    const entries = await api.query.asset.balanceOf.entries(ASSET);
    console.log('Total balanceOf entries for this asset:', entries.length);
    for (const [key, val] of entries) {
      const holderRaw = key.args[1];
      console.log('  Holder:', JSON.stringify(holderRaw.toHuman ? holderRaw.toHuman() : String(holderRaw)));
      console.log('  Balance:', val.toString());
    }
  } catch (e) {
    console.log('balanceOf entries error:', e.message);
  }

  console.log('\n=== 2) All of Alice\'s portfolios ===');
  try {
    const portfolios = await api.query.portfolio.portfolios.entries(ALICE_DID);
    console.log('Alice has', portfolios.length, 'portfolios');
    for (const [key, val] of portfolios) {
      console.log('  portfolio:', key.args[1].toString(), '→ name:', JSON.stringify(val.toHuman ? val.toHuman() : String(val)));
    }
  } catch (e) {
    console.log('portfolios error:', e.message);
  }

  console.log('\n=== 3) Portfolio balances for ALL holders ===');
  try {
    const entries = await api.query.portfolio.portfolioAssetBalances.entries();
    console.log('Total portfolioAssetBalances entries:', entries.length);
    for (const [key, val] of entries.slice(0, 10)) {
      const portfolioId = key.args[0];
      const assetId = key.args[1];
      console.log('  Portfolio:', JSON.stringify(portfolioId.toHuman ? portfolioId.toHuman() : String(portfolioId)));
      console.log('  Asset:', assetId.toHex ? assetId.toHex() : String(assetId));
      console.log('  Balance:', val.toString());
    }
  } catch (e) {
    console.log('portfolioAssetBalances error:', e.message);
  }

  console.log('\n=== 4) Check who is custodian of Alice\'s non-default portfolio (number 1) ===');
  try {
    const cust = await api.query.portfolio.portfolioCustodian({ did: ALICE_DID, kind: { User: 1 } });
    console.log('custodian of portfolio #1:', cust.toHuman ? JSON.stringify(cust.toHuman()) : String(cust));
  } catch (e) {
    console.log('error:', e.message);
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
