'use client';
import { createContext, useContext, useEffect, useState, useCallback, ReactNode } from 'react';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { getApi, getDidKeys, getDidForAddress, hasCdd, getHoldings, getPolyx } from '@/lib/chain';
import { DEV_ACCOUNTS } from '@/lib/devAccounts';

interface WalletState {
  connected: boolean; connecting: boolean;
  pair: any; address: string | null; did: string | null; label: string | null;
  polyx: bigint; holdings: any[]; cdd: boolean;
  connectDev(uri: string): Promise<void>;
  connectMnemonic(m: string): Promise<void>;
  disconnect(): void; refresh(): Promise<void>;
}
const Ctx = createContext<WalletState | null>(null);
export function useWallet() { const c = useContext(Ctx); if (!c) throw new Error('useWallet outside provider'); return c; }

export function WalletProvider({ children }: { children: ReactNode }) {
  const [pair, setPair] = useState<any>(null);
  const [address, setAddress] = useState<string | null>(null);
  const [did, setDid] = useState<string | null>(null);
  const [label, setLabel] = useState<string | null>(null);
  const [polyx, setPolyx] = useState<bigint>(BigInt(0));
  const [holdings, setHoldings] = useState<any[]>([]);
  const [cdd, setCdd] = useState(false);
  const [connecting, setConnecting] = useState(false);

  const load = useCallback(async (p: any) => {
    const api = await getApi();
    const d = await getDidForAddress(api, p.address);
    if (!d) throw new Error('این کیف پول به هیچ DIDی متصل نیست');
    const keys = await getDidKeys(api, d);
    const hold = await getHoldings(api, d, keys);
    const px = await getPolyx(api, [p.address]);
    const c = await hasCdd(api, d);
    setPair(p); setAddress(p.address); setDid(d); setHoldings(hold); setPolyx(px); setCdd(c);
  }, []);

  const connectDev = useCallback(async (uri: string) => {
    setConnecting(true);
    try {
      await cryptoWaitReady();
      const kr = new Keyring({ type: 'sr25519' });
      const p = kr.addFromUri(uri);
      await load(p);
      const acc = DEV_ACCOUNTS.find(a => a.uri === uri);
      setLabel(acc ? acc.name : null);
      localStorage.setItem('tb_dev_uri', uri);
      localStorage.setItem('tb_dev_label', acc ? acc.name : '');
    } finally { setConnecting(false); }
  }, [load]);

  const connectMnemonic = useCallback(async (m: string) => {
    setConnecting(true);
    try {
      await cryptoWaitReady();
      const kr = new Keyring({ type: 'sr25519' });
      const p = kr.addFromUri(m.trim());
      await load(p);
      setLabel(null);
      localStorage.removeItem('tb_dev_label');
    } finally { setConnecting(false); }
  }, [load]);

  const disconnect = useCallback(() => {
    setPair(null); setAddress(null); setDid(null); setLabel(null);
    setHoldings([]); setPolyx(BigInt(0)); setCdd(false);
    localStorage.removeItem('tb_dev_uri'); localStorage.removeItem('tb_dev_label');
  }, []);

  const refresh = useCallback(async () => { if (pair) await load(pair); }, [pair, load]);

  useEffect(() => {
    const uri = localStorage.getItem('tb_dev_uri');
    if (uri) connectDev(uri).catch(() => localStorage.removeItem('tb_dev_uri'));
  }, [connectDev]);

  return <Ctx.Provider value={{ connected: !!pair, connecting, pair, address, did, label, polyx, holdings, cdd, connectDev, connectMnemonic, disconnect, refresh }}>{children}</Ctx.Provider>;
}
