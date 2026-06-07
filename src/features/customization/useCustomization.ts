import { useCallback, useEffect, useState } from "react";
import { invokeCommand } from "../../hooks/useTauriStore";
import { useSettingsStore } from "../../store/settingsStore";
import type {
  PersonalityInfo,
  SkinManifest,
  SoundPackManifest,
} from "./customization.types";
import { applyThemeById } from "./themeUtils";

export function useCustomization() {
  const config = useSettingsStore((state) => state.config);
  const patchConfig = useSettingsStore((state) => state.patchConfig);
  const setConfig = useSettingsStore((state) => state.setConfig);

  const [skins, setSkins] = useState<SkinManifest[]>([]);
  const [soundPacks, setSoundPacks] = useState<SoundPackManifest[]>([]);
  const [personalities, setPersonalities] = useState<PersonalityInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [loadedSkins, loadedPacks, loadedPersonalities] = await Promise.all([
        invokeCommand("list_skins", {}),
        invokeCommand("list_sound_packs", {}),
        invokeCommand("list_personalities", {}),
      ]);
      setSkins(loadedSkins);
      setSoundPacks(loadedPacks);
      setPersonalities(loadedPersonalities);
    } catch (loadError) {
      const message = loadError instanceof Error ? loadError.message : String(loadError);
      setError(message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    void applyThemeById(config.activeTheme);
  }, [config.activeTheme]);

  const setActiveSkin = useCallback(
    async (skinId: string) => {
      setSaving(true);
      setError(null);
      try {
        const saved = await invokeCommand("set_active_skin", { skinId });
        setConfig(saved);
        return saved;
      } catch (saveError) {
        const message = saveError instanceof Error ? saveError.message : String(saveError);
        setError(message);
        throw saveError;
      } finally {
        setSaving(false);
      }
    },
    [setConfig],
  );

  const setActiveTheme = useCallback(
    async (themeId: string) => {
      setSaving(true);
      setError(null);
      try {
        const saved = await invokeCommand("set_active_theme", { themeId });
        setConfig(saved);
        await applyThemeById(themeId);
        return saved;
      } catch (saveError) {
        const message = saveError instanceof Error ? saveError.message : String(saveError);
        setError(message);
        throw saveError;
      } finally {
        setSaving(false);
      }
    },
    [setConfig],
  );

  const setActiveSoundPack = useCallback(
    async (packId: string) => {
      setSaving(true);
      setError(null);
      try {
        const saved = await invokeCommand("set_active_sound_pack", { packId });
        setConfig(saved);
        return saved;
      } catch (saveError) {
        const message = saveError instanceof Error ? saveError.message : String(saveError);
        setError(message);
        throw saveError;
      } finally {
        setSaving(false);
      }
    },
    [setConfig],
  );

  return {
    config,
    patchConfig,
    skins,
    soundPacks,
    personalities,
    loading,
    saving,
    error,
    refresh,
    setActiveSkin,
    setActiveTheme,
    setActiveSoundPack,
  };
}
