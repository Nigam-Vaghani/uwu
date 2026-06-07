export type PetMood = "happy" | "neutral" | "sad" | "excited" | "sleepy";

export type MoodState = {
  mood: PetMood;
  lastInteraction: number;
  excitementUntil: number | null;
};

export const MOOD_LABELS: Record<PetMood, string> = {
  happy: "Happy",
  neutral: "Neutral",
  sad: "Sad",
  excited: "Excited",
  sleepy: "Sleepy",
};

export const MOOD_EMOJI: Record<PetMood, string> = {
  happy: "😊",
  neutral: "😐",
  sad: "😢",
  excited: "🤩",
  sleepy: "😴",
};
