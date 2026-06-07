import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invokeCommand } from "../../hooks/useTauriStore";
import type { SystemStats } from "./system-monitor.types";

export const SYSTEM_STATS_EVENT = "system:stats";

const defaultStats: SystemStats = {
  cpuPercent: 0,
  ramUsedMb: 0,
  ramTotalMb: 0,
  ramPercent: 0,
  batteryPercent: null,
  batteryCharging: null,
  networkSendKbps: 0,
  networkRecvKbps: 0,
  activeWindow: null,
  activeApp: null,
  codingSessionActive: false,
  codingSessionDurationSecs: 0,
  uptimeSecs: 0,
};

export function useSystemStats() {
  const [stats, setStats] = useState<SystemStats>(defaultStats);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let active = true;

    void invokeCommand("get_system_stats", {})
      .then((snapshot) => {
        if (active) {
          setStats(snapshot);
        }
      })
      .finally(() => {
        if (active) {
          setLoading(false);
        }
      });

    const unlistenPromise = listen<SystemStats>(SYSTEM_STATS_EVENT, (event) => {
      if (active) {
        setStats(event.payload);
      }
    });

    return () => {
      active = false;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  return { stats, loading };
}
