'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { getApi, isValidDid, getDidKeys, hasCdd, getHoldings, getPolyx, formatUnits } from '@/lib/chain';

export default function DashboardPage() {
  const [mounted, setMounted] = useState(false);
  const [didInput, setDidInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [data, setData] = useState<any>(null);

  useEffect(() => {
    setMounted(true);
    const saved = localStorage.getItem('tb_did');
    if (saved) setDidInput(saved);
  }, []);

  const load = async () => {
    const did = didInput.trim();
    if (!isValidDid(did)) {
      setError('فرمت DID نامعتبر است. باید با 0x شروع شود و ۶ کاراکتر هگز باشد.');
      return;
    }
    setError('');
    setLoading(true);
    setData(null);
    try {
      const api = await getApi();
      const keys = await getDidKeys(api, did);
      if (keys.length === 0) {
        setError('هیچ کلیدی برای این DID پیدا نشد. آیا این هویت ثبت‌نام شده است؟');
        setLoading(false);
        return;
      }
      const cdd = await hasCdd(api, did);
      const polyx = await getPolyx(api, keys);
      const holdings = await getHoldings(api, did, keys);
      localStorage.setItem('tb_did', did);
      setData({ did, keys, cdd, polyx, holdings });
    } catch (e: any) {
      setError('خطا در خواندن اطلاعات: ' + (e?.message || 'نامشخص'));
    } finally {
      setLoading(false);
    }
  };

  if (!mounted) {
    return <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100" dir="rtl"></div>;
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 p-4" dir="rtl">
      <div className="max-w-3xl mx-auto py-8">
        <div className="flex items-center justify-between mb-6">
          <Link href="/" className="text-blue-600 hover:text-blue-800">← بازگشت</Link>
          <h1 className="text-3xl font-bold text-gray-800">داشبورد سرمایه‌گذار</h1>
        </div>

        <div className="bg-white rounded-2xl shadow-xl p-6 mb-6">
          <label className="block text-gray-700 mb-2 font-medium">شناسه هویت (DID) خود را وارد کنید:</label>
          <div className="flex gap-3">
            <input
              type="text"
              value={didInput}
              onChange={(e) => setDidInput(e.target.value)}
              placeholder="0x..."
              className="flex-grow px-4 py-3 border border-gray-300 rounded-lg font-mono text-sm focus:ring-2 focus:ring-blue-500"
              dir="ltr"
            />
            <button
              onClick={load}
              disabled={loading}
              className="bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-bold px-6 py-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 disabled:opacity-50"
            >
              {loading ? '⏳ در حال خواندن...' : '🔍 نمایش داشبورد'}
            </button>
          </div>
          {error && <div className="mt-3 p-3 bg-red-50 border border-red-200 rounded text-red-700 text-sm">{error}</div>}
        </div>

        {data && (
          <div className="space-y-6">
            <div className="bg-white rounded-2xl shadow-xl p-6">
              <h2 className="text-xl font-bold text-gray-800 mb-4">🪪 وضعیت هویت</h2>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between"><span className="text-gray-600">DID:</span><span className="font-mono text-xs break-all" dir="ltr">{data.did}</span></div>
                <div className="flex justify-between"><span className="text-gray-600">تعداد کلیدهای متصل:</span><span className="font-bold">{data.keys.length}</span></div>
                <div className="flex justify-between items-center">
                  <span className="text-gray-600">احراز هویت (CDD):</span>
                  <span className={`px-3 py-1 rounded-full text-xs font-bold ${data.cdd ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}`}>
                    {data.cdd ? '✅ تایید شده' : '❌ بدون CDD'}
                  </span>
                </div>
              </div>
            </div>

            <div className="bg-white rounded-2xl shadow-xl p-6">
              <h2 className="text-xl font-bold text-gray-800 mb-4">💰 موجودی POLYX</h2>
              <div className="text-3xl font-bold text-blue-600" dir="ltr">{formatUnits(data.polyx, true)}</div>
            </div>

            <div className="bg-white rounded-2xl shadow-xl p-6">
              <h2 className="text-xl font-bold text-gray-800 mb-4">📈 دارایی‌های شما</h2>
              {data.holdings.length === 0 ? (
                <p className="text-gray-500 text-sm">هیچ دارایی روی زنجیره ثبت نشده است.</p>
              ) : (
                <div className="space-y-3">
                  {data.holdings.map((h: any) => (
                    <div key={h.assetId} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                      <div>
                        <div className="font-bold text-gray-800">{h.name}</div>
                        <div className="font-mono text-xs text-gray-500" dir="ltr">{h.assetId}</div>
                      </div>
                      <div className="text-lg font-bold text-gray-800" dir="ltr">{formatUnits(h.total, h.divisible)}</div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="bg-white rounded-2xl shadow-xl p-6">
              <h2 className="text-xl font-bold text-gray-800 mb-4">🔑 کلیدهای این هویت</h2>
              <div className="space-y-2">
                {data.keys.map((k: string, i: number) => (
                  <div key={k} className="font-mono text-xs bg-gray-50 p-2 rounded break-all" dir="ltr">
                    {i === 0 ? '⭐ Primary: ' : '🔸 Secondary: '}{k}
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
