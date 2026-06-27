// UI primitives — "Mission Control" direction (ink + glass + neon).
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { motion } from "framer-motion";
import type {
  ButtonHTMLAttributes,
  InputHTMLAttributes,
  ReactNode,
  TextareaHTMLAttributes,
} from "react";
import { fadeUp, useCountUp } from "./motion";

export function cn(...parts: unknown[]): string {
  return twMerge(clsx(parts));
}

export function Card({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cn("glass rounded-2xl", className)}>{children}</div>;
}

export function CardHeader({
  title,
  hint,
  right,
}: {
  title: string;
  hint?: string;
  right?: ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-3 border-b border-edge px-5 py-3.5">
      <div>
        <h3 className="font-display text-[15px] font-semibold tracking-tight text-fg">{title}</h3>
        {hint && <p className="mt-0.5 text-xs text-fg-muted">{hint}</p>}
      </div>
      {right}
    </div>
  );
}

export function SectionTitle({ children }: { children: ReactNode }) {
  return (
    <div className="mb-3 flex items-center gap-2">
      <span className="h-3.5 w-0.5 rounded-full bg-neon shadow-glow-sm" />
      <span className="font-mono text-[11px] uppercase tracking-[0.18em] text-fg-muted">
        {children}
      </span>
    </div>
  );
}

type Variant = "primary" | "ghost" | "danger" | "subtle";

export function Button({
  variant = "subtle",
  className,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: Variant }) {
  const styles: Record<Variant, string> = {
    primary:
      "bg-neon text-ink-900 font-semibold hover:bg-neon-dim shadow-glow-sm hover:shadow-glow",
    danger: "bg-danger/90 text-white hover:bg-danger",
    ghost: "bg-transparent text-fg-muted hover:text-fg hover:bg-white/5",
    subtle: "bg-white/[0.04] text-fg hover:bg-white/[0.08] border border-edge",
  };
  return (
    <button
      className={cn(
        "inline-flex items-center justify-center gap-1.5 rounded-xl px-3.5 py-2 text-sm transition-all duration-150 active:scale-[0.97] disabled:cursor-not-allowed disabled:opacity-50",
        styles[variant],
        className,
      )}
      {...props}
    />
  );
}

type Tone = "slate" | "green" | "yellow" | "red" | "iris";

export function Badge({ children, tone = "slate" }: { children: ReactNode; tone?: Tone | string }) {
  const tones: Record<string, string> = {
    slate: "bg-white/[0.05] text-fg-muted border-edge",
    green: "bg-neon/12 text-neon border-neon/30",
    yellow: "bg-warn/12 text-warn border-warn/30",
    red: "bg-danger/12 text-danger border-danger/30",
    iris: "bg-iris/12 text-iris border-iris/30",
  };
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-md border px-2 py-0.5 font-mono text-[11px] tracking-tight",
        tones[tone] || tones.slate,
      )}
    >
      {children}
    </span>
  );
}

export function Input({ className, ...props }: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      className={cn(
        "w-full rounded-xl border border-edge bg-ink-850 px-3.5 py-2 text-sm text-fg outline-none transition placeholder:text-fg-dim focus:border-neon/50 focus:ring-2 focus:ring-neon/15",
        className,
      )}
      {...props}
    />
  );
}

export function Textarea({ className, ...props }: TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return (
    <textarea
      className={cn(
        "scroll-thin w-full rounded-xl border border-edge bg-ink-850 px-3.5 py-2.5 text-sm leading-relaxed text-fg outline-none transition placeholder:text-fg-dim focus:border-neon/50 focus:ring-2 focus:ring-neon/15",
        className,
      )}
      {...props}
    />
  );
}

export function Toggle({ on, onClick, label }: { on: boolean; onClick: () => void; label?: string }) {
  return (
    <button
      onClick={onClick}
      role="switch"
      aria-checked={on}
      aria-label={label}
      className={cn(
        "relative inline-flex h-6 w-11 items-center rounded-full border transition-colors duration-200",
        on ? "border-neon/50 bg-neon/25 shadow-glow-sm" : "border-edge bg-white/[0.04]",
      )}
    >
      <span
        className={cn(
          "ml-0.5 h-5 w-5 rounded-full transition-transform duration-200 ease-out",
          on ? "translate-x-5 bg-neon" : "bg-fg-dim",
        )}
      />
    </button>
  );
}

export function Empty({ children }: { children: ReactNode }) {
  return <div className="px-5 py-10 text-center text-sm text-fg-muted">{children}</div>;
}

/// Status pill (agent): colored dot + label, glows when active.
export function StatusPill({
  children,
  tone,
  pulse,
}: {
  children: ReactNode;
  tone: "neon" | "warn" | "danger" | "muted";
  pulse?: boolean;
}) {
  const dot: Record<string, string> = {
    neon: "bg-neon",
    warn: "bg-warn",
    danger: "bg-danger",
    muted: "bg-fg-dim",
  };
  const text: Record<string, string> = {
    neon: "text-neon",
    warn: "text-warn",
    danger: "text-danger",
    muted: "text-fg-muted",
  };
  return (
    <span className="inline-flex items-center gap-2 rounded-full border border-edge bg-white/[0.04] px-3 py-1 text-xs">
      <span className={cn("h-2 w-2 rounded-full", dot[tone], pulse && "animate-pulseGlow")} />
      <span className={cn("font-medium", text[tone])}>{children}</span>
    </span>
  );
}

/// Dashboard metric card (number with count-up + neon glow below).
export function StatCard({ label, value }: { label: string; value: number }) {
  const n = useCountUp(value);
  return (
    <motion.div variants={fadeUp} className="glass relative overflow-hidden rounded-2xl px-5 py-4">
      <div className="font-mono text-3xl font-semibold tracking-tight text-fg">{n}</div>
      <div className="mt-1 text-xs text-fg-muted">{label}</div>
      <span className="pointer-events-none absolute inset-x-0 -bottom-10 h-16 bg-neon/10 blur-2xl" />
    </motion.div>
  );
}
