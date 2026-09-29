const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const DID = process.argv[2] || '0x0100000000000000000000000000000000000000000000000000000000000000';

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });
  console.log('Target DID:', DID);
  console.log('claims storage exists:', !!api.query.identity.claims);

  const entries = await api.query.identity.claims.entries(DID);
  console.log('entries count:', entries.length);

  for (const [key, val] of entries.slice(0, 5)) {
    console.log('--- entry ---');
    console.log('key args count:', key.args.length);
    key.args.forEach((a, i) => {
      console.log('  arg' + i + ':', JSON.stringify(a.toHuman ? a.toHuman() : String(a)));
    });
    console.log('VAL:', JSON.stringify(val.toHuman ? val.toHuman() : String(val)));
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
