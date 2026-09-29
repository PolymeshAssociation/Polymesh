const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const { cryptoWaitReady, mnemonicGenerate } = require('@polkadot/util-crypto');

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

  const mnemonic = mnemonicGenerate();
  const pair = keyring.addFromUri(mnemonic);
  console.log('\n=== New wallet created ===');
  console.log('Address:', pair.address);
  console.log('MNEMONIC (save this!):', mnemonic);

  console.log('\nRegistering DID (WITHOUT CDD)...');
  await tx(api, alice, api.tx.identity.registerDid(pair.address), 'register DID');

  const rec = await api.query.identity.keyRecords(pair.address);
  let did = null;
  if (rec.isSome) {
    const v = rec.unwrap();
    if (typeof v.type === 'string') {
      if ((v.type === 'PrimaryKey' || v.type === 'SecondaryKey') && v.value) did = v.value.toString();
    } else if (v.primaryKey) did = v.primaryKey.toString();
    else if (v.secondaryKey) did = v.secondaryKey.toString();
  }
  console.log('\n=== Result ===');
  console.log('New DID:', did);

  const entries = await api.query.identity.claims.entries();
  const hasCdd = entries.some(([key]) => {
    const first = key.args[0] || {};
    const t = (first.target?.toString() || '').toLowerCase();
    const ct = (first.claimType?.toString() || '').toLowerCase();
    return t === (did || '').toLowerCase() && ct.includes('customerduediligence');
  });
  console.log('Has CDD:', hasCdd ? 'YES' : 'NO (perfect for testing!)');

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
