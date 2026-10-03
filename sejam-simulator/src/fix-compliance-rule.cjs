const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ASSET = '0x1fb383fc367588cb8de25ca5474caa22';
const DEFAULT_CDD_ID = '0x' + '00'.repeat(32);

function tx(api, signer, t, label) {
  return new Promise((resolve, reject) => {
    t.signAndSend(signer, ({ status, dispatchError }) => {
      if (dispatchError) {
        let m = dispatchError.toString();
        if (dispatchError.isModule) {
          try { const d = api.registry.findMetaError(dispatchError.asModule); m = d.section + '.' + d.name; } catch {}
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

  const comps = await api.query.complianceManager.assetCompliances(ASSET);
  console.log('Current compliance:', JSON.stringify(comps.toHuman ? comps.toHuman() : comps));

  const reqs = comps.requirements || [];
  for (const r of reqs) {
    const id = (r.id !== undefined) ? r.id : (r.requirementId !== undefined ? r.requirementId : null);
    if (id === null) { console.log('skip requirement without id'); continue; }
    await tx(api, alice, api.tx.complianceManager.removeComplianceRequirement(ASSET, id), 'remove requirement ' + id);
  }

  const condition = {
    conditionType: { IsPresent: { CustomerDueDiligence: DEFAULT_CDD_ID } },
    issuers: [],
  };
  await tx(api, alice, api.tx.complianceManager.addComplianceRequirement(ASSET, [], [condition]), 'add open CDD rule (any issuer)');

  const after = await api.query.complianceManager.assetCompliances(ASSET);
  console.log('New compliance:', JSON.stringify(after.toHuman ? after.toHuman() : after));

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
