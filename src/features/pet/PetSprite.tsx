import type { CSSProperties } from "react";
import type { PetAnimation } from "./pet.types";
import { useActiveSkin, skinAssetUrl } from "../customization/useActiveSkin";
import "./animations.css";

const PET_SCALE = 1;
const SPRITE_HEADROOM = 10;

type PetSpriteProps = {
  animation: PetAnimation;
};

export function PetSprite({ animation }: PetSpriteProps) {
  const { manifest, basePath, loading, activeSkin } = useActiveSkin();
  const resolvedAnimation = animation === "celebrate" ? "walk" : animation;

  if (loading || !manifest) {
    return (
      <div
        className="pet-sprite-frame"
        style={{ width: `${64 * PET_SCALE}px`, height: `${64 * PET_SCALE + SPRITE_HEADROOM}px` }}
      >
        <div
          className={`pet-sprite pet-sprite--${animation === "celebrate" ? "celebrate" : animation}`}
          style={{ width: `${64 * PET_SCALE}px`, height: `${64 * PET_SCALE}px` }}
          aria-label={`${animation} pet sprite`}
          role="img"
        />
      </div>
    );
  }

  const sheet = manifest.animations[resolvedAnimation];
  if (!sheet) {
    return null;
  }

  const frameWidth = manifest.frameWidth * PET_SCALE;
  const frameHeight = manifest.frameHeight * PET_SCALE;
  const sheetWidth = frameWidth * sheet.frameCount;
  const src = skinAssetUrl(basePath, sheet.file);
  const spriteStyle = {
    backgroundImage: `url(${src})`,
    width: `${frameWidth}px`,
    height: `${frameHeight}px`,
    backgroundSize: `${sheetWidth}px ${frameHeight}px`,
    ["--frame-width" as string]: `${frameWidth}px`,
    ["--frame-count" as string]: String(sheet.frameCount),
    ["--anim-duration" as string]: `${sheet.durationMs}ms`,
  } as CSSProperties;

  return (
    <div
      className="pet-sprite-frame"
      style={{ width: `${frameWidth}px`, height: `${frameHeight + SPRITE_HEADROOM}px` }}
    >
      <div
        key={activeSkin}
        className={`pet-sprite pet-sprite--${animation === "celebrate" ? "celebrate" : animation}`}
        style={spriteStyle}
        aria-label={`${animation} pet sprite`}
        role="img"
      />
    </div>
  );
}
