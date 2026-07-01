// UI — jobRabbit Design System (DESIGN.md). Light-only, Inter, Notion/Height.
// Canonical components: AppShell, PageHeader, Card, ListRow, ScoreBadge,
// StatusBadge, Button, StatCard, Empty/Loading — plus shared field primitives.
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { AnimatePresence, motion } from "framer-motion";
import { AlertCircle, ChevronDown, RefreshCw, Menu, X } from "lucide-react";
import type {
  ButtonHTMLAttributes,
  InputHTMLAttributes,
  ReactNode,
  SelectHTMLAttributes,
  TextareaHTMLAttributes,
} from "react";
import { fadeUp, useCountUp } from "./motion";

export function cn(...parts: unknown[]): string {
  return twMerge(clsx(parts));
}

const FOCUS =
  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:ring-offset-2 focus-visible:ring-offset-bg";

/* ============================================================
   AppShell — sidebar 240 + header sticky + container 960/1120
   ============================================================ */
export function AppShell({
  sidebar,
  title,
  subtitle,
  actions,
  children,
  wide,
  mobileOpen,
  onMobileOpenChange,
}: {
  sidebar: ReactNode;
  title: string;
  subtitle?: string;
  actions?: ReactNode;
  children: ReactNode;
  wide?: boolean;
  mobileOpen: boolean;
  onMobileOpenChange: (o: boolean) => void;
}) {
  return (
    <div className="flex min-h-screen bg-bg text-fg">
      {/* Desktop rail — 240px, always in flow */}
      <aside className="sticky top-0 hidden h-screen w-60 shrink-0 flex-col border-r border-border bg-sidebar lg:flex">
        {sidebar}
      </aside>

      {/* Mobile drawer */}
      <AnimatePresence>
        {mobileOpen && (
          <div className="lg:hidden">
            <motion.div
              key="overlay"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              onClick={() => onMobileOpenChange(false)}
              className="fixed inset-0 z-40 bg-black/40"
            />
            <motion.aside
              key="drawer"
              initial={{ x: "-100%" }}
              animate={{ x: 0 }}
              exit={{ x: "-100%" }}
              transition={{ type: "tween", duration: 0.22, ease: [0.22, 1, 0.36, 1] }}
              className="fixed inset-y-0 left-0 z-50 flex w-60 flex-col border-r border-border bg-sidebar"
            >
              {sidebar}
            </motion.aside>
          </div>
        )}
      </AnimatePresence>

      {/* Main column */}
      <div className="flex min-w-0 flex-1 flex-col">
        <header className="sticky top-0 z-20 border-b border-border bg-bg/90 backdrop-blur supports-[backdrop-filter]:bg-bg/70">
          <div className={cn("mx-auto flex items-center gap-4 px-6 py-3.5 sm:px-8", wide ? "max-w-content-wide" : "max-w-content")}>
            <button
              onClick={() => onMobileOpenChange(true)}
              className="lg:hidden"
              aria-label="Open menu"
            >
              <Menu size={18} className="text-fg-muted" />
            </button>
            <PageHeader title={title} subtitle={subtitle} actions={actions} />
          </div>
        </header>
        <main className="flex-1">
          <div className={cn("mx-auto px-6 py-8 sm:px-8", wide ? "max-w-content-wide" : "max-w-content")}>
            {children}
          </div>
        </main>
      </div>
    </div>
  );
}

/* PageHeader — título 20 semibold + subtítulo + ações */
export function PageHeader({
  title,
  subtitle,
  actions,
}: {
  title: string;
  subtitle?: string;
  actions?: ReactNode;
}) {
  return (
    <div className="flex flex-1 items-center justify-between gap-4">
      <div className="min-w-0">
        <h1 className="truncate text-lg font-semibold tracking-tight text-fg">{title}</h1>
        {subtitle && <p className="truncate text-sm text-fg-muted">{subtitle}</p>}
      </div>
      {actions && <div className="flex shrink-0 items-center gap-2">{actions}</div>}
    </div>
  );
}

/* Sidebar helpers (brand + nav item) for the app to compose */
export function SidebarBrand({ children, onClose }: { children: ReactNode; onClose?: () => void }) {
  return (
    <div className="flex items-center gap-2.5 px-4 py-4">
      {children}
      {onClose && (
        <button onClick={onClose} className="ml-auto lg:hidden" aria-label="Close menu">
          <X size={18} className="text-fg-muted" />
        </button>
      )}
    </div>
  );
}

/* ============================================================
   Card / CardHeader / SectionTitle
   ============================================================ */
export function Card({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cn("rounded-md border border-border bg-surface", className)}>{children}</div>;
}

export function CardBody({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cn("p-6", className)}>{children}</div>;
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
      <div className="min-w-0">
        <h3 className="text-base font-semibold tracking-tight text-fg">{title}</h3>
        {hint && <p className="mt-0.5 text-sm text-fg-muted">{hint}</p>}
      </div>
      {right && <div className="shrink-0">{right}</div>}
    </div>
  );
}

export function SectionTitle({
  children,
  count,
  right,
}: {
  children: ReactNode;
  count?: number;
  right?: ReactNode;
}) {
  return (
    <div className="mb-2.5 flex items-center gap-2">
      <span className="text-xs font-semibold uppercase tracking-wide text-fg-subtle">{children}</span>
      {count != null && (
        <span className="rounded bg-surface-2 px-1.5 py-0.5 text-xs font-medium tabular-nums text-fg-muted">
          {count}
        </span>
      )}
      {right && <span className="ml-auto">{right}</span>}
    </div>
  );
}

/* ListRow — hover bg/muted, sem card por item. padding 12v/16h */
export function ListRow({
  children,
  className,
  onClick,
}: {
  children: ReactNode;
  className?: string;
  onClick?: () => void;
}) {
  return (
    <div
      onClick={onClick}
      className={cn(
        "flex items-center gap-4 px-4 py-3 transition-colors",
        onClick && "cursor-pointer",
        "hover:bg-surface-2",
        className,
      )}
    >
      {children}
    </div>
  );
}

/* ============================================================
   Button
   ============================================================ */
type Variant = "primary" | "secondary" | "ghost" | "danger" | "subtle";
type Size = "sm" | "md";

export function Button({
  variant = "secondary",
  size = "md",
  className,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: Variant; size?: Size }) {
  const styles: Record<Variant, string> = {
    primary: "bg-accent text-accent-fg font-medium hover:bg-accent-hover active:bg-accent-hover",
    secondary: "bg-surface text-fg border border-border hover:bg-surface-2 hover:border-border-strong",
    subtle: "bg-surface text-fg border border-border hover:bg-surface-2 hover:border-border-strong",
    ghost: "bg-transparent text-fg-muted hover:bg-surface-2 hover:text-fg",
    danger: "bg-danger text-white font-medium hover:bg-danger/90 active:bg-danger/80",
  };
  const sizes: Record<Size, string> = {
    sm: "px-2.5 py-1.5 text-xs gap-1.5",
    md: "px-3.5 py-2 text-sm gap-2",
  };
  return (
    <button
      className={cn(
        "inline-flex items-center justify-center rounded-md transition-colors duration-150 disabled:cursor-not-allowed disabled:opacity-50",
        FOCUS,
        styles[variant],
        sizes[size],
        className,
      )}
      {...props}
    />
  );
}

/* ============================================================
   Badges — generic Badge + canonical ScoreBadge + StatusBadge
   ============================================================ */
type Tone = "slate" | "green" | "yellow" | "red" | "iris";

export function Badge({ children, tone = "slate" }: { children: ReactNode; tone?: Tone | string }) {
  const tones: Record<string, string> = {
    slate: "bg-surface-2 text-fg-muted",
    green: "bg-success-tint text-success",
    yellow: "bg-warn-tint text-warn",
    red: "bg-danger-tint text-danger",
    iris: "bg-info-tint text-info",
  };
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-xs font-medium tabular-nums",
        tones[tone] || tones.slate,
      )}
    >
      {children}
    </span>
  );
}

// Continuous score band per DESIGN.md (never color alone — always number + band).
export function scoreBand(value: number): { wrap: string; text: string } {
  if (value >= 0.75) return { wrap: "bg-success-tint", text: "text-success" };
  if (value >= 0.55) return { wrap: "bg-warn-tint", text: "text-warn" };
  return { wrap: "bg-danger-tint", text: "text-danger" };
}

/// Canonical score chip: decimal number + subtle color band. value is 0..1.
export function ScoreBadge({ value, className }: { value: number | null; className?: string }) {
  if (value == null) {
    return (
      <span className={cn("inline-flex min-w-[3rem] items-center justify-center rounded px-2 py-0.5 text-sm font-medium tabular-nums bg-surface-2 text-fg-subtle", className)}>
        —
      </span>
    );
  }
  const b = scoreBand(value);
  return (
    <span
      title={`fit ${value.toFixed(2)}`}
      className={cn(
        "inline-flex min-w-[3rem] items-center justify-center rounded px-2 py-0.5 text-sm font-semibold tabular-nums",
        b.wrap,
        b.text,
        className,
      )}
    >
      {value.toFixed(2)}
    </span>
  );
}

type Tier = "blocking" | "decision" | "routine";

/// Pending status badge (DESIGN.md): blocking(red) / decision(blue) / routine(neutral).
export function StatusBadge({ tier, children }: { tier: Tier; children: ReactNode }) {
  const styles: Record<Tier, string> = {
    blocking: "bg-danger-tint text-danger border border-danger/30",
    decision: "bg-info-tint text-info border border-info/25",
    routine: "bg-surface-2 text-fg-muted border border-transparent",
  };
  return (
    <span className={cn("inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-xs font-medium", styles[tier])}>
      {children}
    </span>
  );
}

/* ============================================================
   Fields — Input / Textarea / Select / Toggle
   ============================================================ */
const FIELD =
  "w-full rounded-md border border-border bg-surface px-3 py-2 text-sm text-fg outline-none transition placeholder:text-fg-subtle hover:border-border-strong focus:border-accent focus-visible:ring-2 focus-visible:ring-accent/30";

export function Input({ className, ...props }: InputHTMLAttributes<HTMLInputElement>) {
  return <input className={cn(FIELD, className)} {...props} />;
}

export function Textarea({ className, ...props }: TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return <textarea className={cn("scroll-thin leading-relaxed", FIELD, className)} {...props} />;
}

export function Select({
  className,
  wrapperClassName,
  children,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement> & { wrapperClassName?: string }) {
  return (
    <div className={cn("relative inline-block", wrapperClassName)}>
      <select className={cn(FIELD, "cursor-pointer appearance-none pr-9", className)} {...props}>
        {children}
      </select>
      <ChevronDown
        size={15}
        className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-fg-subtle"
      />
    </div>
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
        "relative inline-flex h-5 w-9 shrink-0 items-center rounded-full transition-colors duration-200",
        FOCUS,
        on ? "bg-accent" : "bg-border-strong",
      )}
    >
      <span
        className={cn(
          "ml-0.5 h-4 w-4 rounded-full bg-white shadow-sm transition-transform duration-200 ease-out",
          on ? "translate-x-4" : "translate-x-0",
        )}
      />
    </button>
  );
}

/* ============================================================
   States — Empty / Skeleton / Loading / Error
   ============================================================ */
export function Empty({
  children,
  icon,
  hint,
}: {
  children: ReactNode;
  icon?: ReactNode;
  hint?: ReactNode;
}) {
  return (
    <div className="flex flex-col items-center gap-2 px-6 py-14 text-center">
      {icon && <div className="text-fg-subtle">{icon}</div>}
      <p className="text-sm font-medium text-fg-muted">{children}</p>
      {hint && <p className="max-w-sm text-sm text-fg-subtle">{hint}</p>}
    </div>
  );
}

export function Skeleton({ className }: { className?: string }) {
  return <div className={cn("skeleton", className)} />;
}

export function SkeletonRows({ rows = 4 }: { rows?: number }) {
  return (
    <div className="divide-y divide-border">
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} className="flex items-center gap-4 px-4 py-3.5">
          <div className="flex-1 space-y-2">
            <Skeleton className="h-3.5 w-1/2" />
            <Skeleton className="h-3 w-1/3" />
          </div>
          <Skeleton className="h-5 w-12" />
        </div>
      ))}
    </div>
  );
}

// Standard loading placeholder (alias for a rows skeleton).
export function LoadingState({ rows = 4 }: { rows?: number }) {
  return <SkeletonRows rows={rows} />;
}

export function ErrorState({
  message,
  retryLabel,
  onRetry,
}: {
  message: string;
  retryLabel?: string;
  onRetry?: () => void;
}) {
  return (
    <div className="flex flex-col items-center gap-3 px-6 py-12 text-center">
      <AlertCircle size={22} className="text-danger" />
      <p className="text-sm text-fg-muted">{message}</p>
      {onRetry && (
        <Button variant="secondary" size="sm" onClick={onRetry}>
          <RefreshCw size={13} />
          {retryLabel ?? "Retry"}
        </Button>
      )}
    </div>
  );
}

/* Callout — left accent + tint (health/notice blocks) */
type CalloutTone = "danger" | "warn" | "info" | "success" | "neutral";
export function Callout({
  tone = "neutral",
  icon,
  title,
  children,
  right,
  className,
}: {
  tone?: CalloutTone;
  icon?: ReactNode;
  title?: ReactNode;
  children?: ReactNode;
  right?: ReactNode;
  className?: string;
}) {
  const tones: Record<CalloutTone, string> = {
    danger: "border-l-danger bg-danger-tint",
    warn: "border-l-warn bg-warn-tint",
    info: "border-l-info bg-info-tint",
    success: "border-l-success bg-success-tint",
    neutral: "border-l-border-strong bg-surface-2",
  };
  const iconColor: Record<CalloutTone, string> = {
    danger: "text-danger",
    warn: "text-warn",
    info: "text-info",
    success: "text-success",
    neutral: "text-fg-muted",
  };
  return (
    <div className={cn("rounded-md border border-border border-l-[3px] px-4 py-3", tones[tone], className)}>
      <div className="flex items-start gap-3">
        {icon && <div className={cn("mt-0.5 shrink-0", iconColor[tone])}>{icon}</div>}
        <div className="min-w-0 flex-1">
          {title && <div className="text-sm font-medium text-fg">{title}</div>}
          {children && <div className={cn(title && "mt-2")}>{children}</div>}
        </div>
        {right && <div className="shrink-0">{right}</div>}
      </div>
    </div>
  );
}

/* Agent status pill (header) */
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
    <span className="inline-flex items-center gap-1.5 rounded-full border border-border bg-surface px-2.5 py-1 text-xs">
      <span className={cn("h-1.5 w-1.5 rounded-full", dot[tone], pulse && "animate-pulse")} />
      <span className={cn("font-medium", text[tone])}>{children}</span>
    </span>
  );
}

/* StatCard — número grande (28) + label */
export function StatCard({ label, value }: { label: string; value: number }) {
  const n = useCountUp(value);
  return (
    <motion.div variants={fadeUp} className="rounded-md border border-border bg-surface px-5 py-4">
      <div className="text-xl font-semibold tracking-tight tabular-nums text-fg">{n}</div>
      <div className="mt-1 text-sm text-fg-muted">{label}</div>
    </motion.div>
  );
}
