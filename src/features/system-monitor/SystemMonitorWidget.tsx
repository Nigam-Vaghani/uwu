import { useSystemStats } from "./useSystemStats";

function formatRam(usedMb: number, totalMb: number) {
  const usedGb = usedMb / 1024;
  if (totalMb >= 1024) {
    return `${usedGb.toFixed(1)} GB`;
  }
  return `${usedMb} MB`;
}

function formatDuration(totalSecs: number) {
  if (totalSecs <= 0) {
    return "—";
  }

  const hours = Math.floor(totalSecs / 3600);
  const minutes = Math.floor((totalSecs % 3600) / 60);
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

function formatNetwork(sendKbps: number, recvKbps: number) {
  const dominant = Math.max(sendKbps, recvKbps);
  if (dominant >= 1024) {
    return `↑ ${(sendKbps / 1024).toFixed(1)} MB/s`;
  }
  return `↑ ${Math.round(sendKbps)} KB/s`;
}

function MetricBar({
  label,
  percent,
  value,
  warn = false,
}: {
  label: string;
  percent: number;
  value: string;
  warn?: boolean;
}) {
  const clamped = Math.max(0, Math.min(100, percent));

  return (
    <div className="system-monitor-metric">
      <span className="system-monitor-label">{label}</span>
      <div className="system-monitor-bar-bg" aria-hidden="true">
        <div
          className={`system-monitor-bar-fill${warn ? " system-monitor-bar-fill--warn" : ""}`}
          style={{ width: `${clamped}%` }}
        />
      </div>
      <span className="system-monitor-value">{value}</span>
    </div>
  );
}

export function SystemMonitorWidget() {
  const { stats, loading } = useSystemStats();

  if (loading) {
    return <p className="settings-hint">Loading system stats...</p>;
  }

  const batteryValue =
    stats.batteryPercent == null
      ? "N/A"
      : `${Math.round(stats.batteryPercent)}%${stats.batteryCharging ? " ⚡" : ""}`;

  const networkPercent = Math.min(
    100,
    (Math.max(stats.networkSendKbps, stats.networkRecvKbps) / 1024) * 100,
  );

  return (
    <section className="settings-section system-monitor">
      <h2>System Monitor</h2>

      <MetricBar
        label="CPU"
        percent={stats.cpuPercent}
        value={`${Math.round(stats.cpuPercent)}%`}
        warn={stats.cpuPercent > 80}
      />

      <MetricBar
        label="RAM"
        percent={stats.ramPercent}
        value={formatRam(stats.ramUsedMb, stats.ramTotalMb)}
        warn={stats.ramPercent > 85}
      />

      <MetricBar
        label="Battery"
        percent={stats.batteryPercent ?? 0}
        value={batteryValue}
        warn={(stats.batteryPercent ?? 100) < 20}
      />

      <MetricBar
        label="Network"
        percent={networkPercent}
        value={formatNetwork(stats.networkSendKbps, stats.networkRecvKbps)}
      />

      <div className="system-monitor-metric">
        <span className="system-monitor-label">Active App</span>
        <span
          className={`system-monitor-badge${
            stats.codingSessionActive ? " system-monitor-badge--active" : ""
          }`}
        >
          {stats.activeApp ?? "—"}
        </span>
      </div>

      <div className="system-monitor-metric">
        <span className="system-monitor-label">Coding Session</span>
        <span className="system-monitor-value">
          {stats.codingSessionActive
            ? formatDuration(stats.codingSessionDurationSecs)
            : "—"}
        </span>
      </div>
    </section>
  );
}
