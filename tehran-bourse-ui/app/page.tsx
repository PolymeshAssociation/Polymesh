import Link from 'next/link';

export default function Home() {
  return (
    <div className="max-w-6xl mx-auto px-4 py-16">
      <div className="text-center space-y-6">
        <span className="tb-badge tb-badge-gold">🇮🇷 بورس تهران · On-Chain</span>
        <h1 className="text-4xl md:text-5xl font-extrabold leading-tight">
          سهام شما، <span className="text-[var(--gold)]">روی بلاکچین</span>
        </h1>
        <p className="tb-muted max-w-xl mx-auto">
          بورس سهام مبتنی بر Polymesh با احراز هویت سجام و قوانین Compliance خودکار. فقط سرمایه‌گذاران احراز‌شده می‌توانند معامله کنند.
        </p>
        <div className="flex justify-center gap-3">
          <Link href="/dashboard" className="tb-btn tb-btn-gold">شروع کنید</Link>
          <Link href="/onboarding" className="tb-btn tb-btn-ghost">ثبت‌نام با سجام</Link>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-14">
        <div className="tb-card text-center"><div className="text-3xl mb-2">🪪</div><div className="font-bold mb-1">احراز هویت سجام</div><div className="tb-muted text-xs">هر سرمایه‌گذار یک DID با گواهی CDD دارد</div></div>
        <div className="tb-card text-center"><div className="text-3xl mb-2">⚖️</div><div className="font-bold mb-1">Compliance خودکار</div><div className="tb-muted text-xs">انتقال به فرد بدون احراز هویت توسط زنجیره رد می‌شود</div></div>
        <div className="tb-card text-center"><div className="text-3xl mb-2">🔍</div><div className="font-bold mb-1">شفافیت On-Chain</div><div className="tb-muted text-xs">مالکیت سهام به‌صورت عمومی قابل تأیید است</div></div>
      </div>
    </div>
  );
}
