const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });
  const head = await api.rpc.chain.getHeader();
  const n = head.number.toNumber();
  const samples = {};

  for (let i = n; i > Math.max(0, n - 40); i--) {
    const hash = await api.rpc.chain.getBlockHash(i);
    const events = await api.query.system.events.at(hash);
    for (const rec of events) {
      const { event } = rec;
      const key = event.section + '.' + event.method;
      if (['asset', 'settlement', 'balances', 'identity'].includes(event.section) && !samples[key]) {
        samples[key] = JSON.stringify(event.data.toHuman ? event.data.toHuman() : event.data);
      }
    }
  }

  console.log('=== event types seen in last 40 blocks ===');
  Object.keys(samples).forEach(k => console.log(k + '  =>  ' + samples[k]));
  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
