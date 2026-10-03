export interface DevAccount {
  name: string;
  uri: string;
  role: 'issuer' | 'investor' | 'nocdd';
}

export const DEV_ACCOUNTS: DevAccount[] = [
  { name: 'آلیس (ناشر / Issuer)', uri: '//Alice', role: 'issuer' },
  { name: 'رضا (سرمایه‌گذار ۱)', uri: '//TehranInv//1', role: 'investor' },
  { name: 'سارا (سرمایه‌گذار ۲)', uri: '//TehranInv//2', role: 'investor' },
  { name: 'کاربر بدون CDD (تست رد)', uri: '//TehranInv//3', role: 'nocdd' },
];
