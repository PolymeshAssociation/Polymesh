const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ASSET = '0x1fb383fc367588cb8de25ca5474caa22';
const ALICE_DID = '0x0100000000000000000000000000000000000000000000000000000000000000';
const TEST_DID = '0x5939247ff3f43a16fd0085f4f651a89ff3de8c756c3fd2dcd3a7b062e53aed42';
const HALF = 500000000n;

function tx(api, signer, t, label) {
  return new Promise((resolve, reject) => {
    t.signAndSend(signer, ({ status, dispatchError }) => {
      if (dispatchError) {
        let m = dispatchError.toString();
        if (dispatchError.isModule) {
          try {
            const d = api.registry.findMetaError(dispatchError.asModule);
            m = d.section + '.' + d.name;
          } catch {}
        }
        reject(new Error(label + ': ' + m));
      } else if (status.isInBlock || status.isFinalized) {
        console.log('OK ' + label);
        resolve();
      }
    });
  });
}

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  console.log('Transferring 500 tokens from Alice DID to Test User DID...');
  
  // AssetHolder در v8 فقط Identity هست
  const from = { Identity: ALICE_DID };
  const to = { Identity: TEST_DID };
  const fund = {
    description: {
      fungible: {
        assetId: ASSET,
        amount: HALF
      }
    },
    memo: null
  };

  console.log('  from:', JSON.stringify(from));
  console.log('  to:', JSON.stringify(to));

  await tx(api, alice, 
    api.tx.settlement.transferFunds(from, to, fund), 
    'transfer 500 to test user'
  );

  console.log('Done! Compliance was checked during transfer.');
  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
