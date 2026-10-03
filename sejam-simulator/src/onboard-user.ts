import 'dotenv/config';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';

const POLKADOT_WS = process.env.POLKADOT_WS_URL || 'ws://127.0.0.1:9944';
const SEJAM_API_URL = process.env.SEJAM_API_URL || 'http://localhost:3001/api/sejam/verify';
const DEFAULT_CDD_ID = '0x0000000000000000000000000000000000000000000000000000000000000000';

type AnyApi = any;

function accountHex(api: AnyApi, addr: string): string {
  return api.createType('AccountId', addr).toHex();
}

function didFromKeyRecord(val: any): string | null {
  if (!val) return null;
  if (val.primaryKey) return val.primaryKey.toString();
  if (val.secondaryKey) return val.secondaryKey.toString();
  if (val.identity) return val.identity.toString();
  if (val.multiSigSigner) return null;
  const s = val.toString();
  if (s.startsWith('0x')) return s;
  return null;
}

async function findDidByPrimary(api: AnyApi, address: string): Promise<string | null> {
  const target = accountHex(api, address);
  const entries = await api.query.identity.didRecords.entries();
  for (const [key, val] of entries as any) {
    let rec = val as any;
    if (rec && rec.isSome !== undefined) rec = rec.isSome ? rec.unwrap() : null;
    if (!rec) continue;
    const pk = rec.primaryKey ?? rec.primary_key ?? rec.primaryKeyAccount ?? null;
    if (pk && api.createType('AccountId', pk.toString()).toHex() === target) {
      return key.args[0].toString();
    }
  }
  return null;
}

async function getDidForAccount(api: AnyApi, address: string): Promise<string | null> {
  const id = api.query.identity;
  if (id.keyRecords) {
    const opt = await id.keyRecords(address);
    if (opt.isSome) {
      const did = didFromKeyRecord(opt.unwrap());
      if (did) return did;
    }
  }
  return await findDidByPrimary(api, address);
}

// --- بررسی وجود کلیم CDD با روش سازگار (اسکن entries) ---
async function hasCddClaim(api: AnyApi, userDid: string): Promise<boolean> {
  const claimEntries = await api.query.identity.claims.entries(userDid);
  return claimEntries.some(([key, val]: any) => {
    const keyStr = (key.args[1]?.toString() || '').toLowerCase();
    const valStr = (val?.toString() || '').toLowerCase();
    return keyStr.includes('customerduediligence') || valStr.includes('customerduediligence');
  });
}

async function sendTx(api: AnyApi, signer: any, tx: any, label: string): Promise<void> {
  return new Promise((resolve, reject) => {
    tx.signAndSend(signer, ({ status, dispatchError }: any) => {
      if (dispatchError) {
        let errMsg = dispatchError.toString();
        if (dispatchError.isModule) {
          try {
            const decoded = api.registry.findMetaError(dispatchError.asModule);
            errMsg = `${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`;
          } catch { /* keep raw error */ }
        }
        reject(new Error(`${label} failed: ${errMsg}`));
      } else if (status.isInBlock || status.isFinalized) {
        console.log(`   ✅ ${label} included in block.`);
        resolve();
      }
    });
  });
}

async function onboardUser(nationalCode: string, sejamCode: string, userAddress: string) {
  console.log(`\n🚀 Starting onboarding for ${userAddress}`);

  console.log('1️⃣ Verifying with Sejam API...');
  try {
    const res = await fetch(SEJAM_API_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ nationalCode, sejamCode }),
    });
    const result = await res.json();
    if (result.status !== 'success') {
      console.error(`❌ Sejam verification failed: ${result.message}`);
      return;
    }
    console.log(`✅ Sejam verified: ${result.data.fullName}`);
  } catch (e) {
    console.error('❌ Cannot reach Sejam API (port 3001).', e);
    return;
  }

  console.log('2️⃣ Connecting to Polymesh node...');
  await cryptoWaitReady();
  const api: AnyApi = await ApiPromise.create({ provider: new WsProvider(POLKADOT_WS) });
  console.log('✅ Connected.');

  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');
  console.log(`✅ Registrar key loaded: ${alice.address}`);

  try {
    console.log('3️⃣ Resolving user DID...');
    let userDid = await getDidForAccount(api, userAddress);

    if (!userDid) {
      console.log('   ℹ️ No DID found — registering new DID...');
      await sendTx(api, alice, api.tx.identity.registerDid(userAddress), 'DID registration');
      userDid = await getDidForAccount(api, userAddress);
      if (!userDid) throw new Error('DID still not found after registration');
    }
    console.log(`   ✅ User DID: ${userDid}`);

    console.log('4️⃣ Checking existing CDD claim...');
    const alreadyHasCdd = await hasCddClaim(api, userDid);

    if (alreadyHasCdd) {
      console.log('   ℹ️ User already has a CDD claim — nothing to issue.');
    } else {
      console.log('   🖋️ Issuing CDD claim...');
      const claim = { CustomerDueDiligence: DEFAULT_CDD_ID };
      await sendTx(api, alice, api.tx.identity.addClaim(userDid, claim, null), 'CDD issuance');
    }

    console.log('\n🎉 Onboarding completed successfully!');
  } catch (e) {
    console.error('\n💥 Onboarding error:', e);
  } finally {
    await api.disconnect();
  }
}

async function main() {
  await cryptoWaitReady();
  const keyring = new Keyring({ type: 'sr25519' });
  const investor = keyring.addFromUri('//Investor1');
  console.log(`🧪 Test user (Investor1) derived address: ${investor.address}`);
  await onboardUser('0012345678', 'SJ-TEST-001', investor.address);
}

main();
