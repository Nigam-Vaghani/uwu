import { useEffect, useRef } from "react";
import { usePetStore } from "../../store/petStore";
import { useMoodStore } from "../../store/moodStore";
import type { PetAnimation } from "./pet.types";

const INACTIVITY_TO_SLEEP_MS = 45000;

export function usePetAnimation() {
  const mode = usePetStore((state) => state.mode);
  const speech = usePetStore((state) => state.speech);
  const mood = useMoodStore((state) => state.moodState.mood);
  const setMode = usePetStore((state) => state.setMode);
  const lastInteractionRef = useRef(Date.now());

  useEffect(() => {
    if (speech) {
      setMode("talk");
      return undefined;
    }

    if (mode === "talk") {
      setMode("idle");
    }

    return undefined;
  }, [mode, setMode, speech]);

  useEffect(() => {
    if (mode !== "celebrate") {
      return undefined;
    }

    const timer = window.setTimeout(() => {
      if (usePetStore.getState().mode === "celebrate" && !usePetStore.getState().speech) {
        setMode("idle");
      }
    }, 4000);

    return () => window.clearTimeout(timer);
  }, [mode, setMode]);

  useEffect(() => {
    const markInteraction = () => {
      lastInteractionRef.current = Date.now();
      if (usePetStore.getState().mode === "sleep") {
        setMode("idle");
      }
    };

    window.addEventListener("mousedown", markInteraction);
    window.addEventListener("keydown", markInteraction);

    return () => {
      window.removeEventListener("mousedown", markInteraction);
      window.removeEventListener("keydown", markInteraction);
    };
  }, [setMode]);

  useEffect(() => {
    if (speech || mode === "talk" || mode === "celebrate") {
      return undefined;
    }

    if (mood === "sleepy" && mode === "idle") {
      setMode("sleep");
      return undefined;
    }

    if (mood === "excited" && mode === "idle") {
      return undefined;
    }

    const sleepThreshold =
      mood === "sleepy" ? INACTIVITY_TO_SLEEP_MS / 3 : INACTIVITY_TO_SLEEP_MS;

    const sleepTimer = window.setInterval(() => {
      const currentMode = usePetStore.getState().mode;
      if (currentMode !== "idle") {
        return;
      }

      if (Date.now() - lastInteractionRef.current >= sleepThreshold) {
        setMode("sleep");
      }
    }, 2000);

    return () => window.clearInterval(sleepTimer);
  }, [mode, mood, setMode, speech]);

  return {
    animation: mode as PetAnimation,
    setAnimation: setMode,
  };
}
