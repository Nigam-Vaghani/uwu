type PetContextMenuProps = {
  open: boolean;
  x: number;
  y: number;
  onSettings: () => void;
  onQuit: () => void;
};

export function PetContextMenu({ open, x, y, onSettings, onQuit }: PetContextMenuProps) {
  if (!open) {
    return null;
  }

  return (
    <div
      className="pet-context-menu"
      style={{ left: x, top: y }}
      role="menu"
      onMouseDown={(event) => event.stopPropagation()}
    >
      <button type="button" onMouseDown={(event) => event.stopPropagation()} onClick={onSettings}>
        Settings
      </button>
      <button type="button" onMouseDown={(event) => event.stopPropagation()} onClick={onQuit}>
        Quit
      </button>
    </div>
  );
}
