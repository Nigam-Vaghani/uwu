import type { PetMood } from "../mood/mood.types";
import type { ReminderCategory } from "../reminders/reminders.types";

export type PetEventType =
  | "IdleDetected"
  | "ReminderFired"
  | "RandomMessage"
  | "UserInteraction"
  | "HighCpu"
  | "HighRam"
  | "LowBattery"
  | "CriticalBattery"
  | "CodingSessionStarted"
  | "CodingSessionEnded"
  | "LevelUp"
  | "AchievementUnlocked"
  | "ObjectiveCompleted"
  | "StreakMilestone";

export type PetEventPayload = {
  eventType: PetEventType;
  title?: string | null;
  message?: string | null;
  category?: string | null;
  reminderId?: number | null;
  value?: number | null;
  appName?: string | null;
};

type MoodTemplates = Record<PetMood, string[]>;
type ReminderTemplates = Record<ReminderCategory, string[]>;
type SystemTemplates = {
  highCpu: string[];
  highRam: string[];
  lowBattery: string[];
  criticalBattery: string[];
  codingSessionStarted: string[];
  codingSessionEnded: string[];
};

export type PersonalityProfile = {
  id: string;
  name: string;
  tone: string;
  messageStyle: string;
  templates: {
    idle: MoodTemplates;
    random: MoodTemplates;
    reminder: ReminderTemplates;
    interaction: MoodTemplates;
    system: SystemTemplates;
  };
};

const personalityCache = new Map<string, PersonalityProfile>();

function personalityFileName(personality: string) {
  return personality.trim().toLowerCase();
}

export async function loadPersonality(personality: string): Promise<PersonalityProfile> {
  const key = personalityFileName(personality);
  const cached = personalityCache.get(key);
  if (cached) {
    return cached;
  }

  const response = await fetch(`/assets/personalities/${key}.json`);
  if (!response.ok) {
    throw new Error(`Failed to load personality: ${personality}`);
  }

  const profile = (await response.json()) as PersonalityProfile;
  personalityCache.set(key, profile);
  return profile;
}

function pickRandom(items: string[]) {
  return items[Math.floor(Math.random() * items.length)] ?? items[0] ?? "";
}

function applyTemplate(
  template: string,
  values: Record<string, string>,
) {
  return Object.entries(values).reduce(
    (text, [key, value]) => text.replace(new RegExp(`\\{${key}\\}`, "g"), value),
    template,
  );
}

function moodTone(mood: PetMood): "warm" | "curious" | "sleepy" | "bright" {
  switch (mood) {
    case "sleepy":
      return "sleepy";
    case "sad":
      return "warm";
    case "excited":
      return "bright";
    case "neutral":
      return "curious";
    default:
      return "bright";
  }
}

export function resolvePetMessage(
  profile: PersonalityProfile,
  payload: PetEventPayload,
  mood: PetMood,
): { text: string; tone: "warm" | "curious" | "sleepy" | "bright"; detail?: string } {
  switch (payload.eventType) {
    case "IdleDetected": {
      const text = pickRandom(profile.templates.idle[mood]);
      return { text, tone: moodTone(mood) };
    }
    case "RandomMessage": {
      const text = pickRandom(profile.templates.random[mood]);
      return { text, tone: moodTone(mood) };
    }
    case "UserInteraction": {
      const text = pickRandom(profile.templates.interaction[mood]);
      return { text, tone: moodTone(mood) };
    }
    case "ReminderFired": {
      const category = (payload.category ?? "custom") as ReminderCategory;
      const templates =
        profile.templates.reminder[category] ?? profile.templates.reminder.custom;
      const template = pickRandom(templates);
      const title = payload.title ?? "Reminder";
      const message = payload.message ?? "";
      const text = applyTemplate(template, {
        title,
        message,
      });
      return {
        text,
        tone: "bright",
        detail: category !== "custom" ? title : undefined,
      };
    }
    case "HighCpu": {
      const template = pickRandom(profile.templates.system.highCpu);
      const text = applyTemplate(template, {
        value: String(Math.round(payload.value ?? 0)),
      });
      return { text, tone: "warm", detail: "High CPU" };
    }
    case "HighRam": {
      const template = pickRandom(profile.templates.system.highRam);
      const text = applyTemplate(template, {
        value: String(Math.round(payload.value ?? 0)),
      });
      return { text, tone: "warm", detail: "High RAM" };
    }
    case "LowBattery": {
      const template = pickRandom(profile.templates.system.lowBattery);
      const text = applyTemplate(template, {
        value: String(Math.round(payload.value ?? 0)),
      });
      return { text, tone: "warm", detail: "Low battery" };
    }
    case "CriticalBattery": {
      const template = pickRandom(profile.templates.system.criticalBattery);
      const text = applyTemplate(template, {
        value: String(Math.round(payload.value ?? 0)),
      });
      return { text, tone: "warm", detail: "Critical battery" };
    }
    case "CodingSessionStarted": {
      const template = pickRandom(profile.templates.system.codingSessionStarted);
      const text = applyTemplate(template, {
        app: payload.appName ?? "your editor",
      });
      return { text, tone: "bright", detail: payload.appName ?? undefined };
    }
    case "CodingSessionEnded": {
      const template = pickRandom(profile.templates.system.codingSessionEnded);
      const text = applyTemplate(template, {
        app: payload.appName ?? "your editor",
      });
      return { text, tone: "curious" };
    }
    case "LevelUp": {
      const level = payload.value ? Math.round(payload.value) : 1;
      return {
        text: `Level ${level}! We're leveling up together!`,
        tone: "bright",
        detail: payload.title ?? undefined,
      };
    }
    case "AchievementUnlocked": {
      return {
        text: `Achievement unlocked: ${payload.title ?? "Nice work"}!`,
        tone: "bright",
        detail: payload.message ?? undefined,
      };
    }
    case "ObjectiveCompleted": {
      return {
        text: `Objective complete: ${payload.title ?? "Great job"}!`,
        tone: "bright",
      };
    }
    case "StreakMilestone": {
      const days = payload.value ? Math.round(payload.value) : 0;
      return {
        text: `${days}-day streak! You're on fire!`,
        tone: "bright",
        detail: "Streak milestone",
      };
    }
    default:
      return { text: "...", tone: "curious" };
  }
}

export const PET_EVENT = "pet:event";
