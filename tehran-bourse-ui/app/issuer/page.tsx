'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { useWallet } from '@/context/WalletContext';
import { useToast } from '@/components/ui/Toast';
import { Card, Button, Badge, Skeleton, EmptyState } from '@/components/ui/primitives';
import ConnectCard from '@/components/ConnectCard';
import { getApi, formatUnits } from '@/lib/chain';

const ADMIN_ADDRESS = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';

type Tab = 'overview' | 'create' | 'issue' | 'redeem';

interface AssetInfo {
  assetId: string;
  name: string;
  totalSupply: bigint;
  divisible: boolean;
  holderCount: number;
}

function hexToString(hex: string): string {
  const bytes = new Uint8Array((hex.slice(2).match(/.{1,2}/g) || []).map(b => parseInt(b, 16)));
  return new TextDecoder('utf-8').decode(bytes);
}

export default function IssuerPage() {
  const { connected, connecting, address, pair } = useWallet();
  const { push } = useToast();
  const [mounted, setMounted] = useState(false);
  const [activeTab, setActiveTab] = useState<Tab>('overview');
  const [assets, setAssets] = useState<AssetInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);

  const [assetName, setAssetName] = useState('');
  const [divisible, setDivisible] = useState(true);

  const [selectedAssetId, setSelectedAssetId] = useState('');
  const [amount, setAmount] = useState('');

  useEffect(() => { setMounted(true); }, []);

  const isAdmin = address === ADMIN_ADDRESS;

  const loadAssets = async () => {
    setLoading(true);
    try {
      const api = await getApi();
      const nameEntries = await api.query.asset.assetNames.entries();
      const list: AssetInfo[] = [];

      for (const [key, val] of nameEntries) {
        const assetId = key.args[0].toString();
        const name = hexToString(val.toString());

        let divisible = false;
        try {
          const d: any = await api.query.asset.assets(assetId);
          divisible = String(d?.divisible) === 'true';
        } catch {}

        let totalSupply = BigInt(0);
        let holderCount = 0;
        try {
          const bals = await api.query.asset.balanceOf.entries(assetId);
          holderCount = bals.length;
          for (const [, v] of bals) totalSupply += BigInt(v.toString().replace(/,/g, ''));
        } catch {}

        list.push({ assetId, name, totalSupply, divisible, holderCount });
      }

      setAssets(list);
      if (list.length > 0 && !selectedAssetId) setSelectedAssetId(list[0].assetId);
    } catch (e: any) {
      push('error', 'خطا در بارگذاری دارایی‌ها: ' + (e.message || 'نامشخص'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (connected && isAdmin) loadAssets();
  }, [connected, isAdmin]);

  const sendTx = async (txBuilder: () => any, label: string) => {
    if (!pair) { push('error', 'کیف پول متصل نیست'); return; }
    setBusy(true);
    try {
      const api = await getApi();
      const tx = txBuilder();
      await new Promise<void>((resolve, reject) => {
        tx.signAndSend(pair, ({ status, dispatchError }: any) => {
          if (dispatchError) {
            let msg = dispatchError.toString();
            if (dispatchError.isModule) {
              try {
                const m = api.registry.findMetaError(dispatchError.asModule);
                msg = m.section + '.' + m.name;
              } catch {}
            }
            reject(new Error(msg));
          } else if (status.isInBlock || status.isFinalized) {
            resolve();
          }
        });
      });
      push('success', label + ' با موفقیت انجام شد');
      await loadAssets();
    } catch (e: any) {
      push('error', 'خطا: ' + (e.message || 'نامشخص'));
    } finally {
      setBusy(false);
    }
  };

  const createAsset = async () => {
    if (!assetName.trim()) { push('error', 'نام دارایی لازم است'); return; }
    await sendTx(() => {
      return api.tx.asset.createAsset(assetName, divisible, 'EquityCommon', [], null);
    }, 'ساخت دارایی');
    setAssetName('');
    setActiveTab('overview');
  };

  const issueTokens = async () => {
    if (!selectedAssetId || !amount) { push('error', 'دارایی و مقدار لازم است'); return; }
    const asset = assets.find(a => a.assetId === selectedAssetId);
    if (!asset) return;
    const amt = parseFloat(amount);
    if (!amt || amt <= 0) { push('error', 'مقدار نامعتبر'); return; }
    const units = asset.divisible ? BigInt(Math.round(amt * 1e6)) : BigInt(Math.round(amt));
    await sendTx(() => {
      return api.tx.asset.issue(selectedAssetId, units, null);
    }, 'Issue');
    setAmount('');
  };

  const redeemTokens = async () => {
    if (!selectedAssetId || !amount) { push('error', 'دارایی و مقدار لازم است'); return; }
    const asset = assets.find(a => a.assetId === selectedAssetId);
    if (!asset) return;
    const amt = parseFloat(amount);
    if (!amt || amt <= 0) { push('error', 'مقدار نامعتبر'); return; }
    const units = asset.divisible ? BigInt(Math.round(amt * 1e6)) : BigInt(Math.round(amt));
    if (units > asset.totalSupply) { push('error', 'مقدار بیشتر از کل عرضه است'); return; }
    await sendTx(() => {
      return api.tx.asset.redeem(selectedAssetId, units, null);
    }, 'Redeem');
    setAmount('');
  };

  if (!mounted || connecting) return <div className="max-w-5xl mx-auto p-4"><Skeleton h={400} /></div>;
  if (!connected) return <div className="max-w-6xl mx-auto p-6"><ConnectCard /></div>;
  if (!isAdmin) {
    return (
      <div className="max-w-3xl mx-auto p-6">
        <Card>
          <div className="text-center py-10">
            <div className="text-6xl mb-4">🔒</div>
            <h2 className="text-2xl font-extrabold mb-2">دسترسی محدود</h2>
            <p className="tb-muted">این صفحه فقط برای ادمین/ناشر مرکزی در دسترس است.</p>
            <Link href="/dashboard" className="inline-block mt-6"><Button>بازگشت به داشبورد</Button></Link>
          </div>
        </Card>
      </div>
    );
  }

  // دسترسی محلی به api در builder ها
  const api = (window as any).__tb_api;
  const ensureApi = async () => { if (!api) (window as any).__tb_api = await getApi(); return (window as any).__tb_api; };

  const createAsset2 = async () => {
    if (!assetName.trim()) { push('error', 'نام دارایی لازم است'); return; }
    if (!pair) { push('error', 'کیف پول متصل نیست'); return; }
    setBusy(true);
    try {
      const a = await getApi();
      const tx = a.tx.asset.createAsset(assetName, divisible, 'EquityCommon', [], null);
      await new Promise<void>((resolve, reject) => {
        tx.signAndSend(pair, ({ status, dispatchError }: any) => {
          if (dispatchError) {
            let msg = dispatchError.toString();
            if (dispatchError.isModule) {
              try { const m = a.registry.findMetaError(dispatchError.asModule); msg = m.section + '.' + m.name; } catch {}
            }
            reject(new Error(msg));
          } else if (status.isInBlock || status.isFinalized) resolve();
        });
      });
      push('success', 'دارایی با موفقیت ساخته شد');
      setAssetName('');
      setActiveTab('overview');
      await loadAssets();
    } catch (e: any) {
      push('error', 'خطا در ساخت: ' + (e.message || 'نامشخص'));
    } finally { setBusy(false); }
  };

  const issueTokens2 = async () => {
    if (!selectedAssetId || !amount) { push('error', 'دارایی و مقدار لازم است'); return; }
    const asset = assets.find(a => a.assetId === selectedAssetId);
    if (!asset) return;
    const amt = parseFloat(amount);
    if (!amt || amt <= 0) { push('error', 'مقدار نامعتبر'); return; }
    const units = asset.divisible ? BigInt(Math.round(amt * 1e6)) : BigInt(Math.round(amt));
    if (!pair) { push('error', 'کیف پول متصل نیست'); return; }
    setBusy(true);
    try {
      const a = await getApi();
      const tx = a.tx.asset.issue(selectedAssetId, units, null);
      await new Promise<void>((resolve, reject) => {
        tx.signAndSend(pair, ({ status, dispatchError }: any) => {
          if (dispatchError) {
            let msg = dispatchError.toString();
            if (dispatchError.isModule) {
              try { const m = a.registry.findMetaError(dispatchError.asModule); msg = m.section + '.' + m.name; } catch {}
            }
            reject(new Error(msg));
          } else if (status.isInBlock || status.isFinalized) resolve();
        });
      });
      push('success', 'Issue با موفقیت انجام شد');
      setAmount('');
      await loadAssets();
    } catch (e: any) {
      push('error', 'خطا در Issue: ' + (e.message || 'نامشخص'));
    } finally { setBusy(false); }
  };

  const redeemTokens2 = async () => {
    if (!selectedAssetId || !amount) { push('error', 'دارایی و مقدار لازم است'); return; }
    const asset = assets.find(a => a.assetId === selectedAssetId);
    if (!asset) return;
    const amt = parseFloat(amount);
    if (!amt || amt <= 0) { push('error', 'مقدار نامعتبر'); return; }
    const units = asset.divisible ? BigInt(Math.round(amt * 1e6)) : BigInt(Math.round(amt));
    if (units > asset.totalSupply) { push('error', 'مقدار بیشتر از کل عرضه است'); return; }
    if (!pair) { push('error', 'کیف پول متصل نیست'); return; }
    setBusy(true);
    try {
      const a = await getApi();
      const tx = a.tx.asset.redeem(selectedAssetId, units, null);
      await new Promise<void>((resolve, reject) => {
        tx.signAndSend(pair, ({ status, dispatchError }: any) => {
          if (dispatchError) {
            let msg = dispatchError.toString();
            if (dispatchError.isModule) {
              try { const m = a.registry.findMetaError(dispatchError.asModule); msg = m.section + '.' + m.name; } catch {}
            }
            reject(new Error(msg));
          } else if (status.isInBlock || status.isFinalized) resolve();
        });
      });
      push('success', 'Redeem با موفقیت انجام شد');
      setAmount('');
      await loadAssets();
    } catch (e: any) {
      push('error', 'خطا در Redeem: ' + (e.message || 'نامشخص'));
    } finally { setBusy(false); }
  };

  return (
    <div className="max-w-6xl mx-auto p-4 md:p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-extrabold">🏦 داشبورد ناشر</h1>
          <p className="tb-muted text-sm mt-1">مدیریت دارایی‌ها، Issue و Redeem</p>
        </div>
        <Badge tone="gold">👤 ادمین</Badge>
      </div>

      <div className="flex gap-2 border-b border-[var(--border)] overflow-x-auto">
        {[
          { key: 'overview', label: '📊 نمای کلی' },
          { key: 'create', label: '➕ ساخت دارایی' },
          { key: 'issue', label: '📈 Issue' },
          { key: 'redeem', label: '📉 Redeem' },
        ].map(t => (
          <button
            key={t.key}
            onClick={() => setActiveTab(t.key as Tab)}
            className={`px-4 py-2 font-bold text-sm transition whitespace-nowrap ${
              activeTab === t.key
                ? 'text-[var(--gold)] border-b-2 border-[var(--gold)]'
                : 'tb-muted hover:text-[var(--text)]'
            }`}
          >
            {t.label}
          </button>
        ))}
      </div>

      {activeTab === 'overview' && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="font-bold">دارایی‌های منتشرشده</h2>
            <button onClick={loadAssets} disabled={loading} className="tb-btn tb-btn-ghost tb-btn-sm">
              {loading ? '⏳' : '🔄'}
            </button>
          </div>
          {loading ? (
            <div className="space-y-2">{[...Array(3)].map((_, i) => <Skeleton key={i} h={60} />)}</div>
          ) : assets.length === 0 ? (
            <EmptyState icon="📭" title="هنوز دارایی‌ای منتشر نشده" desc="از تب «ساخت دارایی» شروع کنید." />
          ) : (
            <div className="overflow-x-auto">
              <table className="tb-table">
                <thead>
                  <tr><th>نام</th><th>شناسه</th><th className="text-left">کل عرضه</th><th className="text-left">دارندگان</th><th>نوع</th></tr>
                </thead>
                <tbody>
                  {assets.map(a => (
                    <tr key={a.assetId}>
                      <td className="font-bold">{a.name}</td>
                      <td className="tb-muted font-mono text-xs" dir="ltr">{a.assetId.slice(0, 10)}...</td>
                      <td className="text-left tb-num font-bold text-[var(--gold)]" dir="ltr">{formatUnits(a.totalSupply, a.divisible)}</td>
                      <td className="text-left tb-num">{a.holderCount}</td>
                      <td><Badge tone="neutral">{a.divisible ? 'قابل تقسیم' : 'غیرقابل تقسیم'}</Badge></td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </Card>
      )}

      {activeTab === 'create' && (
        <Card title="➕ ساخت دارایی جدید">
          <div className="space-y-4">
            <div>
              <label className="tb-label">نام دارایی:</label>
              <input value={assetName} onChange={e => setAssetName(e.target.value)} placeholder="مثال: Tehran Stock 2" className="tb-input" />
            </div>
            <div>
              <label className="tb-label">نوع:</label>
              <select value={divisible ? 'yes' : 'no'} onChange={e => setDivisible(e.target.value === 'yes')} className="tb-input">
                <option value="yes">قابل تقسیم (Fungible)</option>
                <option value="no">غیرقابل تقسیم (NFT-like)</option>
              </select>
            </div>
            <Button onClick={createAsset2} disabled={busy || !assetName.trim()} className="w-full">
              {busy ? '⏳ در حال ساخت...' : 'ساخت دارایی'}
            </Button>
          </div>
        </Card>
      )}

      {activeTab === 'issue' && (
        <Card title="📈 Issue (افزایش عرضه)">
          <div className="space-y-4">
            <div>
              <label className="tb-label">دارایی:</label>
              <select value={selectedAssetId} onChange={e => setSelectedAssetId(e.target.value)} className="tb-input">
                {assets.map(a => <option key={a.assetId} value={a.assetId}>{a.name} (عرضه فعلی: {formatUnits(a.totalSupply, a.divisible)})</option>)}
              </select>
            </div>
            <div>
              <label className="tb-label">مقدار:</label>
              <input value={amount} onChange={e => setAmount(e.target.value)} type="number" placeholder="0" dir="ltr" className="tb-input tb-num" />
              <p className="tb-muted text-xs mt-1">توکن‌های Issue شده به حساب ادمین (Alice) اضافه می‌شوند.</p>
            </div>
            <Button onClick={issueTokens2} disabled={busy || !selectedAssetId || !amount} className="w-full">
              {busy ? '⏳ در حال Issue...' : 'Issue'}
            </Button>
          </div>
        </Card>
      )}

      {activeTab === 'redeem' && (
        <Card title="📉 Redeem (کاهش عرضه)">
          <div className="space-y-4">
            <div>
              <label className="tb-label">دارایی:</label>
              <select value={selectedAssetId} onChange={e => setSelectedAssetId(e.target.value)} className="tb-input">
                {assets.map(a => <option key={a.assetId} value={a.assetId}>{a.name} (عرضه فعلی: {formatUnits(a.totalSupply, a.divisible)})</option>)}
              </select>
            </div>
            <div>
              <label className="tb-label">مقدار:</label>
              <input value={amount} onChange={e => setAmount(e.target.value)} type="number" placeholder="0" dir="ltr" className="tb-input tb-num" />
              <p className="tb-muted text-xs mt-1">توکن‌ها از حساب ادمین سوزانده (burn) می‌شوند.</p>
            </div>
            <Button variant="danger" onClick={redeemTokens2} disabled={busy || !selectedAssetId || !amount} className="w-full">
              {busy ? '⏳ در حال Redeem...' : 'Redeem'}
            </Button>
          </div>
        </Card>
      )}
    </div>
  );
}
