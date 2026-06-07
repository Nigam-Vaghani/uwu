export type TtsProvider = "webSpeech" | "windowsSapi" | "piper" | "silent";

export type VoiceInfo = {
  id: string;
  name: string;
  language?: string | null;
};

export type TtsConfig = {
  provider: TtsProvider;
  voiceId: string;
  rate: number;
  pitch: number;
  volume: number;
  speakOnReminder: boolean;
  speakOnAi: boolean;
  piperModelPath: string;
};

export type SetTtsConfigInput = {
  provider: TtsProvider;
  voiceId: string;
  rate: number;
  pitch: number;
  volume: number;
  speakOnReminder: boolean;
  speakOnAi: boolean;
  piperModelPath: string;
};

export const TTS_PROVIDER_OPTIONS: { value: TtsProvider; label: string }[] = [
  { value: "webSpeech", label: "Web Speech API (Browser)" },
  { value: "windowsSapi", label: "Windows SAPI (System)" },
  { value: "piper", label: "Piper TTS (Local Neural)" },
  { value: "silent", label: "Silent (No Voice)" },
];

export const DEFAULT_TTS_CONFIG: TtsConfig = {
  provider: "webSpeech",
  voiceId: "",
  rate: 1.0,
  pitch: 1.0,
  volume: 0.8,
  speakOnReminder: true,
  speakOnAi: true,
  piperModelPath: "",
};

export const TEST_VOICE_PHRASE = "Hi! I'm your UWU companion. Nice to meet you!";

export function isRustTtsProvider(provider: TtsProvider): boolean {
  return provider === "windowsSapi" || provider === "piper";
}

export function clampRate(value: number): number {
  return Math.min(2, Math.max(0.5, value));
}

export function clampPitch(value: number): number {
  return Math.min(2, Math.max(0.5, value));
}

export function clampVolume(value: number): number {
  return Math.min(1, Math.max(0, value));
}

export function formatRate(value: number): string {
  return `${value.toFixed(1)}×`;
}

export function formatPitch(value: number): string {
  return `${value.toFixed(1)}×`;
}

export function formatVolumePercent(value: number): string {
  return `${Math.round(value * 100)}%`;
}

export function rateFromSlider(value: number): number {
  return clampRate(value / 100);
}

export function pitchFromSlider(value: number): number {
  return clampPitch(value / 100);
}

export function rateToSlider(value: number): number {
  return Math.round(clampRate(value) * 100);
}

export function pitchToSlider(value: number): number {
  return Math.round(clampPitch(value) * 100);
}

export function volumeToSlider(value: number): number {
  return Math.round(clampVolume(value) * 100);
}

export function volumeFromSlider(value: number): number {
  return clampVolume(value / 100);
}
