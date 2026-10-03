'use client';
import { useState, useEffect } from 'react';
import Link from 'next/link';
import { useWallet } from '@/context/WalletContext';
import { Card, Button, Skeleton, EmptyState, CopyButton } from '@/components/ui/primitives';
import ConnectCard from '@/components/ConnectCard';
import { formatUnits } from '@/lib/chain';

export default function WalletPage() {
  const { connected, connecting, polyx, holdings, address } = useWallet();
  const [mounted, setMounted] = useState(false);
  useEffect(() => { setMounted(true); }, []);
  if (!mounted || connecting) return <div className="max-w-5xl mx-auto p-4"><Skeleton h={300} /></div>;
  if (!connected) return <div className="max-w-6xl mx-auto p-6"><ConnectCard /></div>;

  return (
    <div className="max-w-5xl mx-auto p-4 md:p-6 space-y-6">
      <h1 className="text-2xl font-extrabold">💼 کیف پول و دارایی‌ها</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <div className="tb-muted text-xs mb-1">POLYX (سوخت زنجیره)</div>
          <div className="text-2xl font-extrabold tb-num text-[var(--gold)]" dir="ltr">{formatUnits(polyx, true)}</div>
        </Card>
        <Card>
          <div className="tb-muted text-xs mb-1">آدرس کیف پول</div>
          <div className="flex items-center gap-2 font-mono text-xs" dir="ltr">{address}<CopyButton text={address || ''} /></div>
        </Card>
      </div>
      <Card title="📊 دارایی‌های سهام">
        {holdings.length === 0 ? <EmptyState icon="📭" title="هنوز دارایی ندارید" /> : (
          <div className="overflow-x-auto">
            <table className="tb-table">
              <thead><tr><th>دارایی</th><th>شناسه</th><th className="text-left">موجودی</th><th></th></tr></thead>
              <tbody>
                {holdings.map(h => (
                  <tr key={h.assetId}>
                    <td className="font-bold">{h.name}</td>
                    <td className="tb-muted font-mono text-xs" dir="ltr">{h.assetId.slice(0, 10)}...</td>
                    <td className="text-left tb-num font-bold text-[var(--green)]" dir="ltr">{formatUnits(h.total, h.divisible)}</td>
                    <td><Link href="/trade"><Button size="sm" variant="ghost">انتقال</Button></Link></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>
    </div>
  );
}
