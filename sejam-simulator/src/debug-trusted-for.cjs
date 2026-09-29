const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ALICE_DID = '0x0100000000000000000000000000000000000000000000000000000000000000';

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });

  console.log('=== Testing TrustedFor structure ===');
  const variations = [
    { name: 'Any', val: 'Any' },
    { name: 'Any (object)', val: { Any: null } },
    { name: 'Specific with CDD name', val: { Specific: ['CustomerDueDiligence'] } },
    { name: 'Specific with enum', val: { Specific: [{ CustomerDueDiligence: '0x' + '00'.repeat(32) }] } },
    { name: 'All', val: 'All' },
  ];

  for (const v of variations) {
    try {
      const t = api.createType('PolymeshPrimitivesConditionTrustedFor', v.val);
      console.log('OK ' + v.name + ':', JSON.stringify(t.toJSON()));
    } catch (e) {
      console.log('FAIL ' + v.name + ':', e.message.slice(0, 120));
    }
  }

  console.log('\n=== Testing full TrustedIssuer struct ===');
  const fullVariations = [
    { name: 'issuer direct + Any', val: { issuer: ALICE_DID, trustedFor: 'Any' } },
    { name: 'issuer direct + Any (obj)', val: { issuer: ALICE_DID, trustedFor: { Any: null } } },
    { name: 'issuer direct + Specific CDD', val: { issuer: ALICE_DID, trustedFor: { Specific: ['CustomerDueDiligence'] } } },
  ];

  for (const v of fullVariations) {
    try {
      const t = api.createType('PolymeshPrimitivesConditionTrustedIssuer', v.val);
      console.log('OK ' + v.name + ':', JSON.stringify(t.toJSON()));
    } catch (e) {
      console.log('FAIL ' + v.name + ':', e.message.slice(0, 120));
    }
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
