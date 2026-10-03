'use client';

import { useState } from 'react';
import Link from 'next/link';

export default function DevAccessButton() {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="fixed bottom-4 left-4 z-40">
      {expanded && (
        <div className="absolute bottom-14 left-0 space-y-2 mb-2">
          <Link 
            href="/accounts" 
            className="block tb-btn tb-btn-ghost tb-btn-sm whitespace-nowrap bg-[var(--surface)] shadow-lg"
            onClick={() => setExpanded(false)}
          >
            📒 دفترچه اکانت‌ها
          </Link>
          <Link 
            href="/dev/accounts" 
            className="block tb-btn tb-btn-ghost tb-btn-sm whitespace-nowrap bg-[var(--surface)] shadow-lg"
            onClick={() => setExpanded(false)}
          >
            🔧 ابزارهای Dev
          </Link>
        </div>
      )}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-12 h-12 rounded-full bg-[var(--surface-2)] border border-[var(--border)] shadow-lg flex items-center justify-center text-xl hover:bg-[var(--border)] transition"
        title="ابزارهای توسعه‌دهنده"
      >
        🛠️
      </button>
    </div>
  );
}
