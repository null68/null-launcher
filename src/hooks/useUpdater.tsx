import { useCallback, useEffect, useRef, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdaterStatus =
  | "checking"
  | "idle"
  | "available"
  | "downloading"
  | "relaunching"
  | "error";

export function useUpdater() {
  const [status, setStatus] = useState<UpdaterStatus>("checking");
  const [version, setVersion] = useState<string | null>(null);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const pending = useRef<Update | null>(null);

  const checkForUpdate = useCallback(async () => {
    setStatus("checking");
    setError(null);
    try {
      const update = await check();
      pending.current = update;
      if (update) {
        setVersion(update.version);
        setStatus("available");
      } else {
        setStatus("idle");
      }
    } catch (err) {
      setError(String(err));
      setStatus("error");
    }
  }, []);

  useEffect(() => {
    checkForUpdate();
  }, [checkForUpdate]);

  const installAndRelaunch = useCallback(async () => {
    const update = pending.current;
    if (!update) return;

    setStatus("downloading");
    setProgress(0);
    let downloaded = 0;
    let total = 0;

    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength ?? 0;
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          setProgress(total > 0 ? Math.min(100, Math.round((downloaded / total) * 100)) : 0);
        } else if (event.event === "Finished") {
          setProgress(100);
        }
      });
      setStatus("relaunching");
      await relaunch();
    } catch (err) {
      setError(String(err));
      setStatus("error");
    }
  }, []);

  return { status, version, progress, error, checkForUpdate, installAndRelaunch };
}
