export type SkinManifest = {
  id: string;
  name: string;
  author: string;
  version: string;
  frameWidth: number;
  frameHeight: number;
  preview: string;
  builtin: boolean;
  assetPath: string | null;
};

export type SoundPackManifest = {
  id: string;
  name: string;
  author: string;
  version: string;
  builtin: boolean;
};

export type PersonalityInfo = {
  id: string;
  name: string;
};

export type ThemeDefinition = {
  id: string;
  name: string;
  variables: Record<string, string>;
};

export type SkinAnimationManifest = {
  file: string;
  frameCount: number;
  durationMs: number;
};

export type SkinFileManifest = {
  id: string;
  name: string;
  author: string;
  version: string;
  frameWidth: number;
  frameHeight: number;
  preview: string;
  animations: Record<string, SkinAnimationManifest>;
};

export type SoundPackFileManifest = {
  id: string;
  name: string;
  sounds: Record<string, string>;
};

export type ResolvedSkinManifest = {
  manifest: SkinFileManifest;
  assetPath: string | null;
};

export const THEME_IDS = ["elegance", "elegance-dark", "blue", "green", "pink", "purple", "orange"] as const;
export type ThemeId = (typeof THEME_IDS)[number];
