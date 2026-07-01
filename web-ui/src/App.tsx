import { useEffect, useState } from "react";
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
  Sun,
  Moon,
  Menu,
  X,
} from "lucide-react";
import { AgentProvider, useAgent } from "./events";
import { NavProvider } from "./nav";
import { isRunning, post, useInvalidate, useSettings, type AgentStatus } from "./hooks";
import { useTheme } from "./theme";
import { Button, StatusPill, cn } from "./ui";
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
  { id: "dashboard", labelKey: "nav.dashboard", icon: LayoutDashboard, el: <Dashboard /> },
  { id: "profile", labelKey: "nav.profile", icon: User, el: <Profile /> },
  { id: "pending", labelKey: "nav.pending", icon: AlertTriangle, el: <Pending /> },
  { id: "session", labelKey: "nav.session", icon: Terminal, el: <Session /> },
  { id: "ats", labelKey: "nav.ats", icon: FileCheck2, el: <Ats /> },
  { id: "applications", labelKey: "nav.applications", icon: Send, el: <Applications /> },
  { id: "feedback", labelKey: "nav.feedback", icon: TrendingUp, el: <FeedbackPage /> },
  { id: "doctor", labelKey: "nav.doctor", icon: Stethoscope, el: <Doctor /> },
  { id: "config", labelKey: "nav.config", icon: SettingsIcon, el: <Config /> },
];

function statusView(s: AgentStatus): { tone: "neon" | "warn" | "danger" | "muted"; statusKey: string; pulse: boolean } {
  if (s === "Running") return { tone: "neon", statusKey: "header.runningEllipsis", pulse: true };
  if (s === "Idle") return { tone: "muted", statusKey: "header.idle", pulse: false };
  return { tone: "danger", statusKey: "status.error", pulse: false };
}

function SidebarNavItem({ tab, isActive, onClick }: { tab: typeof TABS[0]; isActive: boolean; onClick: () => void }) {
  const { t } = useTranslation();
  const Icon = tab.icon;
  return (
    <button
      onClick={onClick}
      className={cn(
        "relative flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm transition-colors duration-150",
        isActive
          ? "bg-accent/12 text-accent font-medium"
          : "text-fg-muted hover:text-fg hover:bg-surface-2",
      )}
    >
      <Icon size={18} className="shrink-0" />
      <span>{t(tab.labelKey)}</span>
      {isActive && (
        <motion.span
          layoutId="sidebar-active"
          className="absolute -left-0.5 top-1/2 h-5 w-1 -translate-y-1/2 rounded-r-full bg-accent"
          transition={{ type: "spring", stiffness: 500, damping: 34 }}
        />
      )}
    </button>
  );
}

function Sidebar({ active, setActive, open, setOpen }: { active: string; setActive: (s: string) => void; open: boolean; setOpen: (o: boolean) => void }) {
  const { t } = useTranslation();

  return (
    <>
      {/* Overlay for mobile */}
      <AnimatePresence>
        {open && (
          <motion.div
            key="sidebar-overlay"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={() => setOpen(false)}
            className="fixed inset-0 z-40 bg-black/40 lg:hidden"
          />
        )}
      </AnimatePresence>

      {/* Sidebar */}
      <aside
        className={cn(
          "fixed inset-y-0 left-0 z-50 flex h-screen w-64 flex-col border-r border-border bg-surface transition-transform duration-300 ease-out lg:sticky lg:top-0 lg:z-auto lg:translate-x-0",
          open ? "translate-x-0" : "-translate-x-full",
        )}
      >
        {/* Brand */}
        <div className="flex items-center gap-3 border-b border-border px-5 py-5">
          <span className="grid h-9 w-9 shrink-0 place-items-center rounded-lg border border-border bg-surface-2 text-base font-semibold">
            🐇
          </span>
          <span className="text-base font-bold tracking-tight text-fg">
            job<span className="text-accent">Rabbit</span>
          </span>
          <button
            onClick={() => setOpen(false)}
            className="ml-auto lg:hidden"
            aria-label="Close sidebar"
          >
            <X size={18} className="text-fg-muted" />
          </button>
        </div>

        {/* Nav items */}
        <nav className="flex-1 space-y-1.5 overflow-y-auto px-4 py-4 scroll-thin">
          {TABS.map((tab) => (
            <SidebarNavItem
              key={tab.id}
              tab={tab}
              isActive={active === tab.id}
              onClick={() => {
                setActive(tab.id);
                setOpen(false); // Close sidebar on mobile after click
              }}
            />
          ))}
        </nav>
      </aside>
    </>
  );
}

function Topbar({ active, setActive, sidebarOpen, setSidebarOpen }: { active: string; setActive: (s: string) => void; sidebarOpen: boolean; setSidebarOpen: (o: boolean) => void }) {
  const { t } = useTranslation();
  const { status, connected } = useAgent();
  const invalidate = useInvalidate();
  const running = isRunning(status);
  const sv = statusView(status);
  const { theme, toggle } = useTheme();
  const tab = TABS.find((t) => t.id === active) ?? TABS[0];

  async function run() {
    try {
      await post("/run");
      invalidate();
    } catch (e) {
      alert(String(e));
    }
  }

  return (
    <header className="sticky top-0 z-20 border-b border-border bg-surface">
      <div className="flex items-center justify-between gap-4 px-6 py-4">
        {/* Left: page title + hamburger */}
        <div className="flex items-center gap-4">
          <button
            onClick={() => setSidebarOpen(!sidebarOpen)}
            className="lg:hidden"
            aria-label="Toggle sidebar"
          >
            <Menu size={18} className="text-fg-muted" />
          </button>
          <h1 className="text-base font-semibold text-fg">{t(tab.labelKey)}</h1>
        </div>

        {/* Right: pills + buttons */}
        <div className="flex items-center gap-2">
          <div className="hidden sm:block" title={connected ? t("header.connected") : t("header.reconnecting")}>
            <StatusPill tone={sv.tone} pulse={sv.pulse}>
              {t(sv.statusKey)}
            </StatusPill>
          </div>

          <Button variant="ghost" onClick={toggle} aria-label="Toggle theme" size="sm">
            {theme === "light" ? <Moon size={16} /> : <Sun size={16} />}
          </Button>

          <Button variant="primary" onClick={run} disabled={running} size="sm">
            {running ? <Loader2 size={14} className="animate-spin" /> : <Play size={14} />}
            <span className="hidden sm:inline">{running ? t("header.runningEllipsis") : t("header.runSearch")}</span>
          </Button>
        </div>
      </div>
    </header>
  );
}

function Shell() {
  const [active, setActive] = useState("dashboard");
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const tab = TABS.find((t) => t.id === active) ?? TABS[0];

  return (
    <NavProvider navigate={setActive}>
      <div className="flex min-h-screen bg-bg">
        {/* Sidebar */}
        <Sidebar active={active} setActive={setActive} open={sidebarOpen} setOpen={setSidebarOpen} />

        {/* Main area */}
        <div className="flex flex-1 flex-col">
          {/* Topbar */}
          <Topbar active={active} setActive={setActive} sidebarOpen={sidebarOpen} setSidebarOpen={setSidebarOpen} />

          {/* Content */}
          <main className="flex-1 overflow-y-auto">
            <div className="mx-auto max-w-6xl px-6 py-8 sm:px-8">
              <AnimatePresence mode="wait">
                <motion.div
                  key={active}
                  initial={{ opacity: 0, y: 8 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -6 }}
                  transition={{ duration: 0.22, ease: [0.22, 1, 0.36, 1] }}
                >
                  {tab.el}
                </motion.div>
              </AnimatePresence>
            </div>
          </main>
        </div>
      </div>
    </NavProvider>
  );
}

// Keeps the backend `locale` in sync with the language shown in the UI. The UI
// language can come from the browser detector (i18n) without ever touching the
// backend, which would leave the agent (ATS/search/feedback) running in the wrong
// language. On load, if they diverge, the displayed language wins and is pushed
// to the backend.
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
