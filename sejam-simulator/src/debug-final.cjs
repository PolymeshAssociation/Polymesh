const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ALICE_DID = '0x0100000000000000000000000000000000000000000000000000000000000000';
const ASSET = '0x1fb383fc367588cb8de25ca5474caa22';

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });

  console.log('\n=== portfolio.acceptPortfolioCustody signature ===');
  const apc = api.tx.portfolio.acceptPortfolioCustody;
  if (apc) {
    console.log('args count:', apc.meta.args.length);
    apc.meta.args.forEach((arg, i) => {
      console.log('  arg' + i + ': ' + arg.name + ' : ' + arg.type);
    });
  }

  console.log('\n=== Testing AssetHolder with different formats ===');
  const tests = [
    { Account: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY' },
    { Portfolio: { did: ALICE_DID, kind: 'Default' } },
  ];
  for (const holder of tests) {
    try {
      const bal = await api.query.asset.balanceOf(ASSET, holder);
      console.log('holder:', JSON.stringify(holder), '→ balance:', bal.toString());
    } catch (e) {
      console.log('holder:', JSON.stringify(holder), '→ error:', e.message.slice(0, 80));
    }
  }

  console.log('\n=== Testing PortfolioId structure ===');
  try {
    const pid = api.createType('PolymeshPrimitivesPortfolioId', { did: ALICE_DID, kind: 'Default' });
    console.log('PortfolioId created:', JSON.stringify(pid.toJSON()));
  } catch (e) {
    console.log('PortfolioId error:', e.message.slice(0, 80));
  }

  try {
    const pid2 = api.createType('PolymeshPrimitivesPortfolioId', {
      did: ALICE_DID,
      kind: { default: null }
    });
    console.log('PortfolioId with kind enum:', JSON.stringify(pid2.toJSON()));
  } catch (e) {
    console.log('PortfolioId enum error:', e.message.slice(0, 80));
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
