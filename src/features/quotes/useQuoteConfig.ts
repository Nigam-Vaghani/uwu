import { useCallback, useEffect, useState } from "react";
import { invokeCommand } from "../../hooks/useTauriStore";
import type { QuoteConfig } from "./quote.types";
import { DEFAULT_QUOTE_CONFIG } from "./quote.types";

export function useQuoteConfig() {
  const [config, setConfig] = useState<QuoteConfig>(DEFAULT_QUOTE_CONFIG);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [previewing, setPreviewing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [previewText, setPreviewText] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const loaded = await invokeCommand("get_quote_config", {});
      setConfig(loaded);
    } catch (loadError) {
      setError(loadError instanceof Error ? loadError.message : String(loadError));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const patchConfig = useCallback((patch: Partial<QuoteConfig>) => {
    setConfig((current) => ({ ...current, ...patch }));
  }, []);

  const saveConfig = useCallback(async () => {
    setSaving(true);
    setError(null);
    try {
      const saved = await invokeCommand("set_quote_config", { config });
      setConfig(saved);
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : String(saveError));
      throw saveError;
    } finally {
      setSaving(false);
    }
  }, [config]);

  const previewQuote = useCallback(async () => {
    setPreviewing(true);
    setError(null);
    setPreviewText(null);
    try {
      const result = await invokeCommand("preview_quote", {
        quoteType: config.selectedType,
      });
      setPreviewText(result.text);
    } catch (previewError) {
      setError(previewError instanceof Error ? previewError.message : String(previewError));
    } finally {
      setPreviewing(false);
    }
  }, [config.selectedType]);

  const addCustomType = useCallback((value: string) => {
    const trimmed = value.trim();
    if (!trimmed) {
      return;
    }
    setConfig((current) => {
      if (current.customTypes.includes(trimmed)) {
        return { ...current, selectedType: trimmed };
      }
      return {
        ...current,
        customTypes: [...current.customTypes, trimmed],
        selectedType: trimmed,
      };
    });
  }, []);

  return {
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
    refresh,
  };
}
