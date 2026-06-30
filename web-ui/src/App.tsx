import { useState } from "react";
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
import { NavProvider } from "./nav";
import { isRunning, post, useInvalidate, type AgentStatus } from "./hooks";
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

function TabButton({ tab, isActive, onClick }: { tab: typeof TABS[0]; isActive: boolean; onClick: () => void }) {
  const { t } = useTranslation();
  const Icon = tab.icon;
  return (
    <button
      key={tab.id}
      onClick={onClick}
      className={cn(
        "relative inline-flex shrink-0 items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm transition-colors",
        isActive ? "text-ink-900" : "text-fg-muted hover:text-fg",
      )}
    >
      {isActive && (
        <motion.span
          layoutId="tab-pill"
          className="absolute inset-0 rounded-lg bg-neon shadow-glow-sm"
          transition={{ type: "spring", stiffness: 500, damping: 34 }}
        />
      )}
      <span className="relative z-10 flex items-center gap-1.5">
        <Icon size={15} /> {t(tab.labelKey)}
      </span>
    </button>
  );
}

function Header({ active, setActive }: { active: string; setActive: (s: string) => void }) {
  const { t } = useTranslation();
  const { status, connected } = useAgent();
  const invalidate = useInvalidate();
  const running = isRunning(status);
  const sv = statusView(status);

  async function run() {
    try {
      await post("/run");
      invalidate();
    } catch (e) {
      alert(String(e));
    }
  }

  return (
    <header className="sticky top-0 z-20 border-b border-edge bg-ink-900/70 backdrop-blur-xl">
      <div className="mx-auto flex max-w-6xl items-center gap-4 px-5 py-3">
        <div className="flex items-center gap-2.5">
          <span className="grid h-8 w-8 place-items-center rounded-xl border border-neon/30 bg-neon/10 text-base shadow-glow-sm">
            🐇
          </span>
          <span className="font-display text-lg font-bold tracking-tight text-fg">
            job<span className="text-neon text-glow">Rabbit</span>
          </span>
        </div>

        <div className="ml-1 hidden sm:block" title={connected ? t("header.connected") : t("header.reconnecting")}>
          <StatusPill tone={sv.tone} pulse={sv.pulse}>
            {t(sv.statusKey)}
          </StatusPill>
        </div>

        <div className="flex-1" />

        <Button variant="primary" onClick={run} disabled={running}>
          {running ? <Loader2 size={15} className="animate-spin" /> : <Play size={15} />}
          {running ? t("header.runningEllipsis") : t("header.runSearch")}
        </Button>
      </div>

      {/* Navigation */}
      <nav className="mx-auto flex max-w-6xl gap-1 overflow-x-auto px-4 pb-2 scroll-thin">
        {TABS.map((tab) => (
          <TabButton key={tab.id} tab={tab} isActive={active === tab.id} onClick={() => setActive(tab.id)} />
        ))}
      </nav>
    </header>
  );
}

function Shell() {
  const [active, setActive] = useState("dashboard");
  const tab = TABS.find((t) => t.id === active) ?? TABS[0];
  return (
    <NavProvider navigate={setActive}>
      <div className="min-h-full">
        <Header active={active} setActive={setActive} />
        <main className="mx-auto max-w-6xl px-4 py-7">
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
        </main>
      </div>
    </NavProvider>
  );
}

export default function App() {
  return (
    <AgentProvider>
      <div className="bg-atmosphere" />
      <div className="bg-grain" />
      <Shell />
    </AgentProvider>
  );
}
