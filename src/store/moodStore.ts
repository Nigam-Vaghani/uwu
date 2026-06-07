import { create } from "zustand";
import type { MoodState, PetMood } from "../features/mood/mood.types";

type MoodStore = {
  moodState: MoodState;
  setMoodState: (state: MoodState) => void;
  setMood: (mood: PetMood) => void;
};

const defaultMoodState: MoodState = {
  mood: "happy",
  lastInteraction: Math.floor(Date.now() / 1000),
  excitementUntil: null,
};

export const useMoodStore = create<MoodStore>((set) => ({
  moodState: defaultMoodState,
  setMoodState: (moodState) => set({ moodState }),
  setMood: (mood) =>
    set((state) => ({
      moodState: { ...state.moodState, mood },
    })),
}));
