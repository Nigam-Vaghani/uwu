import { useCallback, useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { usePetStore } from "../../store/petStore";
import { invokeCommand } from "../../hooks/useTauriStore";
import type { SpeechMessage } from "../pet/pet.types";
import type { TtsConfig } from "./speech.types";
import { useTtsConfigRef } from "./useTtsConfig";
import { useSpeechQueue } from "./useSpeechQueue";
import { useTTS } from "./useTTS";

function shouldSpeakMessage(message: SpeechMessage, config: TtsConfig): boolean {
  if (config.provider === "silent") {
    return false;
  }

  switch (message.source) {
    case "reminder":
      return config.speakOnReminder;
    case "ai":
      return config.speakOnAi;
    case "quote":
      return config.speakOnAi;
    default:
      return true;
  }
}

export function useSpeechAudio() {
  const speech = usePetStore((state) => state.speech);
  const mode = usePetStore((state) => state.mode);
  const configRef = useTtsConfigRef();
  const { speak, stop } = useTTS();
  const lastSpeechIdRef = useRef<string | null>(null);

  const processItem = useCallback(
    async ({ text }: { text: string }) => {
      const config = configRef.current;
      if (config.provider === "silent") {
        return;
      }
      await speak(text, config);
    },
    [configRef, speak],
  );

  const { enqueue, clear } = useSpeechQueue(processItem);

  useEffect(() => {
    const petWindow = getCurrentWindow();
    const unlisten = petWindow.onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        return;
      }
      void invokeCommand("get_tts_config", {}).then((loaded) => {
        configRef.current = loaded;
      });
    });

    return () => {
      void unlisten.then((release) => release());
    };
  }, [configRef]);

  useEffect(() => {
    if (mode === "sleep") {
      void stop();
      void clear();
    }
  }, [clear, mode, stop]);

  useEffect(() => {
    if (!speech) {
      lastSpeechIdRef.current = null;
      return;
    }

    const speechId = `${speech.text}:${speech.source ?? "default"}:${speech.priority ?? 0}`;
    if (lastSpeechIdRef.current === speechId) {
      return;
    }
    lastSpeechIdRef.current = speechId;

    const config = configRef.current;
    if (!shouldSpeakMessage(speech, config)) {
      return;
    }

    enqueue(speech.text, speech.priority ?? 0);
  }, [configRef, enqueue, speech]);
}
