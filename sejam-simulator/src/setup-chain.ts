import 'dotenv/config';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';

const CHAIN_URL = process.env.SETUP_CHAIN_URL || 'ws://127.0.0.1:9945';
const DEFAULT_CDD_ID = '0x' + '00'.repeat(32);
const FUND_AMOUNT = 100_000_000;

type AnyApi = any;

function sendTx(api: AnyApi, signer: any, tx: any, label: string): Promise<any[]> {
  return new Promise((resolve, reject) => {
    tx.signAndSend(signer, ({ status, dispatchError, events }: any) => {
      if (dispatchError) {
        let errMsg = dispatchError.toString();
        if (dispatchError.isModule) {
          try {
            const d = api.registry.findMetaError(dispatchError.asModule);
            errMsg = d.section + '.' + d.name + ': ' + d.docs.join(' ');
          } catch {}
        }
        reject(new Error(label + ' failed: ' + errMsg));
      } else if (status.isInBlock || status.isFinalized) {
        console.log('   ✅ ' + label + ' in block.');
        resolve(events || []);
      }
    });
  });
}

async function main() {
  await cryptoWaitReady();
  const api: AnyApi = await ApiPromise.create({ provider: new WsProvider(CHAIN_URL) });
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');
  const investor1 = keyring.addFromUri('//Investor1');
  const investor2 = keyring.addFromUri('//Investor2');
  console.log('✅ Connected to ' + CHAIN_URL + ' as ' + alice.address);

  // A) تامین موجودی سرمایه‌گذاران
  console.log('\n--- A) Funding Investors ---');
  for (const pair of [investor1, investor2]) {
    console.log('💰 Funding ' + pair.address.slice(0, 8) + '...');
    try {
      await sendTx(api, alice, api.tx.balances.transferKeepAlive(pair.address, FUND_AMOUNT), 'Fund ' + pair.address.slice(0, 8));
    } catch (e: any) { console.log('   ⚠️ ' + e.message); }
  }

  // B) ثبت نوع دارایی سفارشی TehranEquity
  console.log('\n--- B) Register Custom Asset Type ---');
  try {
    const regTx = api.tx.asset.registerCustomAssetType;
    if (regTx) {
      await sendTx(api, alice, regTx('TehranEquity'), 'Register custom type TehranEquity');
    }
  } catch (e: any) { console.log('   ⚠️ ' + e.message); }

  // C) ساخت دارایی با نوع سفارشی (5 آرگومان)
  console.log('\n--- C) Create Asset with Custom Type ---');
  let assetId: string | null = null;
  try {
    const createTx = api.tx.asset.createAssetWithCustomType;
    if (createTx) {
      // آرگومان‌ها:
      // 0. assetName: Bytes
      // 1. divisible: bool
      // 2. customAssetType: Bytes (نام نوع سفارشی)
      // 3. assetIdentifiers: Vec (آرایه خالی)
      // 4. fundingRoundName: Option (null)
      const events = await sendTx(api, alice, 
        createTx('Tehran Stock 1', true, 'TehranEquity', [], null),
        'Create asset with custom type'
      );
      
      // استخراج assetId از event
      const created = events.map((e: any) => e.event).filter((ev: any) => 
        ev && ev.section === 'asset' && ev.method === 'AssetCreated'
      );
      if (created.length) {
        assetId = created[0].data[0].toString();
        console.log('   🎯 AssetId: ' + assetId);
      }
    }
  } catch (e: any) { console.log('   ⚠️ ' + e.message); }

  // D) ساخت Venue (3 آرگومان)
  console.log('\n--- D) Create Venue ---');
  try {
    const venueTx = api.tx.settlement.createVenue;
    if (venueTx) {
      // آرگومان‌ها:
      // 0. details: Bytes
      // 1. signers: BTreeSet<AccountId> (آرایه خالی)
      // 2. typ: VenueType ('Other')
      await sendTx(api, alice, venueTx('Tehran Bourse Main Venue', [], 'Other'), 'Create venue');
    }
  } catch (e: any) { console.log('   ⚠️ ' + e.message); }

  // E) قوانین Compliance
  console.log('\n--- E) Set Compliance Rules ---');
  if (assetId) {
    try {
      const rec = await api.query.identity.keyRecords(alice.address);
      let aliceDid: string | null = null;
      if (rec.isSome) {
        const v = rec.unwrap() as any;
        aliceDid = (v.primaryKey || v.secondaryKey || v).toString();
      }
      
      const condition = {
        conditionType: { IsPresent: { CustomerDueDiligence: DEFAULT_CDD_ID } },
        issuers: aliceDid ? [{ issuer: aliceDid, trustedFor: 'Any' }] : [],
      };
      
      await sendTx(api, alice, 
        api.tx.complianceManager.addComplianceRequirement(assetId, [], [condition]),
        'Set compliance (receiver CDD)'
      );
    } catch (e: any) { console.log('   ⚠️ ' + e.message); }
  } else {
    console.log('   ⚠️ No assetId — compliance skipped.');
  }

  console.log('\n🎉 Chain setup finished!');
  console.log('📊 AssetId: ' + (assetId ?? '(none)'));
  
  if (assetId) {
    console.log('\n✅ Your Tehran Bourse network is ready!');
    console.log('   - Investors funded with 100 POLYX');
    console.log('   - Custom type TehranEquity registered');
    console.log('   - Asset created: Tehran Stock 1');
    console.log('   - Venue created: Tehran Bourse Main Venue');
    console.log('   - Compliance rule set: Receiver must have CDD');
  }
  
  await api.disconnect();
}

main().catch(e => { console.error('Fatal error:', e); process.exit(1); });
