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
    t.signAndSend(signer, ({ status, dispatchError, events }) => {
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
        resolve(events || []);
      }
    });
  });
}

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  console.log('Step 1: Creating a new portfolio for Alice...');
  const createEvents = await tx(api, alice, 
    api.tx.portfolio.createPortfolio('Alice Trading Portfolio'), 
    'create new portfolio'
  );

  let portfolioNumber = null;
  for (const { event } of createEvents) {
    if (event.section === 'portfolio' && event.method === 'PortfolioCreated') {
      portfolioNumber = event.data[1]; // Second element is portfolio number
      console.log('  Created portfolio number:', portfolioNumber.toString());
      break;
    }
  }

  if (!portfolioNumber) {
    throw new Error('Could not extract portfolio number from PortfolioCreated event');
  }

  console.log('Step 2: Moving tokens from default to new portfolio...');
  const fromPortfolio = { did: ALICE_DID, kind: 'Default' };
  const toPortfolio = { did: ALICE_DID, kind: { User: parseInt(portfolioNumber.toString()) } };
  const fund = {
    description: {
      fungible: {
        assetId: ASSET,
        amount: HALF
      }
    },
    memo: null
  };

  await tx(api, alice, 
    api.tx.portfolio.movePortfolioFunds(fromPortfolio, toPortfolio, [fund]), 
    'move tokens to new portfolio'
  );

  console.log('Step 3: Transferring 500 tokens to test user from new portfolio...');
  const from = { Portfolio: toPortfolio };
  const to = { Portfolio: { did: TEST_DID, kind: 'Default' } };

  await tx(api, alice, 
    api.tx.settlement.transferFunds(from, to, fund), 
    'transfer 500 to test user'
  );

  console.log('Done!');
  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
