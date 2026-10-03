const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ASSET = '0x1fb383fc367588cb8de25ca5474caa22';
const ALICE_DID = '0x0100000000000000000000000000000000000000000000000000000000000000';

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

  console.log('=== Current trusted issuers ===');
  let val = await api.query.complianceManager.trustedClaimIssuer(ASSET);
  console.log('Before:', val.toHuman ? JSON.stringify(val.toHuman()) : String(val));

  console.log('\n=== Adding Alice as trusted issuer for ANY claim type ===');
  const trustedIssuer = {
    issuer: ALICE_DID,
    trustedFor: 'Any'
  };
  
  await tx(api, alice, api.tx.complianceManager.addDefaultTrustedClaimIssuer(ASSET, trustedIssuer), 'add trusted issuer');

  console.log('\n=== Verifying ===');
  val = await api.query.complianceManager.trustedClaimIssuer(ASSET);
  console.log('After:', val.toHuman ? JSON.stringify(val.toHuman()) : String(val));

  console.log('\n✅ Alice is now a trusted issuer for all claim types on this asset');
  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
