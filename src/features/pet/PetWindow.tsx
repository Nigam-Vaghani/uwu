import { useCallback, useEffect, useRef, useState, type MouseEvent } from "react";
import { PhysicalPosition, getCurrentWindow } from "@tauri-apps/api/window";
import { PetSprite } from "./PetSprite";
import { SpeechBubble } from "./SpeechBubble";import { PetContextMenu } from "./PetContextMenu";
import { usePetAnimation } from "./usePetAnimation";
import { MoodIndicator } from "../mood/MoodIndicator";
import { useMood } from "../mood/useMood";
import { useEventListener } from "../events/useEventListener";
import { useSpeechAudio } from "../speech/useSpeechAudio";
import { AchievementToast } from "../gamification/AchievementToast";
import { useSoundEffects } from "../customization/useSoundEffects";
import { useQuoteListener } from "../quotes/useQuoteListener";
import { applyThemeById } from "../customization/themeUtils";
import { usePetStore } from "../../store/petStore";
import { useSettingsStore } from "../../store/settingsStore";
import { invokeCommand, useTauriStore } from "../../hooks/useTauriStore";

const CONTEXT_MENU_WIDTH = 112;

export function PetWindow() {  
  const { animation } = usePetAnimation();
  const { mood, recordInteraction } = useMood();
  useEventListener();
  useQuoteListener();
  useSpeechAudio();
  useSoundEffects();
  const speech = usePetStore((state) => state.speech);
  const menuOpen = usePetStore((state) => state.menuOpen);
  const menuPosition = usePetStore((state) => state.menuPosition);
  const enqueueSpeech = usePetStore((state) => state.enqueueSpeech);
  const dequeueSpeech = usePetStore((state) => state.dequeueSpeech);
  const openMenu = usePetStore((state) => state.openMenu);
  const closeMenu = usePetStore((state) => state.closeMenu);
  const achievementToast = usePetStore((state) => state.achievementToast);
  const clearAchievementToast = usePetStore((state) => state.clearAchievementToast);
  const config = useSettingsStore((state) => state.config);
  const setConfig = useSettingsStore((state) => state.setConfig);
  const patchConfig = useSettingsStore((state) => state.patchConfig);
  const setHydrated = useSettingsStore((state) => state.setHydrated);
  const { loadConfig, saveConfig } = useTauriStore();
  const [isReady, setIsReady] = useState(false);
  const dragStartRef = useRef<{ x: number; y: number } | null>(null);

  const persistPosition = useCallback(
    async (x: number, y: number) => {
      const nextConfig = { ...useSettingsStore.getState().config, position: { x, y } };
      patchConfig({ position: { x, y } });
      await saveConfig(nextConfig);
    },
    [patchConfig, saveConfig],
  );

  useEffect(() => {
    let movedCleanup: (() => void) | undefined;
    let closeCleanup: (() => void) | undefined;

    async function hydrate() {
      const loadedConfig = await loadConfig();
      setConfig(loadedConfig);
      setHydrated(true);
      setIsReady(true);
      void applyThemeById(loadedConfig.activeTheme);

      const petWindow = getCurrentWindow();
      await petWindow.setTitle(`${loadedConfig.petName} - UWU Companion`);

      if (loadedConfig.position) {
        await petWindow.setPosition(
          new PhysicalPosition(loadedConfig.position.x, loadedConfig.position.y),
        );
      }

      if (!loadedConfig.hasSeenWelcome) {
        enqueueSpeech({
          text: `Hi! I'm ${loadedConfig.petName}. Nice to meet you!`,
          tone: "bright",
          source: "welcome",
        });
        await saveConfig({ ...loadedConfig, hasSeenWelcome: true });
        patchConfig({ hasSeenWelcome: true });
      }

      movedCleanup = await petWindow.onMoved(async ({ payload: position }) => {
        await persistPosition(position.x, position.y);
      });

      closeCleanup = await petWindow.onCloseRequested(async (event) => {
        event.preventDefault();
        await invokeCommand("quit_app", {});
      });
    }

    void hydrate();

    return () => {
      movedCleanup?.();
      closeCleanup?.();
    };
  }, [
    enqueueSpeech,
    loadConfig,
    patchConfig,
    persistPosition,
    saveConfig,
    setConfig,
    setHydrated,
  ]);

  useEffect(() => {
    if (isReady) {
      const audio = new Audio("/sounds/hello-voice.mp3");
      void audio.play().catch(e => console.error("Hello voice play failed:", e));
    }
  }, [isReady]);

  useEffect(() => {
    const petWindow = getCurrentWindow();
    const unlisten = petWindow.onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        return;
      }

      void loadConfig().then((loaded) => {
        setConfig(loaded);
        void petWindow.setTitle(`${loaded.petName} - UWU Companion`);
      });
    });

    return () => {
      void unlisten.then((release) => release());
    };
  }, [loadConfig, setConfig]);

  useEffect(() => {
    const closeMenuOnClick = (event: globalThis.MouseEvent) => {
      const target = event.target as HTMLElement | null;
      if (target?.closest(".pet-context-menu")) {
        return;
      }
      closeMenu();
    };
    window.addEventListener("mousedown", closeMenuOnClick);
    return () => window.removeEventListener("mousedown", closeMenuOnClick);
  }, [closeMenu]);

  const handleStartDrag = async (event: MouseEvent<HTMLDivElement>) => {
    if (event.button !== 0) {
      return;
    }
    dragStartRef.current = { x: event.clientX, y: event.clientY };
    closeMenu();
    void recordInteraction();
    await getCurrentWindow().startDragging();
  };

  const handlePetClick = async (event: MouseEvent<HTMLDivElement>) => {
    const start = dragStartRef.current;
    dragStartRef.current = null;
    if (!start) {
      return;
    }

    const moved =
      Math.abs(event.clientX - start.x) > 4 || Math.abs(event.clientY - start.y) > 4;
    if (moved) {
      return;
    }

    void recordInteraction();
    await invokeCommand("emit_pet_interaction", {});
  };

  const handleContextMenu = (event: MouseEvent<HTMLDivElement>) => {
    event.preventDefault();
    openMenu({
      x: Math.floor((window.innerWidth - CONTEXT_MENU_WIDTH) / 2),
      y: 8,
    });
  };

  const handleSettings = async () => {
    closeMenu();
    await invokeCommand("open_settings", {});
  };

  const handleQuit = async () => {
    closeMenu();
    await invokeCommand("quit_app", {});
  };

  useEffect(() => {
    const markInteraction = () => {
      void recordInteraction();
    };

    window.addEventListener("keydown", markInteraction);
    return () => window.removeEventListener("keydown", markInteraction);
  }, [recordInteraction]);

  const handleSpeechDismiss = useCallback(() => {
    const currentSpeech = usePetStore.getState().speech;
    if (
      currentSpeech?.source === "reminder" &&
      currentSpeech.reminderId != null &&
      currentSpeech.reminderCategory
    ) {
      void invokeCommand("acknowledge_reminder", {
        input: {
          reminderId: currentSpeech.reminderId,
          category: currentSpeech.reminderCategory,
        },
      });
    }
    dequeueSpeech();
  }, [dequeueSpeech]);

  return (
    <main
      className={`pet-window${menuOpen ? " pet-window--menu-open" : ""}${speech ? " pet-window--speech-open" : ""}`}      onContextMenu={handleContextMenu}
      aria-label={isReady ? `${config.petName} desktop pet` : "Loading desktop pet"}
    >
      <div
        className="pet-stage"
        style={{ opacity: config.opacity }}
        onMouseDown={handleStartDrag}
        onClick={handlePetClick}
      >
        <SpeechBubble message={speech} onDismiss={handleSpeechDismiss} />
        <div className="pet-avatar">
          <PetSprite animation={animation} />
          <MoodIndicator mood={mood} />
        </div>
      </div>

      <PetContextMenu
        open={menuOpen}
        x={menuPosition.x}
        y={menuPosition.y}
        onSettings={handleSettings}
        onQuit={handleQuit}
      />

      <AchievementToast achievement={achievementToast} onDismiss={clearAchievementToast} />
    </main>
  );
}
