import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "../store/settingsStore";
import type { TauriCommand, TauriCommandArgs, TauriCommandResult } from "../types/tauri-commands";

export function invokeCommand<T extends TauriCommand>(
  command: T,
  args: TauriCommandArgs<T>,
): Promise<TauriCommandResult<T>> {
  return invoke(command, args) as Promise<TauriCommandResult<T>>;
}

export function useTauriStore() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadConfig = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      return await invokeCommand("get_config", {});
    } catch (loadError) {
      const message = loadError instanceof Error ? loadError.message : String(loadError);
      setError(message);
      throw loadError;
    } finally {
      setLoading(false);
    }
  }, []);

  const saveConfig = useCallback(async (config: AppConfig) => {
    setLoading(true);
    setError(null);
    try {
      return await invokeCommand("set_config", { config });
    } catch (saveError) {
      const message = saveError instanceof Error ? saveError.message : String(saveError);
      setError(message);
      throw saveError;
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    setError(null);
  }, []);

  return {
    loading,
    error,
    loadConfig,
    saveConfig,
  };
}
