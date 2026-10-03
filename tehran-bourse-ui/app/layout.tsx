import type { Metadata } from 'next';
import './globals.css';
import { WalletProvider } from '@/context/WalletContext';
import { ToastProvider } from '@/components/ui/Toast';
import Navbar from '@/components/layout/Navbar';
import Footer from '@/components/layout/Footer';
import DevAccessButton from '@/components/DevAccessButton';

export const metadata: Metadata = {
  title: 'بورس تهران | سهام On-Chain',
  description: 'بورس سهام مبتنی بر بلاکچین با احراز هویت سجام و قوانین Compliance',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="fa" dir="rtl">
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link href="https://fonts.googleapis.com/css2?family=Vazirmatn:wght@300;400;500;700;800&display=swap" rel="stylesheet" />
      </head>
      <body>
        <ToastProvider>
          <WalletProvider>
            <Navbar />
            <main className="min-h-[calc(100vh-140px)]">{children}</main>
            <Footer />
            {process.env.NODE_ENV === 'development' && <DevAccessButton />}
          </WalletProvider>
        </ToastProvider>
      </body>
    </html>
  );
}
