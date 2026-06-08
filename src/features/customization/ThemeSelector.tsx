import { THEME_IDS } from "./customization.types";

const THEME_SWATCHES: Record<string, string> = {
  elegance: "#E64980",
  "elegance-dark": "#FFC2D1",
  blue: "#5A82A8",
  green: "#5BA373",
  pink: "#FF3377",
  purple: "#9B59B6",
  orange: "#E07A3E",
};

type ThemeSelectorProps = {
  activeThemeId: string;
  disabled?: boolean;
  onSelect: (themeId: string) => void;
};

export function ThemeSelector({ activeThemeId, disabled, onSelect }: ThemeSelectorProps) {
  return (
    <div className="customization-theme-row">
      {THEME_IDS.map((themeId) => (
        <button
          key={themeId}
          type="button"
          className={`customization-theme-swatch${activeThemeId === themeId ? " customization-theme-swatch--selected" : ""}`}
          style={{ background: THEME_SWATCHES[themeId] ?? "#E64980" }}
          disabled={disabled}
          aria-label={`${themeId} theme`}
          data-element-id={`theme-${themeId}`}
          onClick={() => onSelect(themeId)}
        />
      ))}
    </div>
  );
}
