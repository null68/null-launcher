import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { InstallProgressPayload } from "../types";

export interface InstallProgressState {
  downloadedBytes: number;
  totalBytes: number;
  filesDone: number;
  filesTotal: number;
}

export function useInstallManager() {
  const [installing, setInstalling] = useState(false);
  const [installingId, setInstallingId] = useState<string | null>(null);
  const [progress, setProgress] = useState<InstallProgressState | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<InstallProgressPayload>("install-progress", (event) => {
      setProgress({
        downloadedBytes: event.payload.downloaded_bytes,
        totalBytes: event.payload.total_bytes,
        filesDone: event.payload.files_done,
        filesTotal: event.payload.files_total,
      });
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const installVanilla = useCallback(async (versionId: string) => {
    setInstalling(true);
    setInstallingId(versionId);
    setProgress(null);
    setError(null);
    try {
      await invoke("install_version", { versionId });
      return versionId;
    } catch (err) {
      setError(String(err));
      throw err;
    } finally {
      setInstalling(false);
      setInstallingId(null);
    }
  }, []);

  const installModded = useCallback(
    async (gameVersionId: string, loader: string, loaderVersion: string) => {
      setInstalling(true);
      setInstallingId(gameVersionId);
      setProgress(null);
      setError(null);
      try {
        return await invoke<string>("install_modded_instance", {
          gameVersionId,
          loader,
          loaderVersion,
        });
      } catch (err) {
        setError(String(err));
        throw err;
      } finally {
        setInstalling(false);
        setInstallingId(null);
      }
    },
    [],
  );

  return { installing, installingId, progress, error, installVanilla, installModded };
}
