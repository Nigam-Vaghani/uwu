export type SystemStats = {
  cpuPercent: number;
  ramUsedMb: number;
  ramTotalMb: number;
  ramPercent: number;
  batteryPercent: number | null;
  batteryCharging: boolean | null;
  networkSendKbps: number;
  networkRecvKbps: number;
  activeWindow: string | null;
  activeApp: string | null;
  codingSessionActive: boolean;
  codingSessionDurationSecs: number;
  uptimeSecs: number;
};

export type CodingSession = {
  id: number;
  appName: string;
  windowTitle: string | null;
  startedAt: number;
  endedAt: number | null;
  durationSecs: number | null;
};
