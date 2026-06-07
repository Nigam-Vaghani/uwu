import type { SoundPackManifest } from "./customization.types";

type SoundPackSelectorProps = {
  packs: SoundPackManifest[];
  activePackId: string;
  disabled?: boolean;
  onSelect: (packId: string) => void;
  onPreview: (packId: string) => void;
};

export function SoundPackSelector({
  packs,
  activePackId,
  disabled,
  onSelect,
  onPreview,
}: SoundPackSelectorProps) {
  return (
    <ul className="customization-sound-list">
      {packs.map((pack) => {
        const selected = pack.id === activePackId;
        return (
          <li key={pack.id} className={`customization-sound-item${selected ? " customization-sound-item--selected" : ""}`}>
            <button
              type="button"
              className="customization-sound-select"
              disabled={disabled}
              onClick={() => onSelect(pack.id)}
            >
              <strong>{pack.name}</strong>
              <span>{pack.author}</span>
            </button>
            <button
              type="button"
              className="settings-secondary customization-sound-preview"
              disabled={disabled}
              onClick={() => onPreview(pack.id)}
            >
              Preview
            </button>
          </li>
        );
      })}
    </ul>
  );
}
