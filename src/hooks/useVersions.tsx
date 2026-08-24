import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { VersionManifest } from "../types";

const POLL_INTERVAL_MS = 3000;
const MAX_ATTEMPTS = 20;

export type VersionsStatus = "loading" | "ready" | "unavailable";

export function useVersions() {
  const [manifest, setManifest] = useState<VersionManifest | null>(null);
  const [status, setStatus] = useState<VersionsStatus>("loading");
  const attemptsRef = useRef(0);
  const generationRef = useRef(0);

  async function fetchOnce(): Promise<boolean> {
    try {
      const result = await invoke<VersionManifest | null>("get_versions");
      console.log(result)
      if (result) {
        setManifest(result);
        setStatus("ready");
        return true;
      }
    } catch (err) {
      console.warn("get_versions failed:", err);
    }
    return false;
  }

  function startPolling() {
    const generation = ++generationRef.current;
    attemptsRef.current = 0;
    setStatus("loading");

    const tick = async () => {
      if (generation !== generationRef.current) return;
      const ok = await fetchOnce();
      if (ok || generation !== generationRef.current) return;
      attemptsRef.current += 1;
      if (attemptsRef.current >= MAX_ATTEMPTS) {
        setStatus("unavailable");
        return;
      }
      setTimeout(tick, POLL_INTERVAL_MS);
    };

    tick();
  }

  useEffect(() => {
    startPolling();
    return () => {
      generationRef.current += 1;
    };
  }, []);

  return { manifest, status, retry: startPolling };
}
