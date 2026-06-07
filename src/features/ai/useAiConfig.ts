import { useCallback, useEffect, useState } from "react";
import { invokeCommand } from "../../hooks/useTauriStore";
import type {
  AiConfig,
  AiConnectionTestResult,
  SetAiConfigInput,
} from "./ai.types";
import { DEFAULT_AI_CONFIG } from "./ai.types";

export function useAiConfig() {
  const [config, setConfig] = useState<AiConfig>(DEFAULT_AI_CONFIG);
  const [apiKey, setApiKey] = useState("");
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<AiConnectionTestResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadConfig = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const loaded = await invokeCommand("get_ai_config", {});
      setConfig(loaded);
      setApiKey("");
    } catch (loadError) {
      const message = loadError instanceof Error ? loadError.message : String(loadError);
      setError(message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadConfig();
  }, [loadConfig]);

  const patchConfig = useCallback((patch: Partial<AiConfig>) => {
    setConfig((current) => ({ ...current, ...patch }));
    setTestResult(null);
  }, []);

  const saveConfig = useCallback(async () => {
    setSaving(true);
    setError(null);
    try {
      const input: SetAiConfigInput = {
        provider: config.provider,
        model: config.model,
        ollamaBaseUrl: config.ollamaBaseUrl,
      };
      if (apiKey.trim()) {
        input.apiKey = apiKey.trim();
      }
      const saved = await invokeCommand("set_ai_config", { input });
      setConfig(saved);
      setApiKey("");
      return saved;
    } catch (saveError) {
      const message = saveError instanceof Error ? saveError.message : String(saveError);
      setError(message);
      throw saveError;
    } finally {
      setSaving(false);
    }
  }, [apiKey, config.model, config.ollamaBaseUrl, config.provider]);

  const testConnection = useCallback(async () => {
    setTesting(true);
    setError(null);
    setTestResult(null);
    try {
      if (apiKey.trim()) {
        await invokeCommand("set_ai_config", {
          input: {
            provider: config.provider,
            model: config.model,
            ollamaBaseUrl: config.ollamaBaseUrl,
            apiKey: apiKey.trim(),
          },
        });
      }
      const result = await invokeCommand("test_ai_connection", {});
      setTestResult(result);
      return result;
    } catch (testError) {
      const message = testError instanceof Error ? testError.message : String(testError);
      setError(message);
      throw testError;
    } finally {
      setTesting(false);
    }
  }, [apiKey, config.model, config.ollamaBaseUrl, config.provider]);

  return {
    config,
    apiKey,
    setApiKey,
    patchConfig,
    loadConfig,
    saveConfig,
    testConnection,
    loading,
    saving,
    testing,
    testResult,
    error,
  };
}
