'use client';
import { useState, useEffect } from 'react';
import { useWallet } from '@/context/WalletContext';
import { Card, Badge, Skeleton, CopyButton } from '@/components/ui/primitives';
import ConnectCard from '@/components/ConnectCard';
import { getApi, getDidKeys } from '@/lib/chain';

export default function ProfilePage() {
  const { connected, connecting, did, address, cdd } = useWallet();
  const [keys, setKeys] = useState<string[]>([]);
  const [mounted, setMounted] = useState(false);
  useEffect(() => { setMounted(true); }, []);
  useEffect(() => { (async () => { if (did) { try { const api = await getApi(); setKeys(await getDidKeys(api, did)); } catch {} } })(); }, [did]);

  if (!mounted || connecting) return <div className="max-w-3xl mx-auto p-4"><Skeleton h={300} /></div>;
  if (!connected) return <div className="max-w-6xl mx-auto p-6"><ConnectCard /></div>;

  return (
    <div className="max-w-3xl mx-auto p-4 md:p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-extrabold">👤 پروفایل و هویت</h1>
        <Badge tone={cdd ? 'success' : 'danger'}>{cdd ? '✅ CDD تایید شده' : '❌ بدون CDD'}</Badge>
      </div>

      <Card title="🪪 هویت On-Chain (DID)">
        <div className="flex items-center gap-2 font-mono text-xs break-all" dir="ltr">{did}<CopyButton text={did || ''} /></div>
        <p className="tb-muted text-xs mt-3">شناسه یکتای شما روی بلاکچین Polymesh که پس از احراز هویت سجام صادر شده است.</p>
      </Card>

      <Card title="🔑 کلیدهای متصل">
        <div className="space-y-2">
          {keys.map((k, i) => (
            <div key={k} className="flex items-center justify-between gap-2 tb-card" style={{ padding: '0.6rem 0.8rem' }}>
              <span className="tb-muted text-xs shrink-0">{i === 0 ? '⭐ Primary' : 'Secondary'}</span>
              <span className="font-mono text-xs truncate" dir="ltr">{k}</span>
              <CopyButton text={k} />
            </div>
          ))}
        </div>
      </Card>

      <Card title="📇 آدرس کیف پول">
        <div className="flex items-center gap-2 font-mono text-xs" dir="ltr">{address}<CopyButton text={address || ''} /></div>
      </Card>
    </div>
  );
}
