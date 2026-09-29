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

  console.log('=== Checking available complianceManager methods ===');
  const methods = Object.keys(api.tx.complianceManager || {}).filter(m => m.toLowerCase().includes('trust'));
  console.log('Trust-related methods:', methods);

  console.log('\n=== Current trusted issuers for asset ===');
  try {
    const trusted = await api.query.complianceManager.trustedClaimIssuers.entries(ASSET);
    console.log('Trusted issuers count:', trusted.length);
    for (const [key, val] of trusted) {
      console.log('  claimType:', key.args[1] ? key.args[1].toString() : 'n/a');
      console.log('  issuers:', val.toHuman ? JSON.stringify(val.toHuman()) : String(val));
    }
  } catch (e) {
    console.log('Error reading trusted issuers:', e.message);
  }

  console.log('\n=== Adding Alice as trusted CDD issuer ===');
  const issuerTarget = { Identity: ALICE_DID };
  const claimType = 'CustomerDueDiligence';
  
  // Try different method signatures
  const attempts = [
    () => api.tx.complianceManager.addDefaultTrustedClaimIssuer(ASSET, issuerTarget),
    () => api.tx.complianceManager.addTrustedClaimIssuer(ASSET, issuerTarget, claimType),
  ];
  
  let success = false;
  for (const f of attempts) {
    try {
      const txx = f();
      await tx(api, alice, txx, 'add trusted issuer');
      success = true;
      break;
    } catch (e) {
      console.log('  attempt failed:', e.message);
    }
  }
  
  if (!success) {
    console.error('Could not add trusted issuer with any known method');
    process.exit(1);
  }

  console.log('\n=== Verifying trusted issuers after addition ===');
  try {
    const trusted = await api.query.complianceManager.trustedClaimIssuers.entries(ASSET);
    console.log('Trusted issuers count:', trusted.length);
    for (const [key, val] of trusted) {
      console.log('  claimType:', key.args[1] ? key.args[1].toString() : 'n/a');
      console.log('  issuers:', val.toHuman ? JSON.stringify(val.toHuman()) : String(val));
    }
  } catch (e) {
    console.log('Error:', e.message);
  }

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
