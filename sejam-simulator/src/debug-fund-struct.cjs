const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });

  console.log('\n=== Testing Fund structure ===');
  try {
    const testFund = api.createType('PolymeshPrimitivesPortfolioFund', {
      assetId: '0x1fb383fc367588cb8de25ca5474caa22',
      amount: 500000000n
    });
    console.log('Fund created successfully');
    console.log('Fund keys:', Object.keys(testFund.toJSON()));
    console.log('Fund toJSON:', JSON.stringify(testFund.toJSON(), null, 2));
  } catch (e) {
    console.log('Error creating Fund with assetId+amount:', e.message);
  }

  try {
    const testFund2 = api.createType('PolymeshPrimitivesPortfolioFund', {
      asset_id: '0x1fb383fc367588cb8de25ca5474caa22',
      amount: 500000000n
    });
    console.log('Fund created with asset_id');
    console.log('Fund keys:', Object.keys(testFund2.toJSON()));
  } catch (e) {
    console.log('Error with asset_id:', e.message);
  }

  console.log('\n=== Testing AssetHolder structure ===');
  try {
    const testHolder = api.createType('PolymeshPrimitivesAssetAssetHolder', {
      Portfolio: {
        did: '0x0100000000000000000000000000000000000000000000000000000000000000',
        kind: 'Default'
      }
    });
    console.log('AssetHolder created successfully');
    console.log('AssetHolder toJSON:', JSON.stringify(testHolder.toJSON(), null, 2));
  } catch (e) {
    console.log('Error creating AssetHolder:', e.message);
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
