import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Loader } from "../types";

export function useLoaderVersions(loader: Loader, gameVersion: string) {
  const [versions, setVersions] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (loader === "vanilla" || !gameVersion) {
      setVersions([]);
      return;
    }
    let cancelled = false;
    setLoading(true);
    invoke<string[]>("list_loader_versions", { loader, gameVersion })
      .then((result) => {
        if (!cancelled) setVersions(result);
      })
      .catch(() => {
        if (!cancelled) setVersions([]);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [loader, gameVersion]);

  return { versions, loading };
}
