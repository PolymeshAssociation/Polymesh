import 'dotenv/config';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';

const CHAIN_URL = process.env.SETUP_CHAIN_URL || 'ws://127.0.0.1:9945';
const DEFAULT_CDD_ID = '0x' + '00'.repeat(32);

function didFromKeyRecord(v: any): string | null {
  if (!v) return null;
  if (typeof v.type === 'string') {
    if (v.type === 'PrimaryKey' || v.type === 'SecondaryKey') {
      return v.value ? v.value.toString() : null;
    }
    return null;
  }
  if (v.primaryKey) return v.primaryKey.toString();
  if (v.secondaryKey) return v.secondaryKey.toString();
  return null;
}

function hexToString(hex: string): string {
  if (hex.startsWith('0x')) hex = hex.slice(2);
  return Buffer.from(hex, 'hex').toString('utf8');
}

async function main() {
  await cryptoWaitReady();
  const api: any = await ApiPromise.create({ provider: new WsProvider(CHAIN_URL) });
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  console.log('--- Assets on chain (' + CHAIN_URL + ') ---');
  const names = await api.query.asset.assetNames.entries();
  let targetIdHex: string | null = null;
  
  for (const [key, val] of names as any) {
    const idHex = key.args[0].toHex();
    const nameHex = val.toString();
    const name = hexToString(nameHex);
    console.log('  ' + idHex + '  ->  "' + name + '"');
    if (name.includes('Tehran Stock 1')) targetIdHex = idHex;
  }
  
  if (!targetIdHex) { 
    console.error('\n❌ Asset "Tehran Stock 1" not found on ' + CHAIN_URL);
    console.error('   Make sure:');
    console.error('   1) Node on port 9945 is running');
    console.error('   2) setup-chain.ts was executed on ws://127.0.0.1:9945');
    process.exit(1); 
  }
  
  console.log('\n✅ Found AssetId: ' + targetIdHex);

  // استخراج DID آلیس
  const rec = await api.query.identity.keyRecords(alice.address);
  let aliceDid: string | null = null;
  if (rec.isSome) aliceDid = didFromKeyRecord(rec.unwrap());
  console.log('✅ Alice DID: ' + aliceDid);
  
  if (!aliceDid) { console.error('Alice DID not resolved!'); process.exit(1); }

  // ست کردن قانون Compliance
  const condition = {
    conditionType: { IsPresent: { CustomerDueDiligence: DEFAULT_CDD_ID } },
    issuers: [{ issuer: aliceDid, trustedFor: 'Any' }],
  };

  console.log('\n--- Setting Compliance Rule ---');
  await new Promise<void>((resolve, reject) => {
    api.tx.complianceManager.addComplianceRequirement(targetIdHex, [], [condition])
      .signAndSend(alice, ({ status, dispatchError }: any) => {
        if (dispatchError) {
          let msg = dispatchError.toString();
          if (dispatchError.isModule) {
            try { const d = api.registry.findMetaError(dispatchError.asModule); msg = d.section + '.' + d.name; } catch {}
          }
          reject(new Error(msg));
        } else if (status.isInBlock || status.isFinalized) {
          console.log('✅ Compliance rule set successfully!');
          resolve();
        }
      });
  });

  await api.disconnect();
}

main().catch(e => { console.error('Error:', e.message); process.exit(1); });
