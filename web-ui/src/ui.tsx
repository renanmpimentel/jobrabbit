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
  return <div className={cn("bg-surface border border-border rounded-xl shadow-sm hover:border-border/80 transition-colors", className)}>{children}</div>;
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
    <div className="flex items-center justify-between gap-3 border-b border-border px-6 py-4">
      <div>
        <h3 className="text-base font-semibold tracking-tight text-fg">{title}</h3>
        {hint && <p className="mt-1 text-xs text-fg-muted">{hint}</p>}
      </div>
      {right}
    </div>
  );
}

export function SectionTitle({ children }: { children: ReactNode }) {
  return (
    <div className="mb-3 flex items-center gap-2">
      <span className="h-3 w-0.5 rounded-full bg-accent" />
      <span className="font-mono text-[11px] uppercase tracking-[0.18em] text-fg-muted">
        {children}
      </span>
    </div>
  );
}

type Variant = "primary" | "ghost" | "danger" | "subtle";
type Size = "sm" | "md";

export function Button({
  variant = "subtle",
  size = "md",
  className,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: Variant; size?: Size }) {
  const styles: Record<Variant, string> = {
    primary:
      "bg-accent text-accent-fg font-semibold hover:bg-accent/90 active:bg-accent/80",
    danger: "bg-danger text-white hover:bg-danger/90 active:bg-danger/80",
    ghost: "bg-transparent text-fg-muted hover:text-fg hover:bg-surface-2",
    subtle: "bg-surface-2 text-fg border border-border hover:border-border/60 hover:bg-surface-2/80",
  };
  const sizes: Record<Size, string> = {
    sm: "px-3 py-1.5 text-xs gap-1",
    md: "px-4 py-2 text-sm gap-1.5",
  };
  return (
    <button
      className={cn(
        "inline-flex items-center justify-center rounded-lg transition-all duration-150 active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-50",
        styles[variant],
        sizes[size],
        className,
      )}
      {...props}
    />
  );
}

type Tone = "slate" | "green" | "yellow" | "red" | "iris";

export function Badge({ children, tone = "slate" }: { children: ReactNode; tone?: Tone | string }) {
  const tones: Record<string, string> = {
    slate: "bg-surface-2 text-fg-muted border-border",
    green: "bg-success/12 text-success border-success/30",
    yellow: "bg-warn/12 text-warn border-warn/30",
    red: "bg-danger/12 text-danger border-danger/30",
    iris: "bg-info/12 text-info border-info/30",
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
        "w-full rounded-lg border border-border bg-surface px-4 py-2.5 text-sm text-fg outline-none transition placeholder:text-fg-subtle focus:border-accent focus:ring-1 focus:ring-accent/30",
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
        "scroll-thin w-full rounded-lg border border-border bg-surface px-4 py-2.5 text-sm leading-relaxed text-fg outline-none transition placeholder:text-fg-subtle focus:border-accent focus:ring-1 focus:ring-accent/30",
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
        on ? "border-accent bg-accent/20" : "border-border bg-surface-2",
      )}
    >
      <span
        className={cn(
          "ml-0.5 h-5 w-5 rounded-full transition-transform duration-200 ease-out",
          on ? "translate-x-5 bg-accent" : "bg-fg-subtle",
        )}
      />
    </button>
  );
}

export function Empty({ children }: { children: ReactNode }) {
  return <div className="px-5 py-10 text-center text-sm text-fg-muted">{children}</div>;
}

/// Status pill (agent): colored dot + label, pulses when active.
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
    neon: "bg-accent",
    warn: "bg-warn",
    danger: "bg-danger",
    muted: "bg-fg-subtle",
  };
  const text: Record<string, string> = {
    neon: "text-accent",
    warn: "text-warn",
    danger: "text-danger",
    muted: "text-fg-muted",
  };
  return (
    <span className="inline-flex items-center gap-2 rounded-full border border-border bg-surface-2 px-3 py-1 text-xs">
      <span className={cn("h-2 w-2 rounded-full", dot[tone], pulse && "animate-pulse")} />
      <span className={cn("font-medium", text[tone])}>{children}</span>
    </span>
  );
}

/// Dashboard metric card (number with count-up).
export function StatCard({ label, value }: { label: string; value: number }) {
  const n = useCountUp(value);
  return (
    <motion.div variants={fadeUp} className="relative overflow-hidden rounded-xl border border-border bg-surface px-6 py-5 shadow-sm transition-colors hover:border-border/60">
      <div className="font-mono text-3xl font-semibold tracking-tight text-fg">{n}</div>
      <div className="mt-2 text-xs text-fg-muted">{label}</div>
    </motion.div>
  );
}
