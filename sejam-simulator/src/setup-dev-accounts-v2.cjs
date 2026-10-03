const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const ASSET = '0x1fb383fc367588cb8de25ca5474caa22';
const CDD_ID = '0x' + '00'.repeat(32);
const ALICE_DID = '0x0100000000000000000000000000000000000000000000000000000000000000';
const POLYX = 100000000n;
const TOKENS = 500000000n;

const ACCOUNTS = [
  { name: 'Reza', uri: '//TehranInv//1', cdd: true, tokens: true },
  { name: 'Sara', uri: '//TehranInv//2', cdd: true, tokens: true },
  { name: 'NoCdd', uri: '//TehranInv//3', cdd: false, tokens: false },
];

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

async function getDidForAddress(api, address) {
  try {
    const opt = await api.query.identity.keyRecords(address);
    if (opt.isSome) {
      const v = opt.unwrap();
      if (typeof v.type === 'string') {
        if ((v.type === 'PrimaryKey' || v.type === 'SecondaryKey') && v.value) return v.value.toString();
        return null;
      }
      if (v.primaryKey) return v.primaryKey.toString();
      if (v.secondaryKey) return v.secondaryKey.toString();
    }
  } catch {}
  return null;
}

async function hasCdd(api, did) {
  const entries = await api.query.identity.claims.entries();
  return entries.some(([key, val]) => {
    const first = key.args[0] || {};
    const t = (first.target ? first.target.toString() : '').toLowerCase();
    const s = ((first.claimType ? first.claimType.toString() : '') + (val ? val.toString() : '')).toLowerCase();
    return t === did.toLowerCase() && s.includes('customerduediligence');
  });
}

async function main() {
  await cryptoWaitReady();
  const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9945') });
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  console.log('=== Step 0: Ensure Alice has self-issued CDD (trusted) ===');
  const comps = await api.query.complianceManager.assetCompliances(ASSET);
  console.log('Current compliance:', JSON.stringify(comps.toHuman ? comps.toHuman() : comps));

  console.log('Issuing fresh CDD for Alice (by Alice)...');
  await tx(api, alice, api.tx.identity.addClaim(ALICE_DID, { CustomerDueDiligence: CDD_ID }, null), 'Alice self-CDD');

  console.log('\n=== Step 1: Setting compliance rules ===');
  const reqs = comps.requirements || [];
  for (const r of reqs) {
    const id = (r.id !== undefined) ? r.id : (r.requirementId !== undefined ? r.requirementId : null);
    if (id === null) continue;
    await tx(api, alice, api.tx.complianceManager.removeComplianceRequirement(ASSET, id), 'remove req ' + id);
  }
  const cond = {
    conditionType: { IsPresent: { CustomerDueDiligence: CDD_ID } },
    issuers: [],
  };
  await tx(api, alice, api.tx.complianceManager.addComplianceRequirement(ASSET, [], [cond]), 'add open CDD rule');

  for (const acc of ACCOUNTS) {
    const pair = keyring.addFromUri(acc.uri);
    console.log('\n=== ' + acc.name + ' (' + pair.address + ') ===');

    let did = await getDidForAddress(api, pair.address);
    if (!did) {
      await tx(api, alice, api.tx.identity.registerDid(pair.address), 'registerDid');
      did = await getDidForAddress(api, pair.address);
    } else console.log('  DID exists');
    console.log('  DID: ' + did);

    if (acc.cdd) {
      if (!(await hasCdd(api, did))) {
        await tx(api, alice, api.tx.identity.addClaim(did, { CustomerDueDiligence: CDD_ID }, null), 'CDD');
      } else console.log('  CDD exists');
    } else console.log('  (no CDD by design)');

    const info = await api.query.system.account(pair.address);
    if (BigInt(info.data.free.toString()) < POLYX) {
      await tx(api, alice, api.tx.balances.transferKeepAlive(pair.address, POLYX), 'fund POLYX');
    } else console.log('  POLYX ok');

    if (acc.tokens) {
      const bal = await api.query.asset.balanceOf(ASSET, did);
      if (BigInt(bal.toString()) < TOKENS) {
        await tx(api, alice, api.tx.asset.transferAsset(ASSET, pair.address, TOKENS, null), 'fund tokens');
      } else console.log('  tokens ok');
    }
  }

  console.log('\n✅ All dev accounts ready');
  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
