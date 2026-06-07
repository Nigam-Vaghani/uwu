import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import { invokeCommand } from "../../hooks/useTauriStore";
import { useSettingsStore } from "../../store/settingsStore";
import type { SkinFileManifest } from "./customization.types";

function joinAssetPath(base: string, file: string) {
  return `${base.replace(/\/$/, "")}/${file.replace(/^\//, "")}`;
}

export function useActiveSkin() {
  const activeSkin = useSettingsStore((state) => state.config.activeSkin);
  const [manifest, setManifest] = useState<SkinFileManifest | null>(null);
  const [basePath, setBasePath] = useState("/skins/default");
  const [loading, setLoading] = useState(true);

  const loadSkin = useCallback(async (skinId: string) => {
    setLoading(true);
    try {
      const resolved = await invokeCommand("get_skin_manifest", { skinId });
      const data = resolved.manifest as SkinFileManifest;
      setManifest(data);
      setBasePath(
        resolved.assetPath
          ? convertFileSrc(resolved.assetPath)
          : `/skins/${skinId}`,
      );
    } catch {
      try {
        const response = await fetch(`/skins/${skinId}/manifest.json`);
        if (!response.ok) {
          throw new Error("Skin manifest not found");
        }
        const data = (await response.json()) as SkinFileManifest;
        setManifest(data);
        setBasePath(`/skins/${skinId}`);
      } catch {
        setManifest(null);
        setBasePath("/skins/default");
      }
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadSkin(activeSkin);
  }, [activeSkin, loadSkin]);

  useEffect(() => {
    const unlisten = listen<string>("customization:skin-changed", (event) => {
      void loadSkin(event.payload);
    });

    return () => {
      void unlisten.then((release) => release());
    };
  }, [loadSkin]);

  return { manifest, basePath, loading, activeSkin, joinAssetPath };
}

export function skinAssetUrl(basePath: string, file: string) {
  return joinAssetPath(basePath, file);
}
