'use client';

import { useState, useEffect } from 'react';
import { ApiPromise, WsProvider } from '@polkadot/api';
import Link from 'next/link';

export default function Home() {
  const [status, setStatus] = useState('قطع');
  const [chainInfo, setChainInfo] = useState<any>(null);
  const [error, setError] = useState('');
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const connectToChain = async () => {
    try {
      setError('');
      setStatus('در حال اتصال...');
      
      const wsProvider = new WsProvider('ws://127.0.0.1:9945');
      const api = await ApiPromise.create({ provider: wsProvider });
      
      const [chain, nodeName, nodeVersion] = await Promise.all([
        api.rpc.system.chain(),
        api.rpc.system.name(),
        api.rpc.system.version()
      ]);

      setChainInfo({
        chain: chain.toString(),
        nodeName: nodeName.toString(),
        nodeVersion: nodeVersion.toString()
      });
      
      setStatus('متصل ✅');
    } catch (err: any) {
      setError(err.message || 'خطا در اتصال');
      setStatus('قطع ❌');
    }
  };

  if (!mounted) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center" dir="rtl">
        <div className="max-w-2xl w-full bg-white rounded-2xl shadow-2xl p-8">
          <h1 className="text-4xl font-bold text-center text-gray-800">بورس تهران On-Chain</h1>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center p-4" dir="rtl">
      <div className="max-w-2xl w-full bg-white rounded-2xl shadow-2xl p-8">
        <h1 className="text-4xl font-bold text-center mb-2 text-gray-800">
          بورس تهران On-Chain
        </h1>
        <p className="text-center text-gray-600 mb-8">
          پلتفرم بورس مبتنی بر بلاکچین Polymesh
        </p>

        <div className="bg-gray-50 rounded-lg p-6 mb-6">
          <div className="flex items-center justify-between mb-4">
            <span className="text-gray-700 font-medium">وضعیت اتصال:</span>
            <span className={`px-4 py-2 rounded-full font-bold ${
              status === 'متصل ✅' ? 'bg-green-100 text-green-800' :
              status === 'در حال اتصال...' ? 'bg-yellow-100 text-yellow-800' :
              'bg-red-100 text-red-800'
            }`}>
              {status}
            </span>
          </div>

          {chainInfo && (
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600">نام شبکه:</span>
                <span className="font-mono text-gray-800">{chainInfo.chain}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">نوع نود:</span>
                <span className="font-mono text-gray-800">{chainInfo.nodeName}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">نسخه:</span>
                <span className="font-mono text-gray-800">{chainInfo.nodeVersion}</span>
              </div>
            </div>
          )}

          {error && (
            <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded text-red-700 text-sm">
              {error}
            </div>
          )}
        </div>

        <button
          onClick={connectToChain}
          disabled={status === 'در حال اتصال...'}
          className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-bold py-4 px-6 rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all transform hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none mb-4"
        >
          {status === 'متصل ✅' ? '🔄 اتصال مجدد' : '🔌 اتصال به شبکه بورس تهران'}
        </button>

        <div className="grid grid-cols-2 gap-3">
          <Link href="/onboarding">
            <button className="w-full bg-gradient-to-r from-green-600 to-emerald-600 text-white font-bold py-3 px-4 rounded-lg hover:from-green-700 hover:to-emerald-700 transition">
              📝 ثبت‌نام (سجام)
            </button>
          </Link>
          <Link href="/dashboard">
            <button className="w-full bg-gradient-to-r from-purple-600 to-pink-600 text-white font-bold py-3 px-4 rounded-lg hover:from-purple-700 hover:to-pink-700 transition">
              📊 داشبورد
            </button>
          </Link>
        </div>

        <div className="mt-8 text-center text-xs text-gray-500">
          <p>🔌 نود محلی روی پورت 9945</p>
        </div>
      </div>
    </div>
  );
}
