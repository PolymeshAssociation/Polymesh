'use client';
import { useState } from 'react';
import { DEV_ACCOUNTS } from '@/lib/devAccounts';
import { useWallet } from '@/context/WalletContext';
import { useToast } from '@/components/ui/Toast';
import { Button, Card } from '@/components/ui/primitives';

export default function ConnectCard() {
  const { connectDev, connectMnemonic, connecting } = useWallet();
  const { push } = useToast();
  const [uri, setUri] = useState('');
  const [mn, setMn] = useState('');

  const doDev = async () => { try { await connectDev(uri); push('success', 'کیف پول متصل شد'); } catch (e: any) { push('error', e?.message || 'خطا در اتصال'); } };
  const doMn = async () => { try { await connectMnemonic(mn); push('success', 'کیف پول متصل شد'); } catch (e: any) { push('error', e?.message || 'خطا در اتصال'); } };

  return (
    <div className="max-w-md mx-auto">
      <Card title="🔐 اتصال کیف پول">
        <label className="tb-label">انتخاب اکانت تستی (بدون Mnemonic):</label>
        <select value={uri} onChange={e => setUri(e.target.value)} className="tb-input mb-3">
          <option value="">-- انتخاب کنید --</option>
          {DEV_ACCOUNTS.map(a => <option key={a.uri} value={a.uri}>{a.name}</option>)}
        </select>
        <Button onClick={doDev} disabled={!uri || connecting} className="w-full">
          {connecting ? '⏳ در حال اتصال...' : 'ورود با اکانت تستی'}
        </Button>
        <div className="text-center tb-muted text-xs my-4">— یا ورود دستی —</div>
        <label className="tb-label">کلمات بازیابی (Mnemonic):</label>
        <textarea value={mn} onChange={e => setMn(e.target.value)} rows={2} dir="ltr" className="tb-input mb-3 font-mono text-xs" placeholder="کلمه۱ کلمه۲ ..." />
        <Button variant="ghost" onClick={doMn} disabled={!mn.trim() || connecting} className="w-full">باز کردن کیف پول</Button>
      </Card>
    </div>
  );
}
