import { useCallback } from "react";
import { useCustomization } from "./useCustomization";

export type CustomizationPanelTab = "skins" | "themes" | "sounds" | "export";

export function useCustomizationPanel() {
  const customization = useCustomization();

  const previewSoundPack = useCallback(async (packId: string) => {
    try {
      const response = await fetch(`/sounds/${packId}/manifest.json`);
      if (!response.ok) {
        return;
      }
      const manifest = (await response.json()) as { sounds: Record<string, string> };
      const fileName = manifest.sounds.reaction ?? manifest.sounds.notification;
      if (!fileName) {
        return;
      }
      const audio = new Audio(`/sounds/${packId}/${fileName}`);
      audio.volume = 0.5;
      await audio.play();
    } catch {
      // Preview is best-effort.
    }
  }, []);

  return {
    ...customization,
    previewSoundPack,
  };
}
