import { MOOD_EMOJI, MOOD_LABELS, type PetMood } from "./mood.types";

type MoodIndicatorProps = {
  mood: PetMood;
};

export function MoodIndicator({ mood }: MoodIndicatorProps) {
  return (
    <div className="mood-indicator" title={`Mood: ${MOOD_LABELS[mood]}`} aria-label={`Mood: ${MOOD_LABELS[mood]}`}>
      <span aria-hidden="true">{MOOD_EMOJI[mood]}</span>
    </div>
  );
}
