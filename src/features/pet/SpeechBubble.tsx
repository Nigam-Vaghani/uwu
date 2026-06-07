import { useEffect, useMemo, useState } from "react";
import type { SpeechMessage } from "./pet.types";

const FADE_MS = 300;
const MIN_SPEECH_DURATION_MS = 4500;
const MAX_SPEECH_DURATION_MS = 18000;
const MS_PER_CHAR = 55;

function speechDurationMs(message: SpeechMessage): number {
  const length = message.text.length + (message.detail?.length ?? 0);
  return Math.min(MAX_SPEECH_DURATION_MS, Math.max(MIN_SPEECH_DURATION_MS, length * MS_PER_CHAR));
}

type SpeechBubbleProps = {
  message: SpeechMessage | null;
  onDismiss: () => void;
};

export function SpeechBubble({ message, onDismiss }: SpeechBubbleProps) {
  const [visible, setVisible] = useState(false);
  const [displayMessage, setDisplayMessage] = useState<SpeechMessage | null>(null);

  const durationMs = useMemo(
    () => (displayMessage ? speechDurationMs(displayMessage) : MIN_SPEECH_DURATION_MS),
    [displayMessage],
  );

  useEffect(() => {
    if (!message) {
      setVisible(false);
      const timeout = window.setTimeout(() => setDisplayMessage(null), FADE_MS);
      return () => window.clearTimeout(timeout);
    }

    setDisplayMessage(message);
    const showFrame = window.requestAnimationFrame(() => setVisible(true));
    const dismissTimer = window.setTimeout(() => {
      setVisible(false);
      window.setTimeout(onDismiss, FADE_MS);
    }, durationMs);

    return () => {
      window.cancelAnimationFrame(showFrame);
      window.clearTimeout(dismissTimer);
    };
  }, [durationMs, message, onDismiss]);

  if (!displayMessage) {
    return null;
  }

  return (
    <aside
      className={`speech-bubble${visible ? " speech-bubble--visible" : " speech-bubble--hidden"}`}
      aria-live="polite"
    >
      <span>{displayMessage.text}</span>
      {displayMessage.detail ? <small>{displayMessage.detail}</small> : null}
    </aside>
  );
}
