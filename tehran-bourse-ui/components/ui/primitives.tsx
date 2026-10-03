'use client';
import { useState, ButtonHTMLAttributes, ReactNode } from 'react';

export function Button({ variant = 'gold', size = 'md', className = '', ...props }: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: 'gold' | 'ghost' | 'danger' | 'success'; size?: 'sm' | 'md' }) {
  return <button className={`tb-btn tb-btn-${variant} ${size === 'sm' ? 'tb-btn-sm' : ''} ${className}`} {...props} />;
}

export function Card({ title, action, children, className = '' }: { title?: string; action?: ReactNode; children: ReactNode; className?: string }) {
  return (
    <div className={`tb-card ${className}`}>
      {(title || action) && (
        <div className="flex items-center justify-between mb-4">
          {title && <h3 className="font-bold text-gray-100">{title}</h3>}
          {action}
        </div>
      )}
      {children}
    </div>
  );
}

export function Badge({ tone = 'neutral', children }: { tone?: 'success' | 'danger' | 'gold' | 'neutral'; children: ReactNode }) {
  return <span className={`tb-badge tb-badge-${tone}`}>{children}</span>;
}

export function Skeleton({ h = 20, className = '' }: { h?: number; className?: string }) {
  return <div className={`tb-skeleton ${className}`} style={{ height: h }} />;
}

export function EmptyState({ icon = '📭', title, desc }: { icon?: string; title: string; desc?: string }) {
  return (
    <div className="text-center py-10">
      <div className="text-4xl mb-3">{icon}</div>
      <div className="font-bold text-gray-200">{title}</div>
      {desc && <div className="tb-muted text-sm mt-1">{desc}</div>}
    </div>
  );
}

export function CopyButton({ text, label = '📋' }: { text: string; label?: string }) {
  const [ok, setOk] = useState(false);
  return (
    <button
      onClick={async () => { try { await navigator.clipboard.writeText(text); setOk(true); setTimeout(() => setOk(false), 1500); } catch {} }}
      className="tb-btn tb-btn-ghost tb-btn-sm"
    >{ok ? '✅' : label}</button>
  );
}
