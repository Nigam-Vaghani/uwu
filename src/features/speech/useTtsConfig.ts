import { useCallback, useEffect, useRef, useState } from "react";
import { invokeCommand } from "../../hooks/useTauriStore";
import type { TtsConfig, VoiceInfo } from "./speech.types";
import { DEFAULT_TTS_CONFIG } from "./speech.types";

export function useTtsConfig() {
  const [config, setConfig] = useState<TtsConfig>(DEFAULT_TTS_CONFIG);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadConfig = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const loaded = await invokeCommand("get_tts_config", {});
      setConfig(loaded);
      return loaded;
    } catch (loadError) {
      const message = loadError instanceof Error ? loadError.message : String(loadError);
      setError(message);
      throw loadError;
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadConfig();
  }, [loadConfig]);

  const patchConfig = useCallback((patch: Partial<TtsConfig>) => {
    setConfig((current) => ({ ...current, ...patch }));
  }, []);

  const saveConfig = useCallback(async (nextConfig: TtsConfig = config) => {
    setSaving(true);
    setError(null);
    try {
      const saved = await invokeCommand("set_tts_config", {
        input: {
          provider: nextConfig.provider,
          voiceId: nextConfig.voiceId,
          rate: nextConfig.rate,
          pitch: nextConfig.pitch,
          volume: nextConfig.volume,
          speakOnReminder: nextConfig.speakOnReminder,
          speakOnAi: nextConfig.speakOnAi,
          piperModelPath: nextConfig.piperModelPath,
        },
      });
      setConfig(saved);
      return saved;
    } catch (saveError) {
      const message = saveError instanceof Error ? saveError.message : String(saveError);
      setError(message);
      throw saveError;
    } finally {
      setSaving(false);
    }
  }, [config]);

  const listVoices = useCallback(async (provider: TtsConfig["provider"]) => {
    if (provider === "webSpeech") {
      return listWebSpeechVoices();
    }
    if (provider === "silent") {
      return [] as VoiceInfo[];
    }
    return invokeCommand("list_voices", { provider });
  }, []);

  return {
    config,
    setConfig,
    patchConfig,
    loadConfig,
    saveConfig,
    listVoices,
    loading,
    saving,
    error,
  };
}

export function listWebSpeechVoices(): VoiceInfo[] {
  if (typeof window === "undefined" || !window.speechSynthesis) {
    return [];
  }

  return window.speechSynthesis.getVoices().map((voice) => ({
    id: voice.voiceURI || voice.name,
    name: voice.name,
    language: voice.lang,
  }));
}

export function waitForWebSpeechVoices(): Promise<VoiceInfo[]> {
  return new Promise((resolve) => {
    const voices = listWebSpeechVoices();
    if (voices.length > 0) {
      resolve(voices);
      return;
    }

    const handleChange = () => {
      const loaded = listWebSpeechVoices();
      if (loaded.length > 0) {
        window.speechSynthesis.removeEventListener("voiceschanged", handleChange);
        resolve(loaded);
      }
    };

    window.speechSynthesis.addEventListener("voiceschanged", handleChange);
    window.setTimeout(() => {
      window.speechSynthesis.removeEventListener("voiceschanged", handleChange);
      resolve(listWebSpeechVoices());
    }, 1000);
  });
}

export function useTtsConfigRef() {
  const configRef = useRef<TtsConfig>(DEFAULT_TTS_CONFIG);

  useEffect(() => {
    void invokeCommand("get_tts_config", {}).then((loaded) => {
      configRef.current = loaded;
    });
  }, []);

  return configRef;
}
