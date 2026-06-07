import type { ThemeDefinition } from "./customization.types";
import { ELEGANCE_THEME, THEME_PRESETS } from "./themeConstants";

export function applyThemeVariables(variables: Record<string, string>) {
  const root = document.documentElement;
  for (const [name, value] of Object.entries(variables)) {
    root.style.setProperty(name, value);
  }
}

export function applyEleganceTheme() {
  applyThemeVariables(ELEGANCE_THEME);
}

export function normalizeThemeId(themeId: string): string {
  return themeId.trim().toLowerCase();
}

export async function loadTheme(themeId: string): Promise<ThemeDefinition | null> {
  const normalizedId = normalizeThemeId(themeId);
  const preset = THEME_PRESETS[normalizedId];
  if (preset) {
    return { id: normalizedId, name: normalizedId, variables: preset };
  }

  try {
    const response = await fetch(`/themes/${normalizedId}.json`);
    if (!response.ok) {
      return null;
    }
    const theme = (await response.json()) as ThemeDefinition;
    return {
      ...theme,
      variables: { ...ELEGANCE_THEME, ...theme.variables },
    };
  } catch {
    return null;
  }
}

export async function applyThemeById(themeId: string) {
  applyEleganceTheme();

  const normalizedId = normalizeThemeId(themeId);
  const preset = THEME_PRESETS[normalizedId];
  if (preset) {
    applyThemeVariables(preset);
    return;
  }

  const theme = await loadTheme(normalizedId);
  if (theme) {
    applyThemeVariables(theme.variables);
  }
}
