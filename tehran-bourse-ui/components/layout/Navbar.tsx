'use client';
import Link from 'next/link';
import { useState } from 'react';
import { useWallet } from '@/context/WalletContext';
import { formatUnits } from '@/lib/chain';

const LINKS = [
  { href: '/dashboard', label: 'داشبورد' },
  { href: '/trade', label: 'معامله' },
  { href: '/wallet', label: 'کیف پول' },
  { href: '/profile', label: 'پروفایل' },
  { href: '/markets', label: 'بازارها' },
  { href: '/history', label: 'تاریخچه' },
  { href: '/issuer', label: '🏦 ناشر', adminOnly: true },
];

export default function Navbar() {
  const { connected, polyx, disconnect, address } = useWallet();
  const ADMIN_ADDRESS = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
  const isAdmin = address === ADMIN_ADDRESS;
  const [open, setOpen] = useState(false);

  return (
    <header className="tb-nav">
      <div className="max-w-6xl mx-auto px-4 h-16 flex items-center justify-between">
        <Link href="/" onClick={() => setOpen(false)} className="flex items-center gap-2">
          <span className="tb-logo">ت</span>
          <span className="font-extrabold text-lg">بورس <span className="text-[var(--gold)]">تهران</span></span>
        </Link>

        <nav className="hidden md:flex items-center gap-1">
          {LINKS.filter(l => !l.adminOnly || isAdmin).map(l => <Link key={l.href} href={l.href} className="tb-navlink">{l.label}</Link>)}
        </nav>

        <div className="flex items-center gap-2">
          {connected && <span className="tb-chip tb-num hidden sm:inline" dir="ltr">POLYX {formatUnits(polyx, true)}</span>}
          <span className="hidden sm:inline">
            {connected
              ? <button onClick={disconnect} className="tb-btn tb-btn-ghost tb-btn-sm">خروج</button>
              : <Link href="/dashboard" className="tb-btn tb-btn-gold tb-btn-sm">اتصال</Link>}
          </span>
          <span className="md:hidden">
            <button className="tb-btn tb-btn-ghost tb-btn-sm" onClick={() => setOpen(o => !o)}>{open ? '✕' : '☰'}</button>
          </span>
        </div>
      </div>

      {open && (
        <nav className="md:hidden border-t border-[var(--border)] bg-[var(--surface)] px-4 py-3 space-y-1">
          {LINKS.filter(l => !l.adminOnly || isAdmin).map(l => <Link key={l.href} href={l.href} onClick={() => setOpen(false)} className="block tb-navlink">{l.label}</Link>)}
          <div className="pt-2 mt-2 border-t border-[var(--border)] flex items-center justify-between">
            {connected ? (
              <>
                <span className="tb-chip tb-num" dir="ltr">POLYX {formatUnits(polyx, true)}</span>
                <button onClick={() => { disconnect(); setOpen(false); }} className="tb-btn tb-btn-ghost tb-btn-sm">خروج</button>
              </>
            ) : (
              <Link href="/dashboard" onClick={() => setOpen(false)} className="tb-btn tb-btn-gold tb-btn-sm">اتصال کیف پول</Link>
            )}
          </div>
        </nav>
      )}
    </header>
  );
}
