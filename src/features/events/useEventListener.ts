import { useEffect } from "react";

import { listen } from "@tauri-apps/api/event";

import {

  isPermissionGranted,

  requestPermission,

  sendNotification,

} from "@tauri-apps/plugin-notification";

import { usePetStore } from "../../store/petStore";

import { useSettingsStore } from "../../store/settingsStore";

import { useMoodStore } from "../../store/moodStore";

import { invokeCommand } from "../../hooks/useTauriStore";

import { isAiEnabledEvent, toAiPayload } from "../ai/aiEvents";
import type { SpeechSource } from "../pet/pet.types";

import {

  loadPersonality,

  PET_EVENT,

  resolvePetMessage,

  type PetEventPayload,

} from "./eventEngine";

function resolveSpeechSource(
  eventType: PetEventPayload["eventType"],
  usedAi: boolean,
): SpeechSource {
  if (eventType === "ReminderFired") {
    return "reminder";
  }
  if (usedAi) {
    return "ai";
  }
  if (eventType === "UserInteraction") {
    return "interaction";
  }
  return "system";
}

function resolveSpeechPriority(eventType: PetEventPayload["eventType"]): number {
  if (eventType === "CriticalBattery") {
    return 10;
  }
  if (eventType === "LowBattery") {
    return 5;
  }
  return 0;
}



async function ensureNotificationPermission() {

  let granted = await isPermissionGranted();

  if (!granted) {

    const permission = await requestPermission();

    granted = permission === "granted";

  }

  return granted;

}



export function useEventListener() {

  const enqueueSpeech = usePetStore((state) => state.enqueueSpeech);
  const setMode = usePetStore((state) => state.setMode);
  const showAchievementToast = usePetStore((state) => state.showAchievementToast);

  const config = useSettingsStore((state) => state.config);

  const setMood = useMoodStore((state) => state.setMood);



  useEffect(() => {

    let active = true;



    const unlistenPromise = listen<PetEventPayload>(PET_EVENT, async (event) => {

      if (!active) {

        return;

      }



      const payload = event.payload;

      const currentMood = useMoodStore.getState().moodState.mood;

      const profile = await loadPersonality(config.personality);

      const resolved = resolvePetMessage(profile, payload, currentMood);



      if (payload.eventType === "IdleDetected") {

        setMood(

          currentMood === "happy" ? "neutral" : currentMood === "neutral" ? "sad" : currentMood,

        );

      }



      if (payload.eventType === "ReminderFired") {

        setMood("excited");

        const canNotify = await ensureNotificationPermission();

        if (canNotify) {

          sendNotification({

            title: payload.title ?? "UWU Companion Reminder",

            body: resolved.text,

          });

        }

      }



      if (payload.eventType === "LevelUp") {
        setMood("excited");
        setMode("celebrate");
        window.setTimeout(() => {
          if (usePetStore.getState().mode === "celebrate") {
            setMode("idle");
          }
        }, 4000);
      }



      if (payload.eventType === "AchievementUnlocked") {
        setMood("excited");
        setMode("celebrate");
        showAchievementToast({
          id: payload.category ?? "achievement",
          name: payload.title ?? "Achievement unlocked",
          icon: payload.message ?? "🏆",
        });

        const canNotify = await ensureNotificationPermission();
        if (canNotify) {
          sendNotification({
            title: "Achievement Unlocked",
            body: payload.title ?? "You earned a new achievement!",
          });
        }

        window.setTimeout(() => {
          if (usePetStore.getState().mode === "celebrate") {
            setMode("idle");
          }
        }, 4000);
      }



      if (payload.eventType === "ObjectiveCompleted" || payload.eventType === "StreakMilestone") {
        setMood("excited");
      }



      if (

        payload.eventType === "LowBattery" ||

        payload.eventType === "CriticalBattery"

      ) {

        const canNotify = await ensureNotificationPermission();

        if (canNotify) {

          sendNotification({

            title: "UWU Companion",

            body: resolved.text,

          });

        }

      }



      if (payload.eventType === "CodingSessionStarted") {

        setMood("excited");

      }



      let speechText = resolved.text;
      let usedAi = false;

      if (isAiEnabledEvent(payload.eventType)) {
        try {
          const aiResponse = await invokeCommand("ai_complete", {
            payload: toAiPayload(payload),
          });
          if (aiResponse.text.trim()) {
            speechText = aiResponse.text;
            usedAi = true;
          }
        } catch {
          // Keep personality template fallback text.
        }
      }

      enqueueSpeech({
        text: speechText,
        tone: resolved.tone,
        detail: resolved.detail,
        source: resolveSpeechSource(payload.eventType, usedAi),
        priority: resolveSpeechPriority(payload.eventType),
        reminderId: payload.reminderId ?? undefined,
        reminderCategory: payload.category ?? undefined,
      });

    });



    return () => {

      active = false;

      void unlistenPromise.then((unlisten) => unlisten());

    };

  }, [config.personality, enqueueSpeech, setMood, setMode, showAchievementToast]);

}

