import { THEME_IDS } from "./customization.types";

const THEME_SWATCHES: Record<string, string> = {
  elegance: "#664930",
  "elegance-dark": "#997E67",
  blue: "#CCBEB1",
  green: "#5A7248",
  pink: "#8F5A5A",
  purple: "#6B5A72",
  orange: "#8F5A30",
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
          style={{ background: THEME_SWATCHES[themeId] ?? "#664930" }}
          disabled={disabled}
          aria-label={`${themeId} theme`}
          data-element-id={`theme-${themeId}`}
          onClick={() => onSelect(themeId)}
        />
      ))}
    </div>
  );
}
