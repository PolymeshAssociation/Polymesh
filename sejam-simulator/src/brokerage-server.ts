import 'dotenv/config';
import express from 'express';
import cors from 'cors';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';

const PORT = Number(process.env.BROKERAGE_PORT || 3002);
const POLKADOT_WS = process.env.SETUP_CHAIN_URL || 'ws://127.0.0.1:9945';
const SEJAM_API_URL = process.env.SEJAM_API_URL || 'http://localhost:3001/api/sejam/verify';
const DEFAULT_CDD_ID = '0x' + '00'.repeat(32);

let api: any;
let alice: any;

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

async function getDidForAccount(address: string): Promise<string | null> {
  const id = api.query.identity;
  if (id.keyRecords) {
    const opt = await id.keyRecords(address);
    if (opt.isSome) {
      const did = didFromKeyRecord(opt.unwrap());
      if (did) return did;
    }
  }
  const entries = await id.didRecords.entries();
  const target = api.createType('AccountId', address).toHex();
  for (const [key, val] of entries as any) {
    let rec = val as any;
    if (rec && rec.isSome !== undefined) rec = rec.isSome ? rec.unwrap() : null;
    if (!rec) continue;
    const pk = rec.primaryKey ?? rec.primary_key ?? null;
    if (pk && api.createType('AccountId', pk.toString()).toHex() === target) {
      return key.args[0].toString();
    }
  }
  return null;
}

async function hasCddClaim(userDid: string): Promise<boolean> {
  const claimEntries = await api.query.identity.claims.entries(userDid);
  return claimEntries.some(([key, val]: any) => {
    const keyStr = (key.args[1]?.toString() || '').toLowerCase();
    const valStr = (val?.toString() || '').toLowerCase();
    return keyStr.includes('customerduediligence') || valStr.includes('customerduediligence');
  });
}

function sendTx(signer: any, tx: any, label: string): Promise<void> {
  return new Promise((resolve, reject) => {
    tx.signAndSend(signer, ({ status, dispatchError }: any) => {
      if (dispatchError) {
        let errMsg = dispatchError.toString();
        if (dispatchError.isModule) {
          try {
            const d = api.registry.findMetaError(dispatchError.asModule);
            errMsg = d.section + '.' + d.name;
          } catch {}
        }
        reject(new Error(label + ' failed: ' + errMsg));
      } else if (status.isInBlock || status.isFinalized) {
        console.log('   OK ' + label + ' in block.');
        resolve();
      }
    });
  });
}

async function verifyWithSejam(nationalCode: string, sejamCode: string): Promise<any> {
  const res = await fetch(SEJAM_API_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ nationalCode, sejamCode }),
  });
  const result = await res.json();
  if (result.status !== 'success') throw new Error('Sejam verification failed');
  return result.data;
}

async function onboardUser(nationalCode: string, sejamCode: string, userAddress: string) {
  const sejamData = await verifyWithSejam(nationalCode, sejamCode);
  let userDid = await getDidForAccount(userAddress);
  let didRegistered = false;
  if (!userDid) {
    await sendTx(alice, api.tx.identity.registerDid(userAddress), 'DID registration');
    userDid = await getDidForAccount(userAddress);
    if (!userDid) throw new Error('DID not found after registration');
    didRegistered = true;
  }
  const alreadyHadCdd = await hasCddClaim(userDid);
  let cddIssued = false;
  if (!alreadyHadCdd) {
    await sendTx(alice, api.tx.identity.addClaim(userDid, { CustomerDueDiligence: DEFAULT_CDD_ID }, null), 'CDD issuance');
    cddIssued = true;
  }
  return { fullName: sejamData.fullName, did: userDid, didRegistered, cddIssued, alreadyHadCdd };
}

const app = express();
app.use(cors());
app.use(express.json());

app.get('/api/brokerage/health', (_req, res) => {
  res.json({ status: 'ok', chain: POLKADOT_WS });
});

app.post('/api/brokerage/register', async (req, res) => {
  const { nationalCode, sejamCode, address } = req.body || {};
  if (!nationalCode || !sejamCode || !address) {
    return res.status(400).json({ status: 'error', message: 'missing fields' });
  }
  try {
    console.log('Register request for ' + address);
    const data = await onboardUser(nationalCode, sejamCode, address);
    res.json({ status: 'success', data });
  } catch (e: any) {
    res.status(500).json({ status: 'error', message: e?.message || String(e) });
  }
});

app.get('/api/brokerage/status/:address', async (req, res) => {
  try {
    const did = await getDidForAccount(req.params.address);
    if (!did) return res.json({ status: 'success', data: { hasDid: false, hasCdd: false } });
    const hasCdd = await hasCddClaim(did);
    res.json({ status: 'success', data: { hasDid: true, did, hasCdd } });
  } catch (e: any) {
    res.status(500).json({ status: 'error', message: e?.message || String(e) });
  }
});

async function main() {
  await cryptoWaitReady();
  api = await ApiPromise.create({ provider: new WsProvider(POLKADOT_WS) });
  const keyring = new Keyring({ type: 'sr25519' });
  alice = keyring.addFromUri('//Alice');
  console.log('Brokerage connected to ' + POLKADOT_WS + ' as ' + alice.address);
  app.listen(PORT, () => console.log('Brokerage API running on http://localhost:' + PORT));
}

main().catch(e => { console.error(e); process.exit(1); });
