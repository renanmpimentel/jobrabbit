// SSE: subscribes to /api/events, maintains live log+status, and invalidates React
// Query cache when backend signals `refresh` (new jobs, pending actions, etc.).
import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { AgentStatus, WebEvent } from "./api";
import { getJSON } from "./api";

interface AgentCtx {
  status: AgentStatus;
  log: string[];
  connected: boolean;
}

const Ctx = createContext<AgentCtx>({ status: "Idle", log: [], connected: false });

export function useAgent(): AgentCtx {
  return useContext(Ctx);
}

export function AgentProvider({ children }: { children: ReactNode }) {
  const qc = useQueryClient();
  const [status, setStatus] = useState<AgentStatus>("Idle");
  const [log, setLog] = useState<string[]>([]);
  const [connected, setConnected] = useState(false);
  const seeded = useRef(false);

  // Initial history (log + status) on load.
  useEffect(() => {
    if (seeded.current) return;
    seeded.current = true;
    getJSON<string[]>("/log").then(setLog).catch(() => {});
    getJSON<AgentStatus>("/status").then(setStatus).catch(() => {});
  }, []);

  useEffect(() => {
    const es = new EventSource("/api/events");
    es.onopen = () => setConnected(true);
    es.onerror = () => setConnected(false);
    es.onmessage = (e) => {
      let we: WebEvent;
      try {
        we = JSON.parse(e.data);
      } catch {
        return;
      }
      if (we.logs?.length) setLog((prev) => [...prev, ...we.logs].slice(-2000));
      if (we.status) setStatus(we.status);
      if (we.refresh) qc.invalidateQueries();
      // Terminal events also refresh everything (stats/sessions).
      if (we.event === "AgentFinished" || we.event === "AgentError") qc.invalidateQueries();
    };
    return () => es.close();
  }, [qc]);

  return <Ctx.Provider value={{ status, log, connected }}>{children}</Ctx.Provider>;
}
