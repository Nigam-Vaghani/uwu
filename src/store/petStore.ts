import { create } from "zustand";
import type { PetAnimation, SpeechMessage } from "../features/pet/pet.types";

type PetStore = {
  mode: PetAnimation;
  speech: SpeechMessage | null;
  speechQueue: SpeechMessage[];
  menuOpen: boolean;
  menuPosition: { x: number; y: number };
  achievementToast: { id: string; name: string; icon: string } | null;
  setMode: (mode: PetAnimation) => void;
  setSpeech: (speech: SpeechMessage | null) => void;
  enqueueSpeech: (message: SpeechMessage) => void;
  dequeueSpeech: () => void;
  openMenu: (position: { x: number; y: number }) => void;
  closeMenu: () => void;
  showAchievementToast: (achievement: { id: string; name: string; icon: string }) => void;
  clearAchievementToast: () => void;
};

export const usePetStore = create<PetStore>((set, get) => ({
  mode: "idle",
  speech: null,
  speechQueue: [],
  menuOpen: false,
  menuPosition: { x: 0, y: 0 },
  achievementToast: null,
  setMode: (mode) => set({ mode }),
  setSpeech: (speech) => set({ speech }),
  enqueueSpeech: (message) => {
    const { speech, speechQueue } = get();
    if (!speech) {
      set({ speech: message, mode: "talk" });
      return;
    }
    set({ speechQueue: [...speechQueue, message] });
  },
  dequeueSpeech: () => {
    const { speechQueue } = get();
    if (speechQueue.length === 0) {
      set({ speech: null, mode: "idle" });
      return;
    }
    const [next, ...rest] = speechQueue;
    set({ speech: next, speechQueue: rest, mode: "talk" });
  },
  openMenu: (menuPosition) => set({ menuOpen: true, menuPosition }),
  closeMenu: () => set({ menuOpen: false }),
  showAchievementToast: (achievement) => set({ achievementToast: achievement }),
  clearAchievementToast: () => set({ achievementToast: null }),
}));
