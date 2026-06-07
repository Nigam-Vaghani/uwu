import { useEffect, useState } from "react";
import type { AppConfig } from "../../store/settingsStore";
import { invokeCommand } from "../../hooks/useTauriStore";
import type { PersonalityInfo } from "../customization/customization.types";
import {
  DEFAULT_OPACITY,
  MAX_OPACITY,
  MIN_OPACITY,
  PERSONALITY_OPTIONS,
} from "./settings.types";

type GeneralSettingsProps = {
  config: AppConfig;
  onPatch: (patch: Partial<AppConfig>) => void;
};

export function GeneralSettings({ config, onPatch }: GeneralSettingsProps) {
  const [personalities, setPersonalities] = useState<PersonalityInfo[]>([]);

  useEffect(() => {
    void invokeCommand("list_personalities", {})
      .then(setPersonalities)
      .catch(() => setPersonalities([]));
  }, []);

  const options =
    personalities.length > 0
      ? personalities.map((item) => item.id)
      : [...PERSONALITY_OPTIONS];

  const personality = options.includes(config.personality) ? config.personality : options[0];

  return (
    <div className="settings-section">
      <h2>General</h2>

      <label className="settings-field">
        <span>Pet Name</span>
        <input
          type="text"
          value={config.petName}
          data-element-id="pet-name"
          onChange={(event) => onPatch({ petName: event.currentTarget.value })}
          placeholder="Uwu-chan"
        />
      </label>

      <label className="settings-field">
        <span>Personality</span>
        <select
          value={personality}
          data-element-id="personality-select"
          onChange={(event) => onPatch({ personality: event.currentTarget.value })}
        >
          {options.map((option) => (
            <option key={option} value={option}>
              {option}
            </option>
          ))}
        </select>
      </label>

      <label className="settings-field">
        <span>Pet Opacity</span>
        <div className="settings-row">
          <input
            type="range"
            min={Math.round(MIN_OPACITY * 100)}
            max={Math.round(MAX_OPACITY * 100)}
            value={Math.round(config.opacity * 100)}
            data-element-id="opacity-slider"
            onChange={(event) =>
              onPatch({ opacity: Number(event.currentTarget.value) / 100 })
            }
          />
          <span>{Math.round(config.opacity * 100)}%</span>
        </div>
      </label>

      <p className="settings-hint">
        Default opacity is {Math.round(DEFAULT_OPACITY * 100)}%. Changes apply after saving.
      </p>
    </div>
  );
}
