import { ApiPromise, WsProvider } from '@polkadot/api';

export const WS_URL = 'ws://127.0.0.1:9945';

let apiPromise: Promise<any> | null = null;

export function getApi(): Promise<any> {
  if (!apiPromise) {
    apiPromise = ApiPromise.create({ provider: new WsProvider(WS_URL) });
  }
  return apiPromise;
}

export function isValidDid(did: string): boolean {
  return /^0x[0-9a-fA-F]{64}$/.test(did);
}

export function hexToString(hex: string): string {
  let h = hex.startsWith('0x') ? hex.slice(2) : hex;
  if (h.length % 2 !== 0) h = '0' + h;
  const bytes = new Uint8Array(h.length / 2);
  for (let i = 0; i < bytes.length; i++) bytes[i] = parseInt(h.substr(i * 2, 2), 16);
  try { return new TextDecoder().decode(bytes); } catch { return hex; }
}

function dedupe(arr: string[]): string[] { return Array.from(new Set(arr)); }

export async function getDidKeys(api: any, did: string): Promise<string[]> {
  const keys: string[] = [];
  try {
    const recs: any = await api.rpc.identity.getDidRecords(did);
    let pk: any = recs.primaryKey ?? recs.primary_key ?? null;
    if (pk && pk.isSome !== undefined) pk = pk.isSome ? pk.unwrap() : null;
    if (pk) keys.push(pk.toString());
    const secs: any[] = recs.secondaryKeys ?? recs.secondary_keys ?? [];
    for (const s of secs) {
      const signer = s.signer ?? s.key ?? s;
      let acc: any = null;
      if (signer && typeof signer.type === 'string') {
        if (signer.type === 'Account') acc = signer.value;
      } else if (signer) {
        acc = signer.account ?? signer.Account ?? signer;
      }
      if (acc) keys.push(acc.toString());
    }
    if (keys.length) return dedupe(keys);
  } catch {}
  try {
    const rec = await api.query.identity.didRecords(did);
    if (rec.isSome) {
      const v: any = rec.unwrap();
      const pk2 = v.primaryKey ?? v.primary_key ?? null;
      if (pk2) keys.push(pk2.toString());
    }
    const entries = await api.query.identity.didKeys.entries(did);
    for (const [k] of entries as any) keys.push(k.args[1].toString());
  } catch {}
  return dedupe(keys);
}

export async function hasCdd(api: any, did: string): Promise<boolean> {
  try {
    const entries = await api.query.identity.claims.entries();
    return entries.some(([key, val]: any) => {
      const first = key.args[0] || {}; const s = ((first.target?.toString() || '') + (first.claimType?.toString() || '') + (val?.toString() || '')).toLowerCase();
      return (first.target?.toString() || '').toLowerCase() === did.toLowerCase() && s.includes('customerduediligence');
    });
  } catch { return false; }
}

async function holderBalance(api: any, assetId: string, holder: any): Promise<bigint> {
  const a = api.query.asset;
  try {
    if (a.balanceOf) {
      const r = await a.balanceOf(assetId, holder);
      return BigInt(r.toString());
    }
  } catch {}
  try {
    if (holder.Account && a.accountAssetBalances) {
      const r = await a.accountAssetBalances(assetId, holder.Account);
      return BigInt(r.toString());
    }
    if (holder.Portfolio && a.portfolioAssetBalances) {
      const r = await a.portfolioAssetBalances(assetId, holder.Portfolio);
      return BigInt(r.toString());
    }
  } catch {}
  return BigInt(0);
}

export interface AssetHolding {
  assetId: string;
  name: string;
  divisible: boolean;
  total: bigint;
}

export async function getHoldings(api: any, did: string, keys: string[]): Promise<AssetHolding[]> {
  const out: AssetHolding[] = [];
  const names = await api.query.asset.assetNames.entries();
  for (const [key, val] of names as any) {
    const assetId = key.args[0].toHex();
    const name = hexToUtf8(val.toString());
    let divisible = false;
    try {
      const d = await api.query.asset.assets(assetId);
      divisible = String(d?.divisible) === 'true';
    } catch {}
    let total = BigInt(0);
    try {
      const b = await api.query.asset.balanceOf(assetId, did);
      total = BigInt(b.toString());
    } catch {}
    if (total > BigInt(0)) out.push({ assetId, name, divisible, total });
  }
  return out;
}

export async function getPolyx(api: any, keys: string[]): Promise<bigint> {
  let total = BigInt(0);
  for (const k of keys) {
    const info = await api.query.system.account(k);
    total = total + BigInt(info.data.free.toString());
  }
  return total;
}

export function formatUnits(v: bigint, divisible: boolean): string {
  if (!divisible) return v.toString();
  const s = v.toString().padStart(7, '0');
  return s.slice(0, -6) + '.' + s.slice(-6);
}

export async function getDidForAddress(api: any, address: string): Promise<string | null> {
  try {
    const opt = await api.query.identity.keyRecords(address);
    if (opt.isSome) {
      const v: any = opt.unwrap();
      if (typeof v.type === 'string') {
        if ((v.type === 'PrimaryKey' || v.type === 'SecondaryKey') && v.value) {
          return v.value.toString();
        }
        return null;
      }
      if (v.primaryKey) return v.primaryKey.toString();
      if (v.secondaryKey) return v.secondaryKey.toString();
    }
  } catch {}
  return null;
}

// ===== History scanner (reads from chain => survives restarts) =====
export interface TransferRow {
  block: number;
  hash: string;
  time: number;
  assetId: string;
  from: string;
  to: string;
  amount: bigint;
  direction: 'in' | 'out';
}

export async function scanTransfers(
  api: any,
  isMine: (s: string) => boolean,
  onProgress?: (scanned: number, total: number) => void
): Promise<TransferRow[]> {
  const head = await api.rpc.chain.getHeader();
  const total = head.number.toNumber();
  const out: TransferRow[] = [];
  const CHUNK = 40;

  for (let start = total; start >= 1; start -= CHUNK) {
    const end = Math.max(1, start - CHUNK + 1);
    const nums: number[] = [];
    for (let b = start; b >= end; b--) nums.push(b);

    const blocks = await Promise.all(nums.map(async (b) => {
      const h = await api.rpc.chain.getBlockHash(b);
      const [events, ts] = await Promise.all([
        api.query.system.events.at(h),
        api.query.timestamp.now.at(h).catch(() => null),
      ]);
      return { b, h, events, ts };
    }));

    for (const { b, h, events, ts } of blocks) {
      for (const rec of events as any) {
        const ev = rec.event;
        if (!ev || ev.section !== 'asset') continue;
        if (!/transfer/i.test(ev.method)) continue;
        const d = ev.data;
        if (!d || d.length < 4) continue;
        const from = d[1].toString();
        const to = d[2].toString();
        if (!isMine(from) && !isMine(to)) continue;
        out.push({
          block: b,
          hash: h.toString(),
          time: ts ? Number(ts.toString()) : 0,
          assetId: d[0].toString(),
          from, to,
          amount: BigInt(d[3].toString()),
          direction: isMine(from) ? 'out' : 'in',
        });
      }
    }
    onProgress?.(total - end + 1, total);
  }
  return out;
}

export function hexToUtf8(hex: string): string {
  const bytes = new Uint8Array((hex.slice(2).match(/.{1,2}/g) || []).map(b => parseInt(b, 16)));
  return new TextDecoder('utf-8').decode(bytes);
}
