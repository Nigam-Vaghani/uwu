import { useState } from "react";
import { BUILTIN_QUOTE_TYPES } from "./quote.types";
import { useQuoteConfig } from "./useQuoteConfig";

export function QuoteSettings() {
  const {
    config,
    loading,
    saving,
    previewing,
    error,
    previewText,
    patchConfig,
    saveConfig,
    previewQuote,
    addCustomType,
  } = useQuoteConfig();
  const [customTypeInput, setCustomTypeInput] = useState("");

  const customOnly = config.customTypes.filter(
    (type) => !(BUILTIN_QUOTE_TYPES as readonly string[]).includes(type),
  );
  const allTypes = [...BUILTIN_QUOTE_TYPES, ...customOnly];

  const handleAddCustomType = () => {
    addCustomType(customTypeInput);
    setCustomTypeInput("");
  };

  return (
    <div className="settings-section quote-settings">
      <h2>Timed Quotes</h2>
      <p className="settings-hint">
        Uses your Groq API key (from AI settings or the <code>GROQ_API_KEY</code> environment variable)
        to generate a fresh short quote on a schedule.
      </p>

      <label className="settings-toggle-field">
        <span>Enable timed quotes</span>
        <button
          type="button"
          className={`settings-toggle${config.enabled ? " settings-toggle--on" : ""}`}
          disabled={loading || saving}
          aria-pressed={config.enabled}
          onClick={() => patchConfig({ enabled: !config.enabled })}
        />
      </label>

      <label className="settings-field">
        <span>Interval (minutes)</span>
        <input
          type="number"
          min={1}
          max={1440}
          value={config.intervalMinutes}
          disabled={loading || saving}
          onChange={(event) =>
            patchConfig({ intervalMinutes: Math.max(1, Number(event.currentTarget.value) || 1) })
          }
        />
      </label>

      <label className="settings-field">
        <span>Quote style</span>
        <select
          value={config.selectedType}
          disabled={loading || saving}
          onChange={(event) => patchConfig({ selectedType: event.currentTarget.value })}
        >
          {allTypes.map((type) => (
            <option key={type} value={type}>
              {type.charAt(0).toUpperCase() + type.slice(1)}
            </option>
          ))}
        </select>
      </label>

      <div className="settings-field">
        <span>Add custom quote style</span>
        <div className="settings-row">
          <input
            type="text"
            value={customTypeInput}
            placeholder="e.g. stoic, anime, dad-joke"
            disabled={loading || saving}
            onChange={(event) => setCustomTypeInput(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                handleAddCustomType();
              }
            }}
          />
          <button
            type="button"
            className="settings-secondary"
            disabled={loading || saving || !customTypeInput.trim()}
            onClick={handleAddCustomType}
          >
            Add
          </button>
        </div>
      </div>

      {error ? <p className="settings-error">{error}</p> : null}
      {previewText ? <p className="quote-preview">Preview: “{previewText}”</p> : null}

      <div className="ai-settings-actions">
        <button
          type="button"
          className="settings-secondary"
          disabled={loading || saving || previewing}
          onClick={() => void previewQuote()}
        >
          {previewing ? "Generating..." : "Preview Quote"}
        </button>
        <button
          type="button"
          className="settings-primary"
          disabled={loading || saving || previewing}
          onClick={() => void saveConfig()}
        >
          {saving ? "Saving..." : "Save Quote Settings"}
        </button>
      </div>
    </div>
  );
}
