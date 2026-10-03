'use client';
import { createContext, useContext, useState, useCallback, ReactNode } from 'react';
type T = { id: number; type: 'success' | 'error' | 'info'; msg: string };
const Ctx = createContext<{ push: (t: T['type'], m: string) => void } | null>(null);
export function useToast() { const c = useContext(Ctx); if (!c) throw new Error('useToast outside provider'); return c; }
export function ToastProvider({ children }: { children: ReactNode }) {
  const [list, setList] = useState<T[]>([]);
  const push = useCallback((type: T['type'], msg: string) => {
    const id = Date.now() + Math.random();
    setList(l => [...l, { id, type, msg }]);
    setTimeout(() => setList(l => l.filter(x => x.id !== id)), 4000);
  }, []);
  return (
    <Ctx.Provider value={{ push }}>
      {children}
      <div className="fixed bottom-4 left-4 z-50 space-y-2" dir="rtl">
        {list.map(t => <div key={t.id} className={`tb-toast tb-toast-${t.type}`}>{t.msg}</div>)}
      </div>
    </Ctx.Provider>
  );
}
