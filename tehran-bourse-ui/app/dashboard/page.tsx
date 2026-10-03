'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { useWallet } from '@/context/WalletContext';
import { Card, Badge, Skeleton, EmptyState, CopyButton, Button } from '@/components/ui/primitives';
import ConnectCard from '@/components/ConnectCard';
import { formatUnits } from '@/lib/chain';

export default function DashboardPage() {
  const { connected, connecting, did, address, polyx, holdings, cdd, label } = useWallet();
  const [mounted, setMounted] = useState(false);
  useEffect(() => { setMounted(true); }, []);

  if (!mounted) return <div className="max-w-6xl mx-auto p-4"><Skeleton h={300} /></div>;
  if (connecting) return <div className="max-w-6xl mx-auto p-4 space-y-4"><Skeleton h={80} /><Skeleton h={200} /></div>;
  if (!connected) return <div className="max-w-6xl mx-auto p-6"><ConnectCard /></div>;

  return (
    <div className="max-w-6xl mx-auto p-4 md:p-6 space-y-6">
      <div className="tb-card flex items-center justify-between" style={{ borderColor: 'var(--gold)' }}>
        <div>
          <div className="font-extrabold text-lg">👋 خوش آمدید، <span className="text-[var(--gold)]">{label || 'سرمایه‌گذار'}</span></div>
          <div className="tb-muted text-xs mt-1">حساب شما فعال و احراز شده است. آمادهٔ معامله هستید.</div>
        </div>
        <span className="text-3xl">🎉</span>
      </div>

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-extrabold">داشبورد سرمایه‌گذار</h1>
          <div className="flex items-center gap-2 mt-2 tb-muted text-xs">
            <span dir="ltr" className="font-mono">{did?.slice(0, 14)}...</span>
            <CopyButton text={did || ''} />
          </div>
        </div>
        <Badge tone={cdd ? 'success' : 'danger'}>{cdd ? '✅ احراز هویت (CDD) تایید شده' : '❌ بدون احراز هویت'}</Badge>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <div className="tb-muted text-xs mb-1">اعتبار POLYX (کارمزد زنجیره)</div>
          <div className="text-2xl font-extrabold tb-num text-[var(--gold)]" dir="ltr">{formatUnits(polyx, true)}</div>
        </Card>
        <Card>
          <div className="tb-muted text-xs mb-1">تعداد دارایی‌های سهام</div>
          <div className="text-2xl font-extrabold tb-num">{holdings.length}</div>
        </Card>
        <Card>
          <div className="tb-muted text-xs mb-1">کلیدهای متصل</div>
          <div className="text-2xl font-extrabold tb-num" dir="ltr">{address?.slice(0, 8)}...</div>
        </Card>
      </div>

      <Card title="📊 دارایی‌های شما" action={<Link href="/trade"><Button size="sm">انتقال سهام</Button></Link>}>
        {holdings.length === 0 ? (
          <EmptyState icon="📭" title="هنوز دارایی ندارید" desc="پس از دریافت سهام، این‌جا نمایش داده می‌شود." />
        ) : (
          <div className="overflow-x-auto">
            <table className="tb-table">
              <thead><tr><th>دارایی</th><th>شناسه</th><th className="text-left">موجودی</th></tr></thead>
              <tbody>
                {holdings.map(h => (
                  <tr key={h.assetId}>
                    <td className="font-bold">{h.name}</td>
                    <td className="tb-muted font-mono text-xs" dir="ltr">{h.assetId.slice(0, 10)}...</td>
                    <td className="text-left tb-num font-bold text-[var(--green)]" dir="ltr">{formatUnits(h.total, h.divisible)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Link href="/trade" className="tb-card hover:border-[var(--gold)] transition text-center">💱 انتقال سهام</Link>
        <Link href="/history" className="tb-card hover:border-[var(--gold)] transition text-center">📜 تاریخچه</Link>
        <Link href="/profile" className="tb-card hover:border-[var(--gold)] transition text-center">👤 پروفایل</Link>
      </div>
    </div>
  );
}
