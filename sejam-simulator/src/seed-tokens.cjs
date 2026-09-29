const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ASSET = '0x1fb383fc367588cb8de25ca5474caa22';
const ALICE_DID = '0x0100000000000000000000000000000000000000000000000000000000000000';
const TEST_DID = '0x5939247ff3f43a16fd0085f4f651a89ff3de8c756c3fd2dcd3a7b062e53aed42';
const AMT = 1000000000n;
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

function buildHolder(did, kind = 'Default') {
  return { Portfolio: { did, kind } };
}

function buildFund(assetId, amount) {
  return {
    description: {
      fungible: {
        assetId: assetId,
        amount: amount
      }
    },
    memo: null
  };
}

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  console.log('Step 1: Issuing 1000 tokens to Alice (Default Portfolio)...');
  await tx(api, alice, api.tx.asset.issue(ASSET, AMT, null), 'issue 1000 to Alice Default Portfolio');

  console.log('Step 2: Moving 500 tokens from Asset Default to Alice DID Portfolio...');
  // انتقال از Portfolio پیش‌فرض Asset به Portfolio آلیس
  // Asset Default Portfolio = { did: 0x000...000, kind: 'Default' } (DID صفر = Asset Default)
  const assetDefaultHolder = { Portfolio: { did: '0x0000000000000000000000000000000000000000000000000000000000000000', kind: 'Default' } };
  const aliceHolder = buildHolder(ALICE_DID, 'Default');
  const fundToAlice = buildFund(ASSET, HALF);
  
  await tx(api, alice, api.tx.settlement.transferFunds(assetDefaultHolder, aliceHolder, fundToAlice), 'move 500 to Alice portfolio');

  console.log('Step 3: Transferring 500 tokens from Alice to test user...');
  const testHolder = buildHolder(TEST_DID, 'Default');
  const fundToTest = buildFund(ASSET, HALF);

  await tx(api, alice, api.tx.settlement.transferFunds(aliceHolder, testHolder, fundToTest), 'transfer 500 to test user');

  console.log('Done!');
  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
