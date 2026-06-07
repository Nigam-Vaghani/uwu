export type Personality = "Cheerful" | "Calm" | "Sassy" | "Motivational";

export type SettingsFormValues = {
  petName: string;
  personality: Personality;
  opacity: number;
};

export const PERSONALITY_OPTIONS: Personality[] = [
  "Cheerful",
  "Calm",
  "Sassy",
  "Motivational",
];

export const DEFAULT_OPACITY = 0.9;
export const MIN_OPACITY = 0.3;
export const MAX_OPACITY = 1;
