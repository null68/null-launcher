import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { LauncherSettings } from "../types";

const DEFAULT_SETTINGS: LauncherSettings = {
  username: "",
  minMemoryMb: 1024,
  maxMemoryMb: 4096,
  terminalMode: false,
};

const SAVE_DEBOUNCE_MS = 400;

export function useSettings() {
  const [settings, setSettings] = useState<LauncherSettings>(DEFAULT_SETTINGS);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const result = await invoke<LauncherSettings | null>("get_settings");
        if (result) setSettings(result);
      } catch {
      }
    })();
  }, []);

  const update = useCallback((patch: Partial<LauncherSettings>) => {
    setSettings((prev) => {
      const next = { ...prev, ...patch };
      if (saveTimer.current) clearTimeout(saveTimer.current);
      saveTimer.current = setTimeout(() => {
        invoke("save_settings", { settings: next }).catch(() => {
          console.log("failed to save settings");
        });
      }, SAVE_DEBOUNCE_MS);
      return next;
    });
  }, []);

  return { settings, update };
}
