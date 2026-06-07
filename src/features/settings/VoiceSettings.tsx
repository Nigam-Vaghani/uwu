import { useCallback, useEffect, useState } from "react";
import {
  formatPitch,
  formatRate,
  formatVolumePercent,
  pitchFromSlider,
  pitchToSlider,
  rateFromSlider,
  rateToSlider,
  TEST_VOICE_PHRASE,
  TTS_PROVIDER_OPTIONS,
  volumeFromSlider,
  volumeToSlider,
  type TtsConfig,
  type VoiceInfo,
} from "../speech/speech.types";
import { useTtsConfig, waitForWebSpeechVoices } from "../speech/useTtsConfig";
import { useTTS } from "../speech/useTTS";

export function VoiceSettings() {
  const {
    config,
    patchConfig,
    saveConfig,
    listVoices,
    loading,
    saving,
    error,
  } = useTtsConfig();
  const { speak, stop } = useTTS();
  const [voices, setVoices] = useState<VoiceInfo[]>([]);
  const [loadingVoices, setLoadingVoices] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testError, setTestError] = useState<string | null>(null);

  const refreshVoices = useCallback(async (provider: TtsConfig["provider"]) => {
    setLoadingVoices(true);
    try {
      if (provider === "webSpeech") {
        const webVoices = await waitForWebSpeechVoices();
        setVoices(webVoices);
        return webVoices;
      }
      const loaded = await listVoices(provider);
      setVoices(loaded);
      return loaded;
    } finally {
      setLoadingVoices(false);
    }
  }, [listVoices]);

  useEffect(() => {
    void refreshVoices(config.provider);
  }, [config.provider, refreshVoices]);

  useEffect(() => {
    if (config.provider === "silent" || config.provider === "piper") {
      return;
    }
    if (voices.length === 0) {
      return;
    }
    if (config.voiceId && voices.some((voice) => voice.id === config.voiceId)) {
      return;
    }
    patchConfig({ voiceId: voices[0]?.id ?? "" });
  }, [config.provider, config.voiceId, patchConfig, voices]);

  const handleProviderChange = async (provider: TtsConfig["provider"]) => {
    patchConfig({ provider, voiceId: "" });
    await refreshVoices(provider);
  };

  const handleSave = async () => {
    await saveConfig();
  };

  const handleTestVoice = async () => {
    setTesting(true);
    setTestError(null);
    try {
      await stop();
      await saveConfig();
      await speak(TEST_VOICE_PHRASE, config);
    } catch (testVoiceError) {
      const message =
        testVoiceError instanceof Error ? testVoiceError.message : String(testVoiceError);
      setTestError(message);
    } finally {
      setTesting(false);
    }
  };

  const showVoiceSelect = config.provider !== "silent";
  const showPiperModel = config.provider === "piper";

  return (
    <div className="settings-section">
      <h2>Voice Settings</h2>

      <label className="settings-field">
        <span>TTS Provider</span>
        <select
          value={config.provider}
          data-element-id="tts-provider"
          disabled={loading || saving}
          onChange={(event) =>
            void handleProviderChange(event.currentTarget.value as TtsConfig["provider"])
          }
        >
          {TTS_PROVIDER_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </label>

      {showVoiceSelect ? (
        <label className="settings-field">
          <span>Voice</span>
          <select
            value={config.voiceId}
            data-element-id="voice-select"
            disabled={loading || saving || loadingVoices || voices.length === 0}
            onChange={(event) => patchConfig({ voiceId: event.currentTarget.value })}
          >
            {voices.length === 0 ? (
              <option value="">No voices available</option>
            ) : (
              voices.map((voice) => (
                <option key={voice.id} value={voice.id}>
                  {voice.name}
                  {voice.language ? ` (${voice.language})` : ""}
                </option>
              ))
            )}
          </select>
        </label>
      ) : null}

      {showPiperModel ? (
        <label className="settings-field">
          <span>Piper Model Path (.onnx)</span>
          <input
            type="text"
            value={config.piperModelPath}
            data-element-id="piper-model-path"
            disabled={loading || saving}
            placeholder="C:\\path\\to\\en_US-lessac-medium.onnx"
            onChange={(event) => patchConfig({ piperModelPath: event.currentTarget.value })}
          />
          <p className="settings-hint">
            Place piper.exe in src-tauri/binaries/ — see binaries/README.md for setup.
          </p>
        </label>
      ) : null}

      <label className="settings-field">
        <span>Volume</span>
        <div className="settings-row">
          <input
            type="range"
            min={0}
            max={100}
            value={volumeToSlider(config.volume)}
            data-element-id="volume"
            disabled={loading || saving || config.provider === "silent"}
            onChange={(event) =>
              patchConfig({ volume: volumeFromSlider(Number(event.currentTarget.value)) })
            }
          />
          <span className="voice-range-value">{formatVolumePercent(config.volume)}</span>
        </div>
      </label>

      <label className="settings-field">
        <span>Speed</span>
        <div className="settings-row">
          <input
            type="range"
            min={50}
            max={200}
            value={rateToSlider(config.rate)}
            data-element-id="speed"
            disabled={loading || saving || config.provider === "silent"}
            onChange={(event) =>
              patchConfig({ rate: rateFromSlider(Number(event.currentTarget.value)) })
            }
          />
          <span className="voice-range-value">{formatRate(config.rate)}</span>
        </div>
      </label>

      <label className="settings-field">
        <span>Pitch</span>
        <div className="settings-row">
          <input
            type="range"
            min={50}
            max={200}
            value={pitchToSlider(config.pitch)}
            data-element-id="pitch"
            disabled={loading || saving || config.provider === "silent"}
            onChange={(event) =>
              patchConfig({ pitch: pitchFromSlider(Number(event.currentTarget.value)) })
            }
          />
          <span className="voice-range-value">{formatPitch(config.pitch)}</span>
        </div>
      </label>

      <div className="settings-field settings-toggle-field">
        <span>Speak reminders aloud</span>
        <button
          type="button"
          className={`settings-toggle${config.speakOnReminder ? " settings-toggle--on" : ""}`}
          data-element-id="speak-reminders"
          aria-pressed={config.speakOnReminder}
          disabled={loading || saving}
          onClick={() => patchConfig({ speakOnReminder: !config.speakOnReminder })}
        />
      </div>

      <div className="settings-field settings-toggle-field">
        <span>Speak AI responses aloud</span>
        <button
          type="button"
          className={`settings-toggle${config.speakOnAi ? " settings-toggle--on" : ""}`}
          data-element-id="speak-ai"
          aria-pressed={config.speakOnAi}
          disabled={loading || saving}
          onClick={() => patchConfig({ speakOnAi: !config.speakOnAi })}
        />
      </div>

      {error ? <p className="settings-error">{error}</p> : null}
      {testError ? <p className="settings-error">{testError}</p> : null}

      <div className="ai-settings-actions">
        <button
          className="settings-primary"
          type="button"
          data-element-id="test-voice-btn"
          onClick={handleTestVoice}
          disabled={loading || saving || testing || config.provider === "silent"}
        >
          {testing ? "Testing..." : "Test Voice"}
        </button>
        <button
          className="settings-primary"
          type="button"
          data-element-id="voice-save-btn"
          onClick={handleSave}
          disabled={loading || saving || testing}
        >
          {saving ? "Saving..." : "Save Voice Settings"}
        </button>
      </div>
    </div>
  );
}
