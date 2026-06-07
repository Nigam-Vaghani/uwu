import { useCallback, useEffect } from "react";
import { invokeCommand } from "../../hooks/useTauriStore";
import { useMoodStore } from "../../store/moodStore";
import type { MoodState, PetMood } from "./mood.types";

function normalizeMoodState(raw: {
  mood: string;
  lastInteraction: number;
  excitementUntil: number | null;
}): MoodState {
  const mood = raw.mood.toLowerCase() as PetMood;
  return {
    mood: ["happy", "neutral", "sad", "excited", "sleepy"].includes(mood)
      ? mood
      : "happy",
    lastInteraction: raw.lastInteraction,
    excitementUntil: raw.excitementUntil,
  };
}

export function useMood() {
  const moodState = useMoodStore((state) => state.moodState);
  const setMoodState = useMoodStore((state) => state.setMoodState);

  const refreshMood = useCallback(async () => {
    const next = await invokeCommand("reconcile_mood", {});
    setMoodState(normalizeMoodState(next));
  }, [setMoodState]);

  const recordInteraction = useCallback(async () => {
    await invokeCommand("record_interaction", {});
    await refreshMood();
  }, [refreshMood]);

  useEffect(() => {
    void refreshMood();
    const interval = window.setInterval(() => {
      void refreshMood();
    }, 60_000);

    return () => window.clearInterval(interval);
  }, [refreshMood]);

  return {
    mood: moodState.mood,
    moodState,
    refreshMood,
    recordInteraction,
  };
}
