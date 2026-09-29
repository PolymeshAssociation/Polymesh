'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { getApi, getDidForAddress, hasCdd, getHoldings, getPolyx, formatUnits } from '@/lib/chain';
import { DEV_ACCOUNTS } from '@/lib/devAccounts';

interface TokenInfo {
  name: string;
  total: bigint;
  divisible: boolean;
}

interface AccountInfo {
  name: string;
  uri: string;
  role: string;
  address: string;
  did: string | null;
  cdd: boolean;
  polyx: bigint;
  tokens: TokenInfo[];
}

const ROLE_LABEL: Record<string, string> = {
  issuer: '🏦 ناشر',
  investor: '💼 سرمایه‌گذار',
  nocdd: '🚫 بدون CDD',
};

const ROLE_COLOR: Record<string, string> = {
  issuer: 'bg-orange-100 text-orange-800 border-orange-300',
  investor: 'bg-green-100 text-green-800 border-green-300',
  nocdd: 'bg-red-100 text-red-800 border-red-300',
};

export default function AccountsPage() {
  const [mounted, setMounted] = useState(false);
  const [loading, setLoading] = useState(false);
  const [accounts, setAccounts] = useState<AccountInfo[]>([]);
  const [error, setError] = useState('');
  const [copied, setCopied] = useState('');

  const copy = async (text: string, id: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(id);
      setTimeout(() => setCopied(''), 2000);
    } catch {}
  };

  const loadAll = async () => {
    setLoading(true);
    setError('');
    try {
      await cryptoWaitReady();
      const api = await getApi();
      const keyring = new Keyring({ type: 'sr25519' });
      const out: AccountInfo[] = [];
      for (const a of DEV_ACCOUNTS) {
        const pair = keyring.addFromUri(a.uri);
        const did = await getDidForAddress(api, pair.address);
        let cdd = false;
        let tokens: TokenInfo[] = [];
        if (did) {
          cdd = await hasCdd(api, did);
          tokens = await getHoldings(api, did, [pair.address]);
        }
        const polyx = await getPolyx(api, [pair.address]);
        out.push({ name: a.name, uri: a.uri, role: a.role, address: pair.address, did, cdd, polyx, tokens });
      }
      setAccounts(out);
    } catch (e: any) {
      setError('خطا در خواندن اطلاعات: ' + (e?.message || 'نامشخص'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    setMounted(true);
    loadAll();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const copyAllDids = () => {
    const text = accounts.map(a => a.name + ': ' + (a.did || 'بدون DID')).join('\n');
    copy(text, 'all');
  };

  if (!mounted) return <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100" dir="rtl"></div>;

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 p-4" dir="rtl">
      <div className="max-w-3xl mx-auto py-8">
        <div className="flex items-center justify-between mb-6">
          <Link href="/" className="text-blue-600 hover:text-blue-800">← بازگشت</Link>
          <h1 className="text-3xl font-bold text-gray-800">📒 دفترچه اکانت‌های تستی</h1>
        </div>

        <div className="flex gap-3 mb-6">
          <button
            onClick={loadAll}
            disabled={loading}
            className="flex-1 bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-bold py-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 disabled:opacity-50 transition"
          >
            {loading ? '⏳ در حال خواندن...' : '🔄 بارگذاری مجدد'}
          </button>
          {accounts.length > 0 && (
            <button
              onClick={copyAllDids}
              className="flex-1 bg-gradient-to-r from-purple-600 to-pink-600 text-white font-bold py-3 rounded-lg hover:from-purple-700 hover:to-pink-700 transition"
            >
              {copied === 'all' ? '✅ کپی شد' : '📋 کپی همه DID ها'}
            </button>
          )}
        </div>

        {error && <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded text-red-700 text-sm">{error}</div>}

        <div className="space-y-4">
          {accounts.map((a) => (
            <div key={a.uri} className="bg-white rounded-2xl shadow-xl p-5">
              <div className="flex items-center justify-between mb-3">
                <h2 className="text-lg font-bold text-gray-800">{a.name}</h2>
                <span className={`px-3 py-1 rounded-full text-xs font-bold border ${ROLE_COLOR[a.role] || 'bg-gray-100 text-gray-700 border-gray-300'}`}>
                  {ROLE_LABEL[a.role] || a.role}
                </span>
              </div>

              <div className="space-y-2 text-sm">
                <div>
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-gray-600">DID:</span>
                    <button
                      onClick={() => a.did && copy(a.did, 'did-' + a.uri)}
                      disabled={!a.did}
                      className="text-xs bg-gray-100 border border-gray-300 rounded px-2 py-1 hover:bg-gray-200 transition disabled:opacity-40"
                    >
                      {copied === 'did-' + a.uri ? '✅ کپی شد' : '📋 کپی'}
                    </button>
                  </div>
                  <div className="font-mono text-xs text-gray-900 bg-gray-50 p-2 rounded break-all border border-gray-200" dir="ltr">
                    {a.did || '❌ بدون DID'}
                  </div>
                </div>

                <div>
                  <div className="text-gray-600 mb-1">آدرس کیف پول:</div>
                  <div className="flex items-center gap-2">
                    <div className="font-mono text-xs text-gray-900 bg-gray-50 p-2 rounded break-all border border-gray-200 flex-1" dir="ltr">
                      {a.address}
                    </div>
                    <button
                      onClick={() => copy(a.address, 'addr-' + a.uri)}
                      className="text-xs bg-gray-100 border border-gray-300 rounded px-2 py-1 hover:bg-gray-200 transition shrink-0"
                    >
                      {copied === 'addr-' + a.uri ? '✅' : '📋'}
                    </button>
                  </div>
                </div>

                <div className="flex items-center justify-between pt-2 border-t border-gray-100">
                  <span className="text-gray-600">احراز هویت (CDD):</span>
                  <span className={`px-3 py-1 rounded-full text-xs font-bold ${a.cdd ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}`}>
                    {a.cdd ? '✅ تایید شده' : '❌ بدون CDD'}
                  </span>
                </div>

                <div className="flex items-center justify-between">
                  <span className="text-gray-600">POLYX:</span>
                  <span className="font-bold text-gray-900" dir="ltr">{formatUnits(a.polyx, true)}</span>
                </div>

                {a.tokens.length > 0 && (
                  <div className="pt-2 border-t border-gray-100">
                    <div className="text-gray-600 mb-1">دارایی‌ها:</div>
                    {a.tokens.map((t) => (
                      <div key={t.name} className="flex items-center justify-between text-xs bg-gray-50 p-2 rounded mb-1">
                        <span className="text-gray-800">{t.name}</span>
                        <span className="font-bold text-gray-900" dir="ltr">{formatUnits(t.total, t.divisible)}</span>
                      </div>
                    ))}
                  </div>
                )}

                <div className="text-xs text-gray-400 pt-1" dir="ltr">URI: {a.uri}</div>
              </div>
            </div>
          ))}
        </div>

        {loading && accounts.length === 0 && (
          <div className="text-center text-gray-600 py-10">⏳ در حال خواندن اطلاعات از بلاکچین...</div>
        )}
      </div>
    </div>
  );
}
