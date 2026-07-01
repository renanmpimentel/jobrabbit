import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { AnimatePresence, motion } from "framer-motion";
import {
  LayoutDashboard,
  User,
  AlertTriangle,
  TrendingUp,
  Terminal,
  FileCheck2,
  Settings as SettingsIcon,
  Stethoscope,
  Play,
  Loader2,
  Send,
} from "lucide-react";
import { AgentProvider, useAgent } from "./events";
import { NavProvider, useNav } from "./nav";
import { isRunning, post, useInvalidate, usePending, useSettings, type AgentStatus } from "./hooks";
import { AppShell, Button, SidebarBrand, StatusPill, cn } from "./ui";
import { useToast } from "./toast";
import Dashboard from "./pages/Dashboard";
import Profile from "./pages/Profile";
import Pending from "./pages/Pending";
import Session from "./pages/Session";
import FeedbackPage from "./pages/Feedback";
import Ats from "./pages/Ats";
import Applications from "./pages/Applications";
import Config from "./pages/Config";
import Doctor from "./pages/Doctor";

const TABS = [
  { id: "dashboard", labelKey: "nav.dashboard", icon: LayoutDashboard, el: <Dashboard />, wide: true },
  { id: "profile", labelKey: "nav.profile", icon: User, el: <Profile /> },
  { id: "pending", labelKey: "nav.pending", icon: AlertTriangle, el: <Pending /> },
  { id: "session", labelKey: "nav.session", icon: Terminal, el: <Session /> },
  { id: "ats", labelKey: "nav.ats", icon: FileCheck2, el: <Ats />, wide: true },
  { id: "applications", labelKey: "nav.applications", icon: Send, el: <Applications />, wide: true },
  { id: "feedback", labelKey: "nav.feedback", icon: TrendingUp, el: <FeedbackPage /> },
  { id: "doctor", labelKey: "nav.doctor", icon: Stethoscope, el: <Doctor /> },
  { id: "config", labelKey: "nav.config", icon: SettingsIcon, el: <Config /> },
];

function statusView(s: AgentStatus): { tone: "neon" | "warn" | "danger" | "muted"; statusKey: string; pulse: boolean } {
  if (s === "Running") return { tone: "neon", statusKey: "header.runningEllipsis", pulse: true };
  if (s === "Idle") return { tone: "muted", statusKey: "header.idle", pulse: false };
  return { tone: "danger", statusKey: "status.error", pulse: false };
}

// Lookup tab metadata by internal id (ids/routes are unchanged — grouping is display-only).
const TAB_BY_ID = Object.fromEntries(TABS.map((tab) => [tab.id, tab]));

// Sidebar grouping (labels only) — Config is pinned to the footer.
const NAV_GROUPS: { key: string; items: string[] }[] = [
  { key: "work", items: ["dashboard", "applications", "pending"] },
  { key: "resume", items: ["profile", "ats"] },
  { key: "agent", items: ["session", "feedback", "doctor"] },
];

function NavItem({
  id,
  active,
  onPick,
  badge,
}: {
  id: string;
  active: string;
  onPick: (id: string) => void;
  badge?: number;
}) {
  const { t } = useTranslation();
  const tab = TAB_BY_ID[id];
  if (!tab) return null;
  const Icon = tab.icon;
  const isActive = active === id;
  return (
    <button
      onClick={() => onPick(id)}
      aria-current={isActive ? "page" : undefined}
      className={cn(
        "flex w-full items-center gap-2.5 rounded-md px-2.5 py-2 text-left text-sm transition-colors duration-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/50",
        isActive ? "bg-surface-2 font-medium text-fg" : "text-fg-muted hover:bg-surface-2 hover:text-fg",
      )}
    >
      <Icon size={17} className={cn("shrink-0", isActive ? "text-accent" : "text-fg-subtle")} />
      <span>{t(tab.labelKey)}</span>
      {badge != null && badge > 0 && (
        <span
          aria-label={t("nav.pendingCount", { count: badge })}
          className="ml-auto grid h-5 min-w-5 shrink-0 place-items-center rounded-full bg-danger px-1.5 text-[11px] font-semibold leading-none text-white"
        >
          {badge > 99 ? "99+" : badge}
        </span>
      )}
    </button>
  );
}

function Sidebar({ active, onPick }: { active: string; onPick: (id: string) => void }) {
  const { t } = useTranslation();
  const pending = usePending();
  const pendingCount = pending.data?.length ?? 0;
  return (
    <>
      <SidebarBrand onClose={() => onPick(active)}>
        <span className="grid h-7 w-7 shrink-0 place-items-center rounded-md bg-surface-2 text-sm">🐇</span>
        <span className="text-base font-semibold tracking-tight text-fg">jobRabbit</span>
      </SidebarBrand>
      <nav className="flex-1 space-y-5 overflow-y-auto px-2.5 py-4 scroll-thin">
        {NAV_GROUPS.map((group) => (
          <div key={group.key} className="space-y-0.5">
            <div className="px-2.5 pb-1 text-[11px] font-semibold uppercase tracking-wide text-fg-subtle">
              {t(`nav.groups.${group.key}`)}
            </div>
            {group.items.map((id) => (
              <NavItem key={id} id={id} active={active} onPick={onPick} badge={id === "pending" ? pendingCount : undefined} />
            ))}
          </div>
        ))}
      </nav>
      <div className="border-t border-border px-2.5 py-2">
        <NavItem id="config" active={active} onPick={onPick} />
      </div>
    </>
  );
}

function HeaderActions() {
  const { t } = useTranslation();
  const { status, connected } = useAgent();
  const invalidate = useInvalidate();
  const toast = useToast();
  const running = isRunning(status);
  const sv = statusView(status);

  async function run() {
    try {
      await post("/run");
      invalidate();
      toast.success(t("common.started"));
    } catch (e) {
      toast.error(String(e));
    }
  }

  return (
    <>
      <div className="hidden sm:block" title={connected ? t("header.connected") : t("header.reconnecting")}>
        <StatusPill tone={sv.tone} pulse={sv.pulse}>
          {t(sv.statusKey)}
        </StatusPill>
      </div>
      <Button variant="primary" onClick={run} disabled={running} size="sm">
        {running ? <Loader2 size={14} className="animate-spin" /> : <Play size={14} />}
        <span className="hidden sm:inline">{running ? t("header.runningEllipsis") : t("header.runSearch")}</span>
      </Button>
    </>
  );
}

// Raises an in-app toast whenever a new pending action arrives, with a shortcut
// to the Pending tab. Lives inside NavProvider so it can navigate.
function PendingWatcher() {
  const { t } = useTranslation();
  const { lastPending } = useAgent();
  const nav = useNav();
  const toast = useToast();
  const seen = useRef(0);

  useEffect(() => {
    if (!lastPending || lastPending.seq === seen.current) return;
    seen.current = lastPending.seq;
    const kindLabel = t(`pending.kind.${lastPending.kind}`, lastPending.kind);
    toast.warn(`${kindLabel}: ${lastPending.description}`, {
      action: { label: t("pending.view"), onClick: () => nav("pending") },
    });
  }, [lastPending, t, toast, nav]);

  return null;
}

function Shell() {
  const { t } = useTranslation();
  const [active, setActive] = useState("dashboard");
  const [mobileOpen, setMobileOpen] = useState(false);
  const tab = TABS.find((x) => x.id === active) ?? TABS[0];

  const pick = (id: string) => {
    setActive(id);
    setMobileOpen(false);
  };

  return (
    <NavProvider navigate={setActive}>
      <PendingWatcher />
      <AppShell
        sidebar={<Sidebar active={active} onPick={pick} />}
        title={t(tab.labelKey)}
        actions={<HeaderActions />}
        wide={tab.wide}
        mobileOpen={mobileOpen}
        onMobileOpenChange={setMobileOpen}
      >
        <AnimatePresence mode="wait">
          <motion.div
            key={active}
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            transition={{ duration: 0.18, ease: [0.22, 1, 0.36, 1] }}
          >
            {tab.el}
          </motion.div>
        </AnimatePresence>
      </AppShell>
    </NavProvider>
  );
}

// Keeps the backend `locale` in sync with the language shown in the UI.
function LocaleSync() {
  const { i18n } = useTranslation();
  const settings = useSettings();
  const invalidate = useInvalidate();
  useEffect(() => {
    if (!settings.data) return;
    const uiLocale = i18n.language === "pt-BR" ? "pt-br" : "en";
    if (settings.data.locale !== uiLocale) {
      post("/settings", { ...settings.data, locale: uiLocale })
        .then(invalidate)
        .catch(() => {});
    }
  }, [settings.data, i18n.language, invalidate]);
  return null;
}

export default function App() {
  return (
    <AgentProvider>
      <LocaleSync />
      <Shell />
    </AgentProvider>
  );
}
