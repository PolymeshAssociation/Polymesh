import Link from 'next/link';

export default function Footer() {
  return (
    <footer className="border-t border-[var(--border)] mt-10">
      <div className="max-w-6xl mx-auto px-4 py-6 flex flex-col md:flex-row items-center justify-between gap-3 text-xs tb-muted">
        <span>بورس تهران — سهام On-Chain با احراز هویت سجام</span>
        <div className="flex items-center gap-3">
          <Link href="/dev/accounts" className="hover:text-[var(--gold)] transition">🔧 ابزار توسعه‌دهنده</Link>
          <span dir="ltr">Tehran Bourse Testnet · ws://127.0.0.1:9945</span>
        </div>
      </div>
    </footer>
  );
}
