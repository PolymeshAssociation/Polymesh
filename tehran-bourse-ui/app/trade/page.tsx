'use client';

import { useState, useEffect } from 'react';
import { useWallet } from '@/context/WalletContext';
import { useToast } from '@/components/ui/Toast';
import { Card, Badge, Button, Skeleton } from '@/components/ui/primitives';
import ConnectCard from '@/components/ConnectCard';
import { DEV_ACCOUNTS } from '@/lib/devAccounts';
import { getApi, isValidDid, getDidKeys, hasCdd, formatUnits } from '@/lib/chain';

export default function TradePage() {
  const { connected, connecting, did, holdings, polyx, pair, refresh } = useWallet();
  const { push } = useToast();
  const [mounted, setMounted] = useState(false);
  const [recipientDid, setRecipientDid] = useState('');
  const [assetId, setAssetId] = useState('');
  const [amount, setAmount] = useState('');
  const [busy, setBusy] = useState(false);
  const [compliance, setCompliance] = useState<'idle' | 'checking' | 'ok' | 'no' | 'invalid'>('idle');

  useEffect(() => { setMounted(true); }, []);
  useEffect(() => { if (holdings.length > 0 && !assetId) setAssetId(holdings[0].assetId); }, [holdings, assetId]);

  // پیش‌نمایش زنده Compliance گیرنده
  useEffect(() => {
    const d = recipientDid.trim();
    if (!d) { setCompliance('idle'); return; }
    if (!isValidDid(d)) { setCompliance('invalid'); return; }
    let cancel = false;
    setCompliance('checking');
    (async () => {
      try {
        const api = await getApi();
        const ok = await hasCdd(api, d);
        if (!cancel) setCompliance(ok ? 'ok' : 'no');
      } catch { if (!cancel) setCompliance('invalid'); }
    })();
    return () => { cancel = true; };
  }, [recipientDid]);

  if (!mounted) return <div className="max-w-3xl mx-auto p-4"><Skeleton h={400} /></div>;
  if (connecting) return <div className="max-w-3xl mx-auto p-4"><Skeleton h={400} /></div>;
  if (!connected) return <div className="max-w-6xl mx-auto p-6"><ConnectCard /></div>;

  const holding = holdings.find(h => h.assetId === assetId);

  const send = async () => {
    const d = recipientDid.trim();
    const amt = parseFloat(amount);
    if (!isValidDid(d)) return push('error', 'DID گیرنده نامعتبر است');
    if (d.toLowerCase() === did?.toLowerCase()) return push('error', 'نمی‌توانید به خودتان انتقال دهید');
    if (!amt || amt <= 0) return push('error', 'مقدار نامعتبر است');
    if (!holding) return push('error', 'دارایی انتخاب نشده');
    const units = holding.divisible ? BigInt(Math.round(amt * 1e6)) : BigInt(Math.round(amt));
    if (units > holding.total) return push('error', 'موجودی کافی نیست');
    if (polyx === BigInt(0)) return push('error', 'POLYX برای کارمزد کافی نیست');
    if (compliance === 'no') return push('error', 'گیرنده CDD ندارد — Compliance رد می‌کند');

    setBusy(true);
    try {
      const api = await getApi();
      const rkeys = await getDidKeys(api, d);
      if (rkeys.length === 0) throw new Error('گیرنده کلیدی ندارد');
      const txx = api.tx.asset.transferAsset(assetId, rkeys[0], units, null);
      await new Promise<void>((resolve, reject) => {
        txx.signAndSend(pair, ({ status, dispatchError }: any) => {
          if (dispatchError) {
            let msg = dispatchError.toString();
            if (dispatchError.isModule) { try { const m = api.registry.findMetaError(dispatchError.asModule); msg = m.section + '.' + m.name; } catch {} }
            reject(new Error(msg));
          } else if (status.isInBlock || status.isFinalized) resolve();
          else if (status.isDropped || status.isInvalid) reject(new Error('تراکنش توسط شبکه رد شد'));
        });
      });
      push('success', '✅ انتقال با موفقیت ثبت شد و Compliance رعایت شد');
      setAmount(''); setRecipientDid('');
      await refresh();
    } catch (e: any) {
      push('error', 'خطا: ' + (e?.message || 'نامشخص'));
    } finally { setBusy(false); }
  };

  const complianceBadge = () => {
    switch (compliance) {
      case 'ok': return <Badge tone="success">✅ گیرنده احراز شده — مجاز</Badge>;
      case 'no': return <Badge tone="danger">❌ گیرنده CDD ندارد — رد می‌شود</Badge>;
      case 'checking': return <Badge tone="neutral">⏳ در حال بررسی...</Badge>;
      case 'invalid': return <Badge tone="danger">⚠️ DID نامعتبر</Badge>;
      default: return null;
    }
  };

  return (
    <div className="max-w-3xl mx-auto p-4 md:p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-extrabold">💱 انتقال سهام</h1>
        <p className="tb-muted text-sm mt-1">انتقال دارایی بین سرمایه‌گذاران با اجرای زنده قوانین Compliance</p>
      </div>

      <Card title="📤 فرم انتقال">
        <div className="space-y-4">
          <div>
            <div className="flex items-center justify-between mb-1">
              <label className="tb-label mb-0">DID گیرنده:</label>
              {complianceBadge()}
            </div>
            <input value={recipientDid} onChange={e => setRecipientDid(e.target.value)} dir="ltr" placeholder="0x..." className="tb-input font-mono text-xs" />
            <div className="flex flex-wrap gap-2 mt-2">
              {DEV_ACCOUNTS.filter(a => a.uri !== '//Alice').map(a => (
                <button key={a.uri} onClick={() => setRecipientDid('')} className="tb-btn tb-btn-ghost tb-btn-sm" title="به‌زودی: پر کردن خودکار DID">
                  {a.name.split(' ')[0]}
                </button>
              ))}
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="tb-label">دارایی:</label>
              <select value={assetId} onChange={e => setAssetId(e.target.value)} className="tb-input">
                {holdings.map(h => <option key={h.assetId} value={h.assetId}>{h.name}</option>)}
              </select>
              {holding && <div className="tb-muted text-xs mt-1">موجودی: <span className="tb-num text-[var(--green)]" dir="ltr">{formatUnits(holding.total, holding.divisible)}</span></div>}
            </div>
            <div>
              <div className="flex items-center justify-between">
                <label className="tb-label mb-0">مقدار:</label>
                {holding && <button onClick={() => setAmount(formatUnits(holding.total, holding.divisible))} className="tb-btn tb-btn-ghost tb-btn-sm">حداکثر</button>}
              </div>
              <input value={amount} onChange={e => setAmount(e.target.value)} type="number" dir="ltr" placeholder="0" className="tb-input tb-num mt-1" />
            </div>
          </div>

          <Button onClick={send} disabled={busy || compliance !== 'ok'} className="w-full">
            {busy ? '⏳ در حال ارسال...' : '✅ امضا و ارسال'}
          </Button>
        </div>
      </Card>
    </div>
  );
}
