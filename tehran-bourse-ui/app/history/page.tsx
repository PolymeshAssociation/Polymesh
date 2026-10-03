'use client';

import { useState, useEffect } from 'react';
import { useWallet } from '@/context/WalletContext';
import { Card, Badge, Skeleton, EmptyState, CopyButton } from '@/components/ui/primitives';
import ConnectCard from '@/components/ConnectCard';
import { getApi, formatUnits } from '@/lib/chain';

interface TxRecord {
  block: number; hash: string; type: 'in' | 'out';
  assetId: string; amount: string; from: string; to: string;
}

export default function HistoryPage() {
  const { connected, connecting, address, holdings } = useWallet();
  const [mounted, setMounted] = useState(false);
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [scanNote, setScanNote] = useState('');
  const [txs, setTxs] = useState<TxRecord[]>([]);
  const [error, setError] = useState('');

  useEffect(() => { setMounted(true); }, []);

  const loadHistory = async () => {
    if (!address) return;
    setLoading(true); setError(''); setTxs([]); setProgress(0); setScanNote('');
    try {
      const api = await getApi();
      const head = await api.rpc.chain.getHeader();
      const total = head.number.toNumber();
      const records: TxRecord[] = [];
      const CHUNK = 25;

      for (let start = total; start >= 1; start -= CHUNK) {
        const end = Math.max(1, start - CHUNK + 1);
        const hashPromises = [];
        for (let b = start; b >= end; b--) hashPromises.push(api.rpc.chain.getBlockHash(b));
        let hashes, eventSets;
        try {
          hashes = await Promise.all(hashPromises);
          eventSets = await Promise.all(hashes.map(h => api.query.system.events.at(h)));
        } catch {
          setScanNote(`اسکن تا بلاک #${start} انجام شد؛ state بلاک‌های قدیمی‌تر توسط نود نگهداری نمی‌شود (pruning).`);
          break;
        }

        hashes.forEach((hash, idx) => {
          const blockNum = start - idx;
          for (const rec of eventSets[idx]) {
            const { event } = rec;
            if (event.section === 'asset' && event.method === 'CreatedAssetTransfer') {
              const data: any = event.data.toHuman ? event.data.toHuman() : event.data;
              if (data.from === address || data.to === address) {
                records.push({
                  block: blockNum, hash: hash.toString(),
                  type: data.from === address ? 'out' : 'in',
                  assetId: data.assetId,
                  amount: String(data.amount).replace(/,/g, ''),
                  from: data.from, to: data.to,
                });
              }
            }
          }
        });
        setProgress(Math.round(((total - end + 1) / total) * 100));
      }

      setTxs(records.sort((a, b) => b.block - a.block));
    } catch (e: any) {
      setError('خطا در بارگذاری تاریخچه: ' + (e.message || 'نامشخص'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { if (connected && address) loadHistory(); }, [connected, address]);

  if (!mounted || connecting) return <div className="max-w-5xl mx-auto p-4"><Skeleton h={400} /></div>;
  if (!connected) return <div className="max-w-6xl mx-auto p-6"><ConnectCard /></div>;

  return (
    <div className="max-w-5xl mx-auto p-4 md:p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-extrabold">📜 تاریخچه تراکنش‌ها</h1>
        <button onClick={loadHistory} disabled={loading} className="tb-btn tb-btn-ghost tb-btn-sm">
          {loading ? '⏳ در حال اسکن...' : '🔄 بارگذاری مجدد'}
        </button>
      </div>

      {loading && (
        <div className="tb-card">
          <div className="flex justify-between text-xs tb-muted mb-2">
            <span>اسکن زنجیره...</span>
            <span className="tb-num" dir="ltr">{progress}%</span>
          </div>
          <div className="h-2 rounded bg-[var(--surface-2)] overflow-hidden">
            <div className="h-full bg-[var(--gold)] transition-all" style={{ width: progress + '%' }} />
          </div>
        </div>
      )}

      {scanNote && <div className="tb-alert tb-alert-gold">ℹ️ {scanNote}</div>}
      {error && <div className="tb-alert tb-alert-error">❌ {error}</div>}

      <Card>
        {!loading && txs.length === 0 ? (
          <EmptyState icon="📭" title="تراکنشی یافت نشد" desc={scanNote || 'کل زنجیرهٔ در دسترس اسکن شد.'} />
        ) : (
          <div className="overflow-x-auto">
            <table className="tb-table">
              <thead>
                <tr><th>نوع</th><th>دارایی</th><th>مقدار</th><th>طرف مقابل</th><th>بلاک</th><th>هش</th></tr>
              </thead>
              <tbody>
                {txs.map((tx, i) => {
                  const asset = holdings.find(h => h.assetId === tx.assetId);
                  const cp = tx.type === 'out' ? tx.to : tx.from;
                  return (
                    <tr key={i}>
                      <td><Badge tone={tx.type === 'in' ? 'success' : 'danger'}>{tx.type === 'in' ? '📥 ورودی' : '📤 خروجی'}</Badge></td>
                      <td className="font-bold">{asset?.name || tx.assetId.slice(0, 10) + '...'}</td>
                      <td className={`tb-num font-bold ${tx.type === 'in' ? 'text-[var(--green)]' : 'text-[var(--red)]'}`} dir="ltr">
                        {tx.type === 'in' ? '+' : '-'}{formatUnits(BigInt(tx.amount), asset?.divisible || false)}
                      </td>
                      <td className="font-mono text-xs" dir="ltr">{cp.slice(0, 10)}...</td>
                      <td className="tb-num" dir="ltr">#{tx.block}</td>
                      <td><div className="flex items-center gap-1 font-mono text-xs" dir="ltr">{tx.hash.slice(0, 8)}...<CopyButton text={tx.hash} /></div></td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </Card>
    </div>
  );
}
