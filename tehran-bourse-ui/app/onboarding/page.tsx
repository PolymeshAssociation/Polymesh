'use client';

import { useState, useEffect } from 'react';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady, mnemonicGenerate } from '@polkadot/util-crypto';
import Link from 'next/link';

type Step = 'form' | 'wallet' | 'registering' | 'done' | 'error';

export default function OnboardingPage() {
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

  useEffect(() => {
    setMounted(true);
  }, []);

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
      setError('خطا در ساخت کیف پول: ' + (e.message || 'نامشخص'));
    }
  };

  const downloadMnemonic = () => {
    const blob = new Blob([
      `کلمات بازیابی کیف پول بورس تهران\n` +
      `====================================\n` +
      `آدرس: ${address}\n\n` +
      `کلمات بازیابی (به ترتیب):\n${mnemonic}\n\n` +
      `⚠️ این کلمات را در جای امن نگه دارید.\n` +
      `در صورت گم کردن، هیچ راهی برای بازیابی کیف پول شما وجود ندارد.`
    ], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `tehran-bourse-wallet-${address.slice(0, 8)}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    setMnemonicSaved(true);
  };

  const registerUser = async () => {
    if (!mnemonicSaved) {
      setError('ابتدا باید کلمات بازیابی را دانلود کنید!');
      return;
    }

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

      if (data.status !== 'success') {
        throw new Error(data.message || 'ثبت‌نام ناموفق بود');
      }

      setProgress(p => [...p, '✅ هویت در سجام تایید شد']);
      setProgress(p => [...p, '✅ DID در بلاکچین ثبت شد']);
      setProgress(p => [...p, '✅ گواهی CDD صادر شد']);
      
      setResult(data.data);
      setStep('done');
    } catch (e: any) {
      setError('خطا در ثبت‌نام: ' + (e.message || 'نامشخص'));
      setStep('error');
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

  if (!mounted) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center" dir="rtl">
        <div className="text-2xl text-gray-700">در حال بارگذاری...</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 p-4" dir="rtl">
      <div className="max-w-2xl mx-auto py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <Link href="/" className="text-blue-600 hover:text-blue-800 flex items-center gap-2">
            <span>←</span>
            <span>بازگشت</span>
          </Link>
          <h1 className="text-3xl font-bold text-gray-800">ثبت‌نام در بورس تهران</h1>
        </div>

        {/* Progress Indicator */}
        <div className="flex items-center justify-center mb-8 gap-2">
          {['فرم اطلاعات', 'ساخت کیف پول', 'ثبت‌نام', 'پایان'].map((label, i) => {
            const currentStepIndex = 
              step === 'form' ? 0 :
              step === 'wallet' ? 1 :
              step === 'registering' || step === 'error' ? 2 :
              3;
            const isCompleted = i < currentStepIndex;
            const isCurrent = i === currentStepIndex;
            
            return (
              <div key={i} className="flex items-center">
                <div className={`w-8 h-8 rounded-full flex items-center justify-center font-bold text-sm ${
                  isCompleted ? 'bg-green-500 text-white' :
                  isCurrent ? 'bg-blue-600 text-white' :
                  'bg-gray-300 text-gray-600'
                }`}>
                  {isCompleted ? '✓' : i + 1}
                </div>
                <span className={`mr-2 text-sm ${isCurrent ? 'text-blue-600 font-bold' : 'text-gray-600'}`}>
                  {label}
                </span>
                {i < 3 && <div className="w-8 h-0.5 bg-gray-300 mx-2"></div>}
              </div>
            );
          })}
        </div>

        {/* Main Card */}
        <div className="bg-white rounded-2xl shadow-2xl p-8">
          {/* STEP 1: Form */}
          {step === 'form' && (
            <div>
              <h2 className="text-2xl font-bold mb-6 text-gray-800">اطلاعات هویتی</h2>
              <p className="text-gray-600 mb-6">
                لطفاً کد ملی و کد سجام خود را وارد کنید.
              </p>
              
              <div className="space-y-4 mb-6">
                <div>
                  <label className="block text-gray-700 mb-2 font-medium">کد ملی</label>
                  <input
                    type="text"
                    value={nationalCode}
                    onChange={(e) => setNationalCode(e.target.value)}
                    placeholder="مثال: 0012345678"
                    maxLength={10}
                    className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono"
                  />
                </div>
                
                <div>
                  <label className="block text-gray-700 mb-2 font-medium">کد سجام</label>
                  <input
                    type="text"
                    value={sejamCode}
                    onChange={(e) => setSejamCode(e.target.value)}
                    placeholder="مثال: SJ-TEST-001"
                    className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono"
                  />
                </div>
              </div>

              <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6 text-sm text-blue-800">
                <strong>💡 راهنمایی:</strong> برای تست، می‌توانید از این مقادیر استفاده کنید:
                <ul className="mt-2 mr-4 list-disc">
                  <li>کد ملی: <code className="bg-white px-2 py-0.5 rounded">0012345678</code> با کد سجام: <code className="bg-white px-2 py-0.5 rounded">SJ-TEST-001</code></li>
                  <li>کد ملی: <code className="bg-white px-2 py-0.5 rounded">0098765432</code> با کد سجام: <code className="bg-white px-2 py-0.5 rounded">SJ-TEST-002</code></li>
                </ul>
              </div>

              <button
                onClick={generateWallet}
                disabled={!nationalCode || !sejamCode}
                className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-bold py-4 px-6 rounded-lg hover:from-blue-700 hover:to-indigo-700 transition disabled:opacity-50 disabled:cursor-not-allowed"
              >
                ساخت کیف پول و ادامه
              </button>

              {error && (
                <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded text-red-700 text-sm">
                  {error}
                </div>
              )}
            </div>
          )}

          {/* STEP 2: Wallet Generation */}
          {step === 'wallet' && (
            <div>
              <h2 className="text-2xl font-bold mb-4 text-gray-800">🔑 کیف پول شما ساخته شد</h2>
              <p className="text-gray-600 mb-6">
                برای حفظ امنیت، کلمات بازیابی را دانلود و در جای امن نگه دارید.
              </p>

              <div className="bg-gray-50 border border-gray-200 rounded-lg p-4 mb-4">
                <div className="text-xs text-gray-500 mb-1">آدرس کیف پول:</div>
                <div className="font-mono text-sm text-gray-800 break-all">{address}</div>
              </div>

              <div className="bg-yellow-50 border-2 border-yellow-300 rounded-lg p-4 mb-6">
                <div className="text-yellow-800 font-bold mb-2">⚠️ هشدار امنیتی بسیار مهم:</div>
                <p className="text-yellow-900 text-sm mb-3">
                  کلمات بازیابی، کلید دسترسی به دارایی‌های شما در بلاکچین هستند.
                  در صورت گم کردن آن‌ها، هیچ راهی برای بازیابی وجود ندارد.
                </p>
                <button
                  onClick={downloadMnemonic}
                  className="w-full bg-yellow-500 text-white font-bold py-3 px-4 rounded-lg hover:bg-yellow-600 transition"
                >
                  📥 دانلود کلمات بازیابی (اجباری)
                </button>
                {mnemonicSaved && (
                  <div className="mt-3 text-green-700 text-sm font-bold flex items-center gap-2">
                    <span>✅</span>
                    <span>کلمات بازیابی با موفقیت دانلود شد</span>
                  </div>
                )}
              </div>

              <div className="flex gap-3">
                <button
                  onClick={resetForm}
                  className="flex-1 bg-gray-200 text-gray-700 font-bold py-3 px-4 rounded-lg hover:bg-gray-300 transition"
                >
                  بازگشت
                </button>
                <button
                  onClick={registerUser}
                  disabled={!mnemonicSaved}
                  className="flex-2 bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-bold py-3 px-6 rounded-lg hover:from-blue-700 hover:to-indigo-700 transition disabled:opacity-50 disabled:cursor-not-allowed flex-grow"
                >
                  ثبت‌نام در بورس تهران
                </button>
              </div>

              {error && (
                <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded text-red-700 text-sm">
                  {error}
                </div>
              )}
            </div>
          )}

          {/* STEP 3: Registering */}
          {step === 'registering' && (
            <div>
              <h2 className="text-2xl font-bold mb-6 text-gray-800">⏳ در حال ثبت‌نام...</h2>
              <div className="space-y-3">
                {progress.map((msg, i) => (
                  <div key={i} className="flex items-center gap-3 p-3 bg-blue-50 rounded-lg">
                    <div className="animate-spin rounded-full h-5 w-5 border-2 border-blue-600 border-t-transparent"></div>
                    <span className="text-gray-800">{msg}</span>
                  </div>
                ))}
                <div className="flex items-center gap-3 p-3 bg-gray-50 rounded-lg">
                  <div className="animate-spin rounded-full h-5 w-5 border-2 border-gray-400 border-t-transparent"></div>
                  <span className="text-gray-600">در حال تایید در بلاکچین...</span>
                </div>
              </div>
            </div>
          )}

          {/* STEP 4: Done */}
          {step === 'done' && result && (
            <div>
              <div className="text-center mb-6">
                <div className="text-6xl mb-4">🎉</div>
                <h2 className="text-2xl font-bold text-gray-800 mb-2">ثبت‌نام با موفقیت انجام شد!</h2>
                <p className="text-gray-600">
                  خوش آمدید، <span className="font-bold text-blue-600">{result.fullName}</span>
                </p>
              </div>

              <div className="bg-green-50 border border-green-200 rounded-lg p-4 mb-4">
                <div className="text-green-800 font-bold mb-3">✅ وضعیت حساب شما:</div>
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-700">DID:</span>
                    <span className="font-mono text-gray-800 text-xs">{result.did?.slice(0, 20)}...</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-700">وضعیت CDD:</span>
                    <span className={`px-2 py-0.5 rounded-full text-xs font-bold ${
                      result.cddIssued ? 'bg-green-200 text-green-900' : 'bg-blue-200 text-blue-900'
                    }`}>
                      {result.cddIssued ? 'صادر شد ✅' : 'از قبل موجود بود'}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-700">مجاز به معامله:</span>
                    <span className="px-2 py-0.5 rounded-full text-xs font-bold bg-green-200 text-green-900">بله</span>
                  </div>
                </div>
              </div>

              <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6 text-sm text-blue-800">
                <strong>📋 مراحل بعدی:</strong>
                <ul className="mt-2 mr-4 list-disc">
                  <li>وارد داشبورد شوید و موجودی POLYX خود را شارژ کنید</li>
                  <li>به بازار بروید و سهام بخرید</li>
                  <li>پروفایل خود را مشاهده کنید</li>
                </ul>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={resetForm}
                  className="flex-1 bg-gray-200 text-gray-700 font-bold py-3 px-4 rounded-lg hover:bg-gray-300 transition"
                >
                  ثبت‌نام کاربر جدید
                </button>
                <Link href="/dashboard" className="flex-2 flex-grow">
                  <button className="w-full bg-gradient-to-r from-green-600 to-emerald-600 text-white font-bold py-3 px-6 rounded-lg hover:from-green-700 hover:to-emerald-700 transition">
                    ورود به داشبورد
                  </button>
                </Link>
              </div>
            </div>
          )}

          {/* Error State */}
          {step === 'error' && (
            <div>
              <div className="text-center mb-6">
                <div className="text-6xl mb-4">❌</div>
                <h2 className="text-2xl font-bold text-gray-800 mb-2">مشکلی پیش آمد</h2>
              </div>

              <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
                <div className="text-red-800 font-bold mb-2">جزئیات خطا:</div>
                <div className="text-red-700 text-sm font-mono">{error}</div>
              </div>

              <div className="space-y-3 mb-6">
                {progress.map((msg, i) => (
                  <div key={i} className="flex items-center gap-3 p-3 bg-gray-50 rounded-lg">
                    <span className="text-gray-800">{msg}</span>
                  </div>
                ))}
              </div>

              <div className="flex gap-3">
                <button
                  onClick={resetForm}
                  className="flex-1 bg-gray-200 text-gray-700 font-bold py-3 px-4 rounded-lg hover:bg-gray-300 transition"
                >
                  شروع مجدد
                </button>
                <button
                  onClick={registerUser}
                  className="flex-2 flex-grow bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-bold py-3 px-6 rounded-lg hover:from-blue-700 hover:to-indigo-700 transition"
                >
                  تلاش مجدد
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
