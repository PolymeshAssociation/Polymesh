'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { getApi, isValidDid, getDidKeys, getDidForAddress, hasCdd, getHoldings, getPolyx, formatUnits } from '@/lib/chain';
import { DEV_ACCOUNTS } from '@/lib/devAccounts';

export default function TradePage() {
  const [mounted, setMounted] = useState(false);
  const [mnemonic, setMnemonic] = useState('');
  const [devUri, setDevUri] = useState('');
  const [signer, setSigner] = useState<any>(null);
  const [holdings, setHoldings] = useState<any[]>([]);
  const [polyx, setPolyx] = useState<bigint>(BigInt(0));
  const [recipientDid, setRecipientDid] = useState('');
  const [assetId, setAssetId] = useState('');
  const [amount, setAmount] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');

  useEffect(() => { setMounted(true); }, []);

  const loadSigner = async (pair: any) => {
    const api = await getApi();
    const did = await getDidForAddress(api, pair.address);
    if (!did) { setError('این کیف پول به هیچ DIDی متصل نیست. اول ثبت‌نام کنید.'); return; }
    const keys = await getDidKeys(api, did);
    const hold = await getHoldings(api, did, keys);
    const px = await getPolyx(api, [pair.address]);
    setSigner({ pair, address: pair.address, did });
    setHoldings(hold);
    setPolyx(px);
    if (hold.length > 0) setAssetId(hold[0].assetId);
  };

  const unlock = async () => {
    setError(''); setSuccess('');
    try {
      await cryptoWaitReady();
      const keyring = new Keyring({ type: 'sr25519' });
      const pair = keyring.addFromUri(mnemonic.trim());
      await loadSigner(pair);
    } catch (e: any) {
      setError('خطا در باز کردن کیف پول: ' + (e?.message || 'نامشخص'));
    }
  };

  const unlockDev = async () => {
    if (!devUri) return;
    setError(''); setSuccess('');
    try {
      await cryptoWaitReady();
      const keyring = new Keyring({ type: 'sr25519' });
      const pair = keyring.addFromUri(devUri);
      await loadSigner(pair);
    } catch (e: any) {
      setError('خطا در ورود با اکانت تستی: ' + (e?.message || 'نامشخص'));
    }
  };

  const refresh = async () => {
    const api = await getApi();
    const keys = await getDidKeys(api, signer.did);
    setHoldings(await getHoldings(api, signer.did, keys));
    setPolyx(await getPolyx(api, [signer.address]));
  };

  const send = async () => {
    setError(''); setSuccess('');
    const did = recipientDid.trim();
    if (!isValidDid(did)) { setError('DID گیرنده نامعتبر است.'); return; }
    if (did.toLowerCase() === signer.did.toLowerCase()) { setError('نمی‌توانید به خودتان انتقال دهید.'); return; }
    const amt = parseFloat(amount);
    if (!amt || amt <= 0) { setError('مقدار نامعتبر است.'); return; }
    const holding = holdings.find(h => h.assetId === assetId);
    if (!holding) { setError('دارایی انتخاب نشده.'); return; }
    const units = holding.divisible ? BigInt(Math.round(amt * 1e6)) : BigInt(Math.round(amt));
    if (units > holding.total) { setError('موجودی کافی نیست.'); return; }
    if (polyx === BigInt(0)) { setError('موجودی POLYX شما صفر است؛ برای پرداخت کارمزد تراکنش به POLYX نیاز دارید.'); return; }
    setBusy(true);
    try {
      const api = await getApi();
      const receiverCdd = await hasCdd(api, did);
      if (!receiverCdd) {
        setError('❌ گیرنده CDD ندارد — قانون Compliance این دارایی انتقال را رد می‌کند.');
        setBusy(false);
        return;
      }
      const receiverKeys = await getDidKeys(api, did);
      if (receiverKeys.length === 0) { setError('گیرنده هیچ کلیدی ندارد.'); setBusy(false); return; }
      const txx = api.tx.asset.transferAsset(assetId, receiverKeys[0], units, null);
      await new Promise<void>((resolve, reject) => {
        txx.signAndSend(signer.pair, ({ status, dispatchError }: any) => {
          if (dispatchError) {
            let msg = dispatchError.toString();
            if (dispatchError.isModule) {
              try { const d = api.registry.findMetaError(dispatchError.asModule); msg = d.section + '.' + d.name; } catch {}
            }
            reject(new Error(msg));
          } else if (status.isInBlock || status.isFinalized) { resolve(); }
          else if (status.isDropped || status.isInvalid || status.isFinalityTimeout) {
            reject(new Error('تراکنش توسط شبکه رد شد (' + (status.type || status.toString()) + ')'));
          }
        });
      });
      setSuccess('✅ انتقال با موفقیت در بلاک ثبت شد و قوانین Compliance رعایت شد.');
      await refresh();
    } catch (e: any) {
      setError('خطا در انتقال: ' + (e?.message || 'نامشخص'));
    } finally { setBusy(false); }
  };

  if (!mounted) return <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100" dir="rtl"></div>;

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 p-4" dir="rtl">
      <div className="max-w-3xl mx-auto py-8">
        <div className="flex items-center justify-between mb-6">
          <Link href="/" className="text-blue-600 hover:text-blue-800">← بازگشت</Link>
          <h1 className="text-3xl font-bold text-gray-800">انتقال سهام</h1>
        </div>

        {!signer ? (
          <div className="bg-white rounded-2xl shadow-xl p-6">
            <h2 className="text-xl font-bold text-gray-800 mb-4">🔐 ورود به کیف پول</h2>

            <div className="bg-gradient-to-br from-purple-50 to-pink-50 border border-purple-200 rounded-xl p-4 mb-4">
              <label className="block text-purple-900 font-bold mb-2 text-sm">🚀 انتخاب اکانت تستی (بدون نیاز به Mnemonic):</label>
              <select
                value={devUri}
                onChange={(e) => setDevUri(e.target.value)}
                className="w-full px-4 py-3 border border-purple-300 rounded-lg text-sm mb-3 text-gray-900"
              >
                <option value="">-- انتخاب کنید --</option>
                {DEV_ACCOUNTS.map(a => <option key={a.uri} value={a.uri}>{a.name}</option>)}
              </select>
              <button
                onClick={unlockDev}
                disabled={!devUri}
                className="w-full bg-gradient-to-r from-purple-600 to-pink-600 text-white font-bold py-3 rounded-lg hover:from-purple-700 hover:to-pink-700 disabled:opacity-50 transition"
              >
                ورود با اکانت تستی
              </button>
            </div>

            <div className="text-center text-xs text-gray-400 my-4">— یا ورود دستی —</div>

            <p className="text-gray-600 text-sm mb-2">کلمات بازیابی (Mnemonic) کیف پول خود را وارد کنید:</p>
            <textarea
              value={mnemonic}
              onChange={(e) => setMnemonic(e.target.value)}
              rows={3}
              placeholder="کلمه۱ کلمه۲ کلمه۳ ..."
              className="w-full px-4 py-3 border border-gray-300 rounded-lg font-mono text-sm focus:ring-2 focus:ring-blue-500 text-gray-900"
              dir="ltr"
            />
            <button
              onClick={unlock}
              disabled={!mnemonic.trim()}
              className="mt-3 w-full bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-bold py-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 disabled:opacity-50 transition"
            >
              باز کردن کیف پول
            </button>

            {error && <div className="mt-3 p-3 bg-red-50 border border-red-200 rounded text-red-700 text-sm">{error}</div>}
          </div>
        ) : (
          <div className="space-y-6">
            <div className="bg-white rounded-2xl shadow-xl p-6">
              <div className="flex justify-between text-sm mb-2">
                <span className="text-gray-600">DID شما:</span>
                <span className="font-mono text-xs text-gray-900" dir="ltr">{signer.did.slice(0, 18)}...</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-600">POLYX (کارمزد):</span>
                <span className="font-bold text-gray-900" dir="ltr">{formatUnits(polyx, true)}</span>
              </div>
            </div>

            <div className="bg-white rounded-2xl shadow-xl p-6">
              <h2 className="text-xl font-bold text-gray-800 mb-4">📤 فرم انتقال</h2>
              <div className="space-y-4">
                <div>
                  <div className="flex items-center justify-between mb-2"><label className="block text-gray-700 text-sm">DID گیرنده:</label><Link href="/accounts" target="_blank" className="text-xs text-purple-600 hover:text-purple-800">📒 کپی DID از دفترچه</Link></div>
                  <input
                    value={recipientDid}
                    onChange={(e) => setRecipientDid(e.target.value)}
                    placeholder="0x..."
                    dir="ltr"
                    className="w-full px-4 py-3 border border-gray-300 rounded-lg font-mono text-sm text-gray-900"
                  />
                </div>
                <div>
                  <label className="block text-gray-700 mb-2 text-sm">دارایی:</label>
                  <select
                    value={assetId}
                    onChange={(e) => setAssetId(e.target.value)}
                    className="w-full px-4 py-3 border border-gray-300 rounded-lg text-sm text-gray-900"
                  >
                    {holdings.map(h => <option key={h.assetId} value={h.assetId}>{h.name} (موجودی: {formatUnits(h.total, h.divisible)})</option>)}
                  </select>
                </div>
                <div>
                  <label className="block text-gray-700 mb-2 text-sm">مقدار:</label>
                  <input
                    value={amount}
                    onChange={(e) => setAmount(e.target.value)}
                    type="number"
                    placeholder="0"
                    dir="ltr"
                    className="w-full px-4 py-3 border border-gray-300 rounded-lg text-sm text-gray-900"
                  />
                </div>
                <button
                  onClick={send}
                  disabled={busy}
                  className="w-full bg-gradient-to-r from-green-600 to-emerald-600 text-white font-bold py-3 rounded-lg hover:from-green-700 hover:to-emerald-700 disabled:opacity-50 transition"
                >
                  {busy ? '⏳ در حال ارسال...' : '✅ امضا و ارسال'}
                </button>
                {error && <div className="p-3 bg-red-50 border border-red-200 rounded text-red-700 text-sm">{error}</div>}
                {success && <div className="p-3 bg-green-50 border border-green-200 rounded text-green-700 text-sm">{success}</div>}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
