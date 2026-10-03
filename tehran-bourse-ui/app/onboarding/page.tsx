'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady, mnemonicGenerate } from '@polkadot/util-crypto';
import { Card, Button, Badge, CopyButton } from '@/components/ui/primitives';
import { useToast } from '@/components/ui/Toast';

type Step = 'form' | 'wallet' | 'registering' | 'done' | 'error';

const STEPS = [
  { key: 'form', label: 'اطلاعات هویتی', icon: '🪪' },
  { key: 'wallet', label: 'ساخت کیف پول', icon: '🔑' },
  { key: 'registering', label: 'ثبت‌نام', icon: '⚙️' },
  { key: 'done', label: 'تکمیل', icon: '✅' },
];

function Stepper({ current }: { current: Step }) {
  const idx = STEPS.findIndex(s => s.key === current);
  return (
    <div className="flex items-center justify-between mb-8">
      {STEPS.map((s, i) => (
        <div key={s.key} className="flex items-center">
          <div className="flex flex-col items-center">
            <div className={`w-10 h-10 rounded-full flex items-center justify-center text-lg font-bold transition ${
              i < idx ? 'bg-[var(--green)] text-[var(--bg)]' :
              i === idx ? 'bg-[var(--gold)] text-[var(--bg)]' :
              'bg-[var(--surface-2)] text-[var(--text-muted)]'
            }`}>
              {i < idx ? '✓' : s.icon}
            </div>
            <div className={`text-xs mt-1 ${i <= idx ? 'text-[var(--text)]' : 'tb-muted'}`}>{s.label}</div>
          </div>
          {i < STEPS.length - 1 && <div className={`w-12 md:w-20 h-0.5 mx-2 ${i < idx ? 'bg-[var(--green)]' : 'bg-[var(--border)]'}`} />}
        </div>
      ))}
    </div>
  );
}

export default function OnboardingPage() {
  const { push } = useToast();
  const [mounted, setMounted] = useState(false);
  const [step, setStep] = useState<Step>('form');
  const [nationalCode, setNationalCode] = useState('');
  const [sejamCode, setSejamCode] = useState('');
  const [mnemonic, setMnemonic] = useState('');
  const [address, setAddress] = useState('');
  const [error, setError] = useState('');
  const [progress, setProgress] = useState<string[]>([]);
  const [result, setResult] = useState<any>(null);
  const [mnemonicSaved, setMnemonicSaved] = useState(false);

  useEffect(() => { setMounted(true); }, []);

  const generateWallet = async () => {
    try {
      setError('');
      await cryptoWaitReady();
      const keyring = new Keyring({ type: 'sr25519' });
      const newMnemonic = mnemonicGenerate();
      const pair = keyring.addFromUri(newMnemonic);
      setMnemonic(newMnemonic);
      setAddress(pair.address);
      setStep('wallet');
    } catch (e: any) {
      push('error', 'خطا در ساخت کیف پول: ' + (e.message || 'نامشخص'));
    }
  };

  const downloadMnemonic = () => {
    const blob = new Blob([
      'کلمات بازیابی کیف پول بورس تهران\n' +
      '====================================\n' +
      'آدرس: ' + address + '\n\n' +
      'کلمات بازیابی (به ترتیب):\n' + mnemonic + '\n\n' +
      '⚠️ این کلمات را در جای امن نگه دارید.\n' +
      'در صورت گم کردن، هیچ راهی برای بازیابی کیف پول شما وجود ندارد.'
    ], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'tehran-bourse-wallet-' + address.slice(0, 8) + '.txt';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    setMnemonicSaved(true);
  };

  const registerUser = async () => {
    if (!mnemonicSaved) { setError('ابتدا باید کلمات بازیابی را دانلود کنید!'); return; }
    try {
      setError('');
      setStep('registering');
      setProgress([]);
      setProgress(p => [...p, '🔍 استعلام از سامانه سجام...']);
      const response = await fetch('http://localhost:3002/api/brokerage/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ nationalCode, sejamCode, address }),
      });
      const data = await response.json();
      if (data.status !== 'success') throw new Error(data.message || 'ثبت‌نام ناموفق بود');
      setProgress(p => [...p, '✅ هویت در سجام تایید شد']);
      await new Promise(r => setTimeout(r, 400));
      setProgress(p => [...p, '✅ DID در بلاکچین ثبت شد']);
      await new Promise(r => setTimeout(r, 400));
      setProgress(p => [...p, '✅ گواهی CDD صادر شد']);
      setResult(data.data);
      setStep('done');
      push('success', 'ثبت‌نام با موفقیت انجام شد!');
    } catch (e: any) {
      setError('خطا در ثبت‌نام: ' + (e.message || 'نامشخص'));
      setStep('error');
      push('error', e.message || 'خطا در ثبت‌نام');
    }
  };

  const resetForm = () => {
    setStep('form');
    setNationalCode('');
    setSejamCode('');
    setMnemonic('');
    setAddress('');
    setError('');
    setProgress([]);
    setResult(null);
    setMnemonicSaved(false);
  };

  if (!mounted) return <div className="max-w-2xl mx-auto p-6 text-center">در حال بارگذاری...</div>;

  return (
    <div className="max-w-2xl mx-auto p-4 md:p-6">
      <div className="flex items-center justify-between mb-6">
        <Link href="/" className="tb-btn tb-btn-ghost tb-btn-sm">← بازگشت</Link>
        <h1 className="text-2xl font-extrabold">ثبت‌نام در بورس تهران</h1>
      </div>

      <Stepper current={step} />

      {step === 'form' && (
        <Card title="🪪 اطلاعات هویتی">
          <div className="space-y-4">
            <div>
              <label className="tb-label">کد ملی</label>
              <input type="text" value={nationalCode} onChange={e => setNationalCode(e.target.value)} placeholder="مثال: 0012345678" maxLength={10} className="tb-input font-mono" />
            </div>
            <div>
              <label className="tb-label">کد سجام</label>
              <input type="text" value={sejamCode} onChange={e => setSejamCode(e.target.value)} placeholder="مثال: SJ-TEST-001" className="tb-input font-mono" />
            </div>
            <Button onClick={generateWallet} disabled={!nationalCode || !sejamCode} className="w-full">
              ساخت کیف پول و ادامه
            </Button>
            {error && <div className="tb-alert tb-alert-error">{error}</div>}
          </div>
        </Card>
      )}

      {step === 'wallet' && (
        <Card title="🔑 کیف پول شما ساخته شد">
          <div className="space-y-4">
            <div>
              <div className="tb-label mb-1">آدرس کیف پول:</div>
              <div className="flex items-center gap-2 font-mono text-xs bg-[var(--surface-2)] p-3 rounded break-all" dir="ltr">
                {address}<CopyButton text={address} />
              </div>
            </div>
            <div className="tb-alert tb-alert-gold">
              <div className="font-bold text-[var(--gold)] mb-2">⚠️ هشدار امنیتی:</div>
              <p className="text-sm tb-muted mb-3">کلمات بازیابی را دانلود و در جای امن نگه دارید. در صورت گم کردن، هیچ راهی برای بازیابی کیف پول شما وجود ندارد.</p>
              <Button variant="gold" onClick={downloadMnemonic} className="w-full">
                📥 دانلود کلمات بازیابی (اجباری)
              </Button>
              {mnemonicSaved && <div className="mt-3 text-[var(--green)] text-sm font-bold text-center">✅ کلمات بازیابی دانلود شد</div>}
            </div>
            <div className="flex gap-3">
              <Button variant="ghost" onClick={resetForm} className="flex-1">بازگشت</Button>
              <Button onClick={registerUser} disabled={!mnemonicSaved} className="flex-grow">ثبت‌نام در بورس تهران</Button>
            </div>
          </div>
        </Card>
      )}

      {step === 'registering' && (
        <Card title="⏳ در حال ثبت‌نام...">
          <div className="space-y-3">
            {progress.map((msg, i) => (
              <div key={i} className="flex items-center gap-3 p-3 bg-[var(--surface-2)] rounded-lg">
                {i === progress.length - 1 ? (
                  <div className="animate-spin rounded-full h-5 w-5 border-2 border-[var(--gold)] border-t-transparent shrink-0"></div>
                ) : (
                  <span className="text-[var(--green)] text-lg">✓</span>
                )}
                <span className="text-sm">{msg}</span>
              </div>
            ))}
          </div>
        </Card>
      )}

      {step === 'done' && result && (
        <Card>
          <div className="text-center mb-6">
            <div className="text-6xl mb-4">🎉</div>
            <h2 className="text-2xl font-extrabold mb-2">ثبت‌نام با موفقیت انجام شد!</h2>
            <p className="tb-muted">خوش آمدید، <span className="font-bold text-[var(--gold)]">{result.fullName}</span></p>
          </div>
          <div className="tb-alert tb-alert-success space-y-3">
            <div className="font-bold text-[var(--green)]">✅ اطلاعات هویتی شما روی بلاکچین:</div>
            <div className="space-y-3 text-sm">
              <div>
                <div className="flex items-center justify-between mb-1">
                  <span className="tb-label mb-0">شناسه هویت (DID):</span>
                  <CopyButton text={result.did} />
                </div>
                <div className="font-mono text-xs bg-[var(--surface-2)] p-2 rounded break-all" dir="ltr">{result.did}</div>
              </div>
              <div>
                <div className="flex items-center justify-between mb-1">
                  <span className="tb-label mb-0">آدرس کیف پول:</span>
                  <CopyButton text={address} />
                </div>
                <div className="font-mono text-xs bg-[var(--surface-2)] p-2 rounded break-all" dir="ltr">{address}</div>
              </div>
              <div className="flex justify-between items-center pt-2 border-t border-[var(--border)]">
                <span className="tb-label mb-0">وضعیت CDD:</span>
                <Badge tone={result.cddIssued ? 'success' : 'gold'}>{result.cddIssued ? '✅ صادر شد' : 'ℹ️ از قبل موجود'}</Badge>
              </div>
              <div className="flex justify-between items-center">
                <span className="tb-label mb-0">مجاز به معامله:</span>
                <Badge tone="success">✅ بله</Badge>
              </div>
            </div>
          </div>
          <div className="flex gap-3 mt-6">
            <Button variant="ghost" onClick={resetForm} className="flex-1">ثبت‌نام کاربر جدید</Button>
            <Link href="/dashboard" className="flex-grow"><Button className="w-full">ورود به داشبورد</Button></Link>
          </div>
        </Card>
      )}

      {step === 'error' && (
        <Card>
          <div className="text-center mb-6">
            <div className="text-6xl mb-4">❌</div>
            <h2 className="text-2xl font-extrabold mb-2">مشکلی پیش آمد</h2>
          </div>
          <div className="tb-alert tb-alert-error">
            <div className="font-bold text-[var(--red)] mb-2">جزئیات خطا:</div>
            <div className="text-[var(--red)] text-sm font-mono">{error}</div>
          </div>
          <div className="flex gap-3 mt-6">
            <Button variant="ghost" onClick={resetForm} className="flex-1">شروع مجدد</Button>
            <Button onClick={registerUser} className="flex-grow">تلاش مجدد</Button>
          </div>
        </Card>
      )}
    </div>
  );
}
