import { create } from "zustand";

export type WindowPosition = {
  x: number;
  y: number;
};

export type AppConfig = {
  petName: string;
  personality: string;
  opacity: number;
  position: WindowPosition | null;
  hasSeenWelcome: boolean;
  activeSkin: string;
  activeTheme: string;
  activeSoundPack: string;
};

const defaultConfig: AppConfig = {
  petName: "Uwu-chan",
  personality: "Cheerful",
  opacity: 0.9,
  position: null,
  hasSeenWelcome: false,
  activeSkin: "default",
  activeTheme: "elegance",
  activeSoundPack: "default",
};

type SettingsStore = {
  config: AppConfig;
  hydrated: boolean;
  setHydrated: (hydrated: boolean) => void;
  setConfig: (config: AppConfig) => void;
  patchConfig: (patch: Partial<AppConfig>) => void;
  resetConfig: () => void;
};

export const useSettingsStore = create<SettingsStore>((set) => ({
  config: defaultConfig,
  hydrated: false,
  setHydrated: (hydrated) => set({ hydrated }),
  setConfig: (config) => set({ config }),
  patchConfig: (patch) => set((state) => ({ config: { ...state.config, ...patch } })),
  resetConfig: () => set({ config: defaultConfig }),
}));

export { defaultConfig };