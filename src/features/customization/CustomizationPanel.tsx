import { useState } from "react";
import { CustomizationPanelTab, useCustomizationPanel } from "./useCustomizationPanel";
import { SkinSelector } from "./SkinSelector";
import { ThemeSelector } from "./ThemeSelector";
import { SoundPackSelector } from "./SoundPackSelector";
import { ImportExport } from "./ImportExport";

const TABS: { id: CustomizationPanelTab; label: string }[] = [
  { id: "skins", label: "Skins" },
  { id: "themes", label: "Themes" },
  { id: "sounds", label: "Sounds" },
  { id: "export", label: "Import/Export" },
];

export function CustomizationPanel() {
  const [tab, setTab] = useState<CustomizationPanelTab>("skins");
  const {
    config,
    skins,
    soundPacks,
    loading,
    saving,
    error,
    refresh,
    setActiveSkin,
    setActiveTheme,
    setActiveSoundPack,
    previewSoundPack,
  } = useCustomizationPanel();

  return (
    <div className="settings-section customization-panel">
      <h2>Customization</h2>

      <nav className="customization-tabs" aria-label="Customization sections">
        {TABS.map((item) => (
          <button
            key={item.id}
            type="button"
            className={`customization-tab${tab === item.id ? " customization-tab--active" : ""}`}
            data-element-id={`tab-${item.id}`}
            onClick={() => setTab(item.id)}
          >
            {item.label}
          </button>
        ))}
      </nav>

      {error ? <p className="settings-error">{error}</p> : null}

      {tab === "skins" ? (
        <SkinSelector
          skins={skins}
          activeSkinId={config.activeSkin}
          disabled={loading || saving}
          onSelect={setActiveSkin}
        />
      ) : null}

      {tab === "themes" ? (
        <div>
          <p className="settings-hint">Color theme</p>
          <ThemeSelector
            activeThemeId={config.activeTheme}
            disabled={loading || saving}
            onSelect={setActiveTheme}
          />
        </div>
      ) : null}

      {tab === "sounds" ? (
        <SoundPackSelector
          packs={soundPacks}
          activePackId={config.activeSoundPack}
          disabled={loading || saving}
          onSelect={setActiveSoundPack}
          onPreview={previewSoundPack}
        />
      ) : null}

      {tab === "export" ? (
        <ImportExport disabled={loading || saving} onImported={refresh} />
      ) : null}
    </div>
  );
}
