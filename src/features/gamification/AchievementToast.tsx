import { useEffect, useState } from "react";
import type { AchievementToastPayload } from "./gamification.types";

type AchievementToastProps = {
  achievement: AchievementToastPayload | null;
  onDismiss: () => void;
};

const TOAST_DURATION_MS = 4500;

export function AchievementToast({ achievement, onDismiss }: AchievementToastProps) {
  const [visible, setVisible] = useState(false);
  const [display, setDisplay] = useState<AchievementToastPayload | null>(null);

  useEffect(() => {
    if (!achievement) {
      setVisible(false);
      const timeout = window.setTimeout(() => setDisplay(null), 300);
      return () => window.clearTimeout(timeout);
    }

    setDisplay(achievement);
    const showFrame = window.requestAnimationFrame(() => setVisible(true));
    const dismissTimer = window.setTimeout(() => {
      setVisible(false);
      window.setTimeout(onDismiss, 300);
    }, TOAST_DURATION_MS);

    return () => {
      window.cancelAnimationFrame(showFrame);
      window.clearTimeout(dismissTimer);
    };
  }, [achievement, onDismiss]);

  if (!display) {
    return null;
  }

  return (
    <div className={`achievement-toast${visible ? " achievement-toast--visible" : ""}`}>
      <div className="achievement-toast-icon">{display.icon}</div>
      <div>
        <strong>Achievement Unlocked</strong>
        <p>{display.name}</p>
      </div>
    </div>
  );
}
