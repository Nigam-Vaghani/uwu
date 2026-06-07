import {
  AI_PROVIDER_OPTIONS,
  defaultModelForProvider,
  providerRequiresApiKey,
} from "./ai.types";
import { QuoteSettings } from "../quotes/QuoteSettings";
import { useAiConfig } from "./useAiConfig";

export function AISettings() {
  const {
    config,
    apiKey,
    setApiKey,
    patchConfig,
    saveConfig,
    testConnection,
    loading,
    saving,
    testing,
    testResult,
    error,
  } = useAiConfig();

  const handleProviderChange = (provider: typeof config.provider) => {
    patchConfig({
      provider,
      model: defaultModelForProvider(provider) || config.model,
    });
  };

  const handleSave = async () => {
    await saveConfig();
  };

  const handleTest = async () => {
    await testConnection();
  };

  const showApiKey = providerRequiresApiKey(config.provider);
  const showOllamaUrl = config.provider === "ollama";
  const statusClass = testResult
    ? testResult.ok
      ? "ai-status ai-status--ok"
      : "ai-status ai-status--err"
    : "ai-status";

  return (
    <div className="settings-section">
      <h2>AI Settings</h2>

      <label className="settings-field">
        <span>AI Provider</span>
        <select
          value={config.provider}
          data-element-id="ai-provider"
          disabled={loading || saving}
          onChange={(event) =>
            handleProviderChange(event.currentTarget.value as typeof config.provider)
          }
        >
          {AI_PROVIDER_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </label>

      {showApiKey ? (
        <label className="settings-field">
          <span>API Key</span>
          <input
            type="password"
            value={apiKey}
            placeholder={config.hasApiKey ? "Saved key — enter to replace" : "gsk_..."}
            data-element-id="api-key"
            disabled={loading || saving}
            onChange={(event) => setApiKey(event.currentTarget.value)}
          />
          <p className="settings-hint">
            Your key is stored locally and never sent anywhere except the provider.
          </p>
        </label>
      ) : null}

      {config.provider !== "none" ? (
        <label className="settings-field">
          <span>Model</span>
          <input
            type="text"
            value={config.model}
            data-element-id="model-name"
            disabled={loading || saving}
            onChange={(event) => patchConfig({ model: event.currentTarget.value })}
          />
        </label>
      ) : null}

      {showOllamaUrl ? (
        <label className="settings-field">
          <span>Ollama Base URL</span>
          <input
            type="text"
            value={config.ollamaBaseUrl}
            data-element-id="ollama-url"
            disabled={loading || saving}
            onChange={(event) => patchConfig({ ollamaBaseUrl: event.currentTarget.value })}
          />
        </label>
      ) : null}

      {error ? <p className="settings-error">{error}</p> : null}

      <div className="ai-settings-actions">
        <button
          className="settings-primary"
          type="button"
          data-element-id="test-btn"
          onClick={handleTest}
          disabled={loading || saving || testing}
        >
          {testing ? "Testing..." : "Test Connection"}
        </button>
        <button
          className="settings-primary"
          type="button"
          data-element-id="ai-save-btn"
          onClick={handleSave}
          disabled={loading || saving || testing}
        >
          {saving ? "Saving..." : "Save AI Settings"}
        </button>
      </div>

      {testResult ? (
        <div className={statusClass} data-element-id="status-msg">
          {testResult.ok ? "✓" : "✗"} {testResult.message}
        </div>
      ) : null}

      <QuoteSettings />
    </div>
  );
}
