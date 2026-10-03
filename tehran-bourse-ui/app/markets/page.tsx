'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { useWallet } from '@/context/WalletContext';
import { Card, Button, Skeleton, EmptyState, Badge } from '@/components/ui/primitives';
import ConnectCard from '@/components/ConnectCard';
import { getApi, formatUnits } from '@/lib/chain';

interface Asset {
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

export default function MarketsPage() {
  const { connected, connecting } = useWallet();
  const [mounted, setMounted] = useState(false);
  const [loading, setLoading] = useState(false);
  const [assets, setAssets] = useState<Asset[]>([]);
  const [error, setError] = useState('');

  useEffect(() => { setMounted(true); }, []);

  const loadAssets = async () => {
    setLoading(true);
    setError('');
    try {
      const api = await getApi();
      const nameEntries = await api.query.asset.assetNames.entries();
      const list: Asset[] = [];

      for (const [key, val] of nameEntries) {
        const assetId = key.args[0].toString();
        const name = hexToString(val.toString());

        let divisible = false;
        try {
          const d: any = await api.query.asset.assets(assetId);
          divisible = String(d?.divisible) === 'true';
        } catch {}

        // کل عرضه = جمع موجودی همه دارندگان؛ تعداد دارندگان = تعداد ورودی‌ها
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
    } catch (e: any) {
      setError('خطا در بارگذاری دارایی‌ها: ' + (e.message || 'نامشخص'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (connected) loadAssets();
  }, [connected]);

  if (!mounted || connecting) return <div className="max-w-6xl mx-auto p-4"><Skeleton h={400} /></div>;
  if (!connected) return <div className="max-w-6xl mx-auto p-6"><ConnectCard /></div>;

  return (
    <div className="max-w-6xl mx-auto p-4 md:p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-extrabold">📈 بازارها</h1>
          <p className="tb-muted text-sm mt-1">لیست دارایی‌های موجود در بورس تهران</p>
        </div>
        <button onClick={loadAssets} disabled={loading} className="tb-btn tb-btn-ghost tb-btn-sm">
          {loading ? '⏳ در حال بارگذاری...' : '🔄 بارگذاری مجدد'}
        </button>
      </div>

      <Card>
        {error && <div className="mb-4 p-3 rounded bg-[var(--red)] bg-opacity-10 text-[var(--red)] text-sm">{error}</div>}
        {loading ? (
          <div className="space-y-2">
            {[...Array(3)].map((_, i) => <Skeleton key={i} h={60} />)}
          </div>
        ) : assets.length === 0 ? (
          <EmptyState icon="📭" title="هنوز دارایی‌ای ثبت نشده" />
        ) : (
          <div className="overflow-x-auto">
            <table className="tb-table">
              <thead>
                <tr>
                  <th>نام دارایی</th>
                  <th>شناسه</th>
                  <th className="text-left">کل عرضه</th>
                  <th className="text-left">تعداد دارندگان</th>
                  <th className="text-left">قیمت</th>
                  <th className="text-left">تغییر ۲۴ ساعته</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {assets.map((asset) => (
                  <tr key={asset.assetId}>
                    <td className="font-bold">{asset.name}</td>
                    <td className="tb-muted font-mono text-xs" dir="ltr">{asset.assetId.slice(0, 10)}...</td>
                    <td className="text-left tb-num font-bold text-[var(--gold)]" dir="ltr">{formatUnits(asset.totalSupply, asset.divisible)}</td>
                    <td className="text-left tb-num">{asset.holderCount}</td>
                    <td className="text-left tb-muted">—</td>
                    <td className="text-left tb-muted">—</td>
                    <td>
                      <Link href="/trade">
                        <Button size="sm" variant="ghost">معامله</Button>
                      </Link>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      <Card>
        <div className="text-center py-4">
          <Badge tone="gold">🔜 به‌زودی</Badge>
          <p className="tb-muted text-sm mt-2">نمودار قیمت، Order Book و موتور تطبیق سفارش در فاز بعدی اضافه می‌شوند.</p>
        </div>
      </Card>
    </div>
  );
}
