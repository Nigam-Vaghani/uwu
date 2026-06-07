import { useState } from "react";
import { save, open } from "@tauri-apps/plugin-dialog";
import { invokeCommand } from "../../hooks/useTauriStore";
import { useSettingsStore } from "../../store/settingsStore";
import { applyThemeById } from "./themeUtils";

type ImportExportProps = {
  disabled?: boolean;
  onImported?: () => void;
};

export function ImportExport({ disabled, onImported }: ImportExportProps) {
  const setConfig = useSettingsStore((state) => state.setConfig);
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const handleExport = async () => {
    setBusy(true);
    setStatus(null);
    try {
      const outputPath = await save({
        defaultPath: "my-companion.uwu",
        filters: [{ name: "UWU Bundle", extensions: ["uwu"] }],
      });
      if (!outputPath) {
        return;
      }

      await invokeCommand("export_uwu_bundle", { outputPath });
      setStatus("Configuration exported successfully.");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setStatus(message);
    } finally {
      setBusy(false);
    }
  };

  const handleImport = async () => {
    setBusy(true);
    setStatus(null);
    try {
      const inputPath = await open({
        multiple: false,
        filters: [{ name: "UWU Bundle", extensions: ["uwu"] }],
      });
      if (!inputPath || Array.isArray(inputPath)) {
        return;
      }

      const saved = await invokeCommand("import_uwu_bundle", { inputPath });
      setConfig(saved);
      await applyThemeById(saved.activeTheme);
      setStatus("Configuration imported successfully.");
      onImported?.();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setStatus(message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="customization-import-export">
      <p className="settings-hint">
        Export your settings, reminders, skin, theme, and sound preferences as a `.uwu` bundle.
      </p>
      <div className="customization-import-export-actions">
        <button
          type="button"
          className="settings-secondary"
          data-element-id="import-btn"
          disabled={disabled || busy}
          onClick={handleImport}
        >
          Import .uwu
        </button>
        <button
          type="button"
          className="settings-primary"
          data-element-id="export-btn"
          disabled={disabled || busy}
          onClick={handleExport}
        >
          Export .uwu
        </button>
      </div>
      {status ? <p className="settings-hint">{status}</p> : null}
    </div>
  );
}
