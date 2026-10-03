const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const FOOLAD = '0x0204d20efb578829a98efc7b893004ca';

function tx(api, signer, t, label) {
  return new Promise((resolve, reject) => {
    t.signAndSend(signer, ({ status, dispatchError }) => {
      if (dispatchError) {
        let m = dispatchError.toString();
        if (dispatchError.isModule) {
          try { const d = api.registry.findMetaError(dispatchError.asModule); m = d.section + '.' + d.name; } catch {}
        }
        reject(new Error(label + ': ' + m));
      } else if (status.isInBlock || status.isFinalized) { console.log('  OK ' + label); resolve(); }
    });
  });
}

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');
  const sara = keyring.addFromUri('//TehranInv//2');

  console.log('Step 1: try direct transfer (1 token) Alice->Sara ...');
  let failed = false;
  try {
    await tx(api, alice, api.tx.asset.transferAsset(FOOLAD, sara.address, 1000000n, null), 'transfer');
  } catch (e) { console.log('  transfer failed: ' + e.message); failed = true; }

  if (failed) {
    console.log('\nStep 2: normalize -> redeem portfolio-level, re-issue with null ...');
    try { await tx(api, alice, api.tx.asset.redeem(FOOLAD, 10000000n, 'DefaultPortfolio'), 'redeem portfolio-level'); } catch (e) { console.log('  redeem note: ' + e.message); }
    try { await tx(api, alice, api.tx.asset.issue(FOOLAD, 10000000n, null), 'issue with null'); } catch (e) { console.log('  issue note: ' + e.message); }

    console.log('\nStep 3: retry transfer ...');
    await tx(api, alice, api.tx.asset.transferAsset(FOOLAD, sara.address, 1000000n, null), 'transfer retry');
  }

  console.log('\n=== Final storage for Foolad ===');
  const b = await api.query.asset.balanceOf.entries(FOOLAD);
  for (const [k, v] of b) console.log('  DID-level:', JSON.stringify(k.args[1].toHuman()), '=>', v.toString());
  const p = await api.query.portfolio.portfolioAssetBalances.entries();
  const mine = p.filter(([k]) => k.args[1].toString() === FOOLAD);
  console.log('  portfolio-level entries:', mine.length);

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
