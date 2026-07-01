// Lightweight toast system — visual feedback for actions (save, run, apply…).
// Presentation only: pages call useToast() at their action call sites.
import { createContext, useCallback, useContext, useRef, useState, type ReactNode } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { CheckCircle2, AlertCircle, AlertTriangle, Info, X } from "lucide-react";
import { cn } from "./ui";

type ToastType = "success" | "error" | "info" | "warn";
type ToastAction = { label: string; onClick: () => void };
type ToastOpts = { action?: ToastAction; ttl?: number };
type Toast = { id: number; type: ToastType; message: string; action?: ToastAction };

type ToastApi = {
  show: (type: ToastType, message: string, opts?: ToastOpts) => void;
  success: (message: string) => void;
  error: (message: string) => void;
  info: (message: string) => void;
  warn: (message: string, opts?: ToastOpts) => void;
};

const ToastCtx = createContext<ToastApi>({
  show: () => {},
  success: () => {},
  error: () => {},
  info: () => {},
  warn: () => {},
});

export function useToast(): ToastApi {
  return useContext(ToastCtx);
}

const ICON = { success: CheckCircle2, error: AlertCircle, info: Info, warn: AlertTriangle } as const;
const ACCENT = {
  success: "text-success",
  error: "text-danger",
  info: "text-info",
  warn: "text-warn",
} as const;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const nextId = useRef(1);

  const remove = useCallback((id: number) => {
    setToasts((prev) => prev.filter((x) => x.id !== id));
  }, []);

  const show = useCallback(
    (type: ToastType, message: string, opts?: ToastOpts) => {
      const id = nextId.current++;
      setToasts((prev) => [...prev, { id, type, message, action: opts?.action }]);
      // Errors/warnings linger a bit longer so they can be read and acted on.
      const ttl = opts?.ttl ?? (type === "error" || type === "warn" ? 8000 : 3000);
      setTimeout(() => remove(id), ttl);
    },
    [remove],
  );

  const api: ToastApi = {
    show,
    success: (m) => show("success", m),
    error: (m) => show("error", m),
    info: (m) => show("info", m),
    warn: (m, opts) => show("warn", m, opts),
  };

  return (
    <ToastCtx.Provider value={api}>
      {children}
      <div className="pointer-events-none fixed bottom-4 right-4 z-[100] flex w-[min(92vw,360px)] flex-col gap-2">
        <AnimatePresence initial={false}>
          {toasts.map((toChange) => {
            const Icon = ICON[toChange.type];
            return (
              <motion.div
                key={toChange.id}
                layout
                initial={{ opacity: 0, y: 12, scale: 0.98 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                exit={{ opacity: 0, y: 8, scale: 0.98 }}
                transition={{ duration: 0.2, ease: [0.22, 1, 0.36, 1] }}
                className="pointer-events-auto flex items-start gap-2.5 rounded-md border border-border bg-surface px-3.5 py-3 shadow-md"
                role="status"
                aria-live="polite"
              >
                <Icon size={17} className={cn("mt-px shrink-0", ACCENT[toChange.type])} />
                <span className="min-w-0 flex-1 break-words text-sm text-fg">{toChange.message}</span>
                {toChange.action && (
                  <button
                    onClick={() => {
                      toChange.action?.onClick();
                      remove(toChange.id);
                    }}
                    className="shrink-0 rounded-md border border-border bg-surface-2 px-2 py-0.5 text-xs font-medium text-fg transition hover:border-border-strong"
                  >
                    {toChange.action.label}
                  </button>
                )}
                <button
                  onClick={() => remove(toChange.id)}
                  className="shrink-0 text-fg-subtle transition hover:text-fg"
                  aria-label="Dismiss"
                >
                  <X size={14} />
                </button>
              </motion.div>
            );
          })}
        </AnimatePresence>
      </div>
    </ToastCtx.Provider>
  );
}
