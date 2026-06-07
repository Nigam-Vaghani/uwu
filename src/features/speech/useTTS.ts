import { useCallback, useRef } from "react";
import type { TtsConfig } from "./speech.types";
import { isRustTtsProvider } from "./speech.types";
import { invokeCommand } from "../../hooks/useTauriStore";

type SpeakFn = (text: string, config: TtsConfig) => Promise<void>;

const FALLBACK_ORDER: TtsConfig["provider"][] = ["webSpeech", "windowsSapi", "piper"];

function resolveVoice(
  utterance: SpeechSynthesisUtterance,
  config: TtsConfig,
): void {
  const voices = window.speechSynthesis.getVoices();
  if (voices.length === 0) {
    return;
  }

  const match = config.voiceId
    ? voices.find(
        (voice) =>
          voice.voiceURI === config.voiceId ||
          voice.name === config.voiceId,
      )
    : voices.find((voice) => voice.default) ?? voices[0];

  if (match) {
    utterance.voice = match;
  }
}

function speakWebSpeech(text: string, config: TtsConfig): Promise<void> {
  return new Promise((resolve, reject) => {
    if (!window.speechSynthesis) {
      reject(new Error("Web Speech API is unavailable"));
      return;
    }

    window.speechSynthesis.cancel();

    const utterance = new SpeechSynthesisUtterance(text);
    utterance.rate = config.rate;
    utterance.pitch = config.pitch;
    utterance.volume = config.volume;
    resolveVoice(utterance, config);

    utterance.onend = () => resolve();
    utterance.onerror = (event) => {
      reject(new Error(event.error || "Web Speech synthesis failed"));
    };

    window.speechSynthesis.speak(utterance);
  });
}

async function speakWithProvider(
  provider: TtsConfig["provider"],
  text: string,
  config: TtsConfig,
): Promise<void> {
  if (provider === "silent") {
    return;
  }

  if (provider === "webSpeech") {
    await speakWebSpeech(text, config);
    return;
  }

  if (isRustTtsProvider(provider)) {
    await invokeCommand("speak", { text, provider });
  }
}

export function useTTS() {
  const activeProviderRef = useRef<TtsConfig["provider"] | null>(null);

  const speak = useCallback<SpeakFn>(async (text, config) => {
    if (config.provider === "silent") {
      return;
    }

    const trimmed = text.trim();
    if (!trimmed) {
      return;
    }

    const chain = [
      config.provider,
      ...FALLBACK_ORDER.filter((provider) => provider !== config.provider),
    ];

    let lastError: Error | null = null;

    for (const provider of chain) {
      if (provider === "silent") {
        return;
      }

      try {
        await speakWithProvider(provider, trimmed, config);
        activeProviderRef.current = provider;
        return;
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));
      }
    }

    if (lastError) {
      throw lastError;
    }
  }, []);

  const stop = useCallback(async () => {
    if (window.speechSynthesis) {
      window.speechSynthesis.cancel();
    }
    try {
      await invokeCommand("stop_speaking", {});
    } catch {
      // Ignore when Rust TTS is idle.
    }
    activeProviderRef.current = null;
  }, []);

  return { speak, stop };
}

export { speakWebSpeech };
