const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const TEST_ADDR = '5CkxfwztUJCTPfNvXWRGsqi7fQLBfRr4Zu4bes3ybwEg8NGq';
const AMT = 100000000n; // 100 POLYX

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  await new Promise((res, rej) => {
    api.tx.balances.transferKeepAlive(TEST_ADDR, AMT).signAndSend(alice, ({ status, dispatchError }) => {
      if (dispatchError) rej(new Error(dispatchError.toString()));
      else if (status.isInBlock || status.isFinalized) { console.log('OK funded 100 POLYX to test user'); res(); }
    });
  });

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
