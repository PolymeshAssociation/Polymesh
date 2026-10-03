const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });

  console.log('\n=== api.tx.asset (all methods) ===');
  for (const name of Object.keys(api.tx.asset || {})) {
    console.log('  asset.' + name);
  }

  console.log('\n=== api.tx.settlement (all methods) ===');
  for (const name of Object.keys(api.tx.settlement || {})) {
    console.log('  settlement.' + name);
  }

  console.log('\n=== Transfer-related candidates ===');
  const candidates = ['transfer', 'transferFunds', 'transferWithMemo',
    'controllerTransfer', 'unsafeAcceptTransfer', 'unsafeRejectTransfer',
    'createInstruction', 'affirmInstruction', 'executeInstruction',
    'rejectInstruction', 'acceptPrimaryKey', 'preApproveTicker'];
  for (const c of candidates) {
    const exists = (api.tx.asset && api.tx.asset[c]) || (api.tx.settlement && api.tx.settlement[c]);
    console.log('  ' + c + ': ' + (exists ? 'YES' : 'no'));
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
