import type { SkinManifest } from "./customization.types";
import { convertFileSrc } from "@tauri-apps/api/core";
import { skinAssetUrl } from "./useActiveSkin";

type SkinSelectorProps = {
  skins: SkinManifest[];
  activeSkinId: string;
  disabled?: boolean;
  onSelect: (skinId: string) => void;
};

export function SkinSelector({ skins, activeSkinId, disabled, onSelect }: SkinSelectorProps) {
  return (
    <div className="customization-skin-grid">
      {skins.map((skin) => {
        const selected = skin.id === activeSkinId;
        const previewSrc = skin.assetPath
          ? skinAssetUrl(convertFileSrc(skin.assetPath), skin.preview)
          : skinAssetUrl(`/skins/${skin.id}`, skin.preview);

        return (
          <button
            key={skin.id}
            type="button"
            className={`customization-skin-card${selected ? " customization-skin-card--selected" : ""}`}
            disabled={disabled}
            data-element-id={`skin-${skin.id}`}
            onClick={() => onSelect(skin.id)}
          >
            <div className="customization-skin-preview">
              <img src={previewSrc} alt="" width={64} height={64} />
            </div>
            <div className="customization-skin-name">{skin.name}</div>
          </button>
        );
      })}

      <div className="customization-skin-card customization-skin-card--add">
        <div className="customization-skin-add">+</div>
        <div className="customization-skin-name">Drop folder in skins/</div>
      </div>
    </div>
  );
}
