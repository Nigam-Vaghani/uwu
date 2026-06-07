import { useCallback, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { useSettingsStore } from "../../store/settingsStore";
import type { PetEventPayload } from "../events/eventEngine";
import type { SoundPackFileManifest } from "./customization.types";

const EVENT_SOUND_MAP: Partial<Record<PetEventPayload["eventType"], keyof SoundPackFileManifest["sounds"]>> = {
  ReminderFired: "notification",
  UserInteraction: "reaction",
  LevelUp: "levelUp",
  AchievementUnlocked: "levelUp",
  ObjectiveCompleted: "reaction",
  StreakMilestone: "levelUp",
};

export function useSoundEffects() {
  const activeSoundPack = useSettingsStore((state) => state.config.activeSoundPack);
  const manifestRef = useRef<SoundPackFileManifest | null>(null);
  const audioCacheRef = useRef<Map<string, HTMLAudioElement>>(new Map());

  const loadManifest = useCallback(async (packId: string) => {
    try {
      const response = await fetch(`/sounds/${packId}/manifest.json`);
      if (!response.ok) {
        manifestRef.current = null;
        return;
      }
      manifestRef.current = (await response.json()) as SoundPackFileManifest;
      audioCacheRef.current.clear();
    } catch {
      manifestRef.current = null;
    }
  }, []);

  const playSound = useCallback((soundKey: string) => {
    const manifest = manifestRef.current;
    if (!manifest) {
      return;
    }

    const fileName = manifest.sounds[soundKey];
    if (!fileName) {
      return;
    }

    const cacheKey = `${manifest.id}/${fileName}`;
    let audio = audioCacheRef.current.get(cacheKey);
    if (!audio) {
      audio = new Audio(`/sounds/${manifest.id}/${fileName}`);
      audio.volume = 0.5;
      audioCacheRef.current.set(cacheKey, audio);
    }

    audio.currentTime = 0;
    void audio.play().catch(() => undefined);
  }, []);

  useEffect(() => {
    void loadManifest(activeSoundPack);
  }, [activeSoundPack, loadManifest]);

  useEffect(() => {
    const unlistenPack = listen<string>("customization:sound-pack-changed", (event) => {
      void loadManifest(event.payload);
    });

    const unlistenPet = listen<PetEventPayload>("pet:event", (event) => {
      const soundKey = EVENT_SOUND_MAP[event.payload.eventType];
      if (soundKey) {
        playSound(soundKey);
      }
    });

    return () => {
      void unlistenPack.then((release) => release());
      void unlistenPet.then((release) => release());
    };
  }, [loadManifest, playSound]);

  return { playSound };
}
