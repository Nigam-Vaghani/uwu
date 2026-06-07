export type PetAnimation = "idle" | "walk" | "sleep" | "talk" | "celebrate";

export type AnimationState = PetAnimation;

export type SpeechSource = "reminder" | "ai" | "system" | "welcome" | "interaction" | "quote";

export type SpeechMessage = {
  text: string;
  tone?: "warm" | "curious" | "sleepy" | "bright";
  detail?: string;
  source?: SpeechSource;
  priority?: number;
  reminderId?: number;
  reminderCategory?: string;
};

export type PetState = {
  animation: AnimationState;
  speechQueue: SpeechMessage[];
  activeSpeech: SpeechMessage | null;
};
