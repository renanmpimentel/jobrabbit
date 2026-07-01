// SSE: subscribes to /api/events, maintains live log+status, and invalidates React
// Query cache when backend signals `refresh` (new jobs, pending actions, etc.).
import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { AgentStatus, PendingNotify, WebEvent } from "./api";
import { getJSON } from "./api";

/// A pending alert, tagged with a monotonic `seq` so consumers can react even
/// when two consecutive pendings carry the same text.
export interface PendingAlert extends PendingNotify {
  seq: number;
}

interface AgentCtx {
  status: AgentStatus;
  log: string[];
  connected: boolean;
  lastPending: PendingAlert | null;
}

const Ctx = createContext<AgentCtx>({ status: "Idle", log: [], connected: false, lastPending: null });

export function useAgent(): AgentCtx {
  return useContext(Ctx);
}

export function AgentProvider({ children }: { children: ReactNode }) {
  const qc = useQueryClient();
  const [status, setStatus] = useState<AgentStatus>("Idle");
  const [log, setLog] = useState<string[]>([]);
  const [connected, setConnected] = useState(false);
  const [lastPending, setLastPending] = useState<PendingAlert | null>(null);
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
      // A new pending action needs the user's attention right away.
      if (we.notify) {
        const n = we.notify;
        setLastPending((prev) => ({ kind: n.kind, description: n.description, seq: (prev?.seq ?? 0) + 1 }));
      }
      // Terminal events also refresh everything (stats/sessions).
      if (we.event === "AgentFinished" || we.event === "AgentError") qc.invalidateQueries();
    };
    return () => es.close();
  }, [qc]);

  return <Ctx.Provider value={{ status, log, connected, lastPending }}>{children}</Ctx.Provider>;
}
