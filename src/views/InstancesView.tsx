import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useVersions } from "../hooks/useVersions";
import { useInstances } from "../hooks/useInstances";
import { useSettings } from "../hooks/useSettings";
import { useLoaderVersions } from "../hooks/useLoaderVersions";
import { Select } from "../components/Select";
import { EmptyState } from "../components/EmptyState";
import { ProgressBar } from "../components/ProgressBar";
import { CubeIcon, DownloadIcon, PlayIcon, RefreshIcon } from "../components/icons";
import type { Loader, VersionType } from "../types";
import type { SelectOption } from "../components/Select";
import type { useInstallManager } from "../hooks/useInstallManager";

import "../styles/InstancesView.css";

const TYPE_LABELS: Record<VersionType, string> = {
  release: "Release",
  snapshot: "Snapshot",
  old_beta: "Beta",
  old_alpha: "Alpha",
};

const TYPE_ORDER: VersionType[] = ["release", "snapshot", "old_beta", "old_alpha"];

const LOADER_LABELS: Record<Loader, string> = {
  vanilla: "Vanilla",
  fabric: "Fabric",
  quilt: "Quilt",
  forge: "Forge",
  neoforge: "NeoForge",
  optifine: "OptiFine",
};

const LOADER_ORDER: Loader[] = ["vanilla", "fabric", "quilt", "forge", "neoforge"];

function chipVariant(type: VersionType): "release" | "snapshot" | "legacy" {
  if (type === "release") return "release";
  if (type === "snapshot") return "snapshot";
  return "legacy";
}

function tagClassFor(type: VersionType): string {
  if (type === "release") return "tag-release";
  if (type === "snapshot") return "tag-snapshot";
  return "tag-legacy";
}

interface InstancesViewProps {
  installManager: ReturnType<typeof useInstallManager>;
}

export function InstancesView({ installManager }: InstancesViewProps) {
  const { manifest, status, retry } = useVersions();
  const { instances, loading: instancesLoading, refresh: refreshInstances } = useInstances();
  const { settings } = useSettings();

  const [activeTypes, setActiveTypes] = useState<Set<VersionType>>(new Set(["release"]));
  const [selectedInstanceId, setSelectedInstanceId] = useState("");
  const [versionToInstall, setVersionToInstall] = useState("");
  const [loaderToInstall, setLoaderToInstall] = useState<Loader>("vanilla");
  const [loaderVersionToInstall, setLoaderVersionToInstall] = useState("");
  const [launching, setLaunching] = useState(false);
  const [launchError, setLaunchError] = useState<string | null>(null);

  const { installing, progress, error: installError, installVanilla, installModded } = installManager;

  const { versions: loaderVersions, loading: loaderVersionsLoading } = useLoaderVersions(
    loaderToInstall,
    versionToInstall,
  );

  useEffect(() => {
    if (instances.length > 0 && !instances.some((i) => i.id === selectedInstanceId)) {
      setSelectedInstanceId(instances[0].id);
    }
    if (instances.length === 0 && selectedInstanceId) {
      setSelectedInstanceId("");
    }
  }, [instances, selectedInstanceId]);

  useEffect(() => {
    setLoaderVersionToInstall("");
  }, [loaderToInstall, versionToInstall]);

  const filteredVersions = useMemo(() => {
     if (!manifest) return [];
     return manifest.versions.filter((v) => activeTypes.has(v.type));
   }, [manifest, activeTypes]);

   const instanceOptions: SelectOption[] = instances.map((i) => ({ value: i.id, label: i.id }));
   const versionOptions: SelectOption[] = filteredVersions.map((v) => ({ value: v.id, label: v.id }));
   const loaderVersionOptions: SelectOption[] = loaderVersions.map((v) => ({ value: v, label: v }));

   const versionPlaceholder =
     status === "loading" ? "Loading versions…" : status === "unavailable" ? "Couldn't load versions" : "Choose a version…";
   const loaderVersionPlaceholder = loaderVersionsLoading
     ? "Loading…"
     : loaderVersions.length === 0
       ? `No ${LOADER_LABELS[loaderToInstall]} builds for this version`
       : `Choose a ${LOADER_LABELS[loaderToInstall]} version…`;

  function toggleType(type: VersionType) {
    setActiveTypes((prev) => {
      const next = new Set(prev);
      if (next.has(type)) {
        if (next.size > 1) next.delete(type);
      } else {
        next.add(type);
      }
      return next;
    });
  }

  async function handleInstall() {
    if (!versionToInstall || installing) return;
    if (loaderToInstall !== "vanilla" && !loaderVersionToInstall) return;
    try {
      const installedId =
        loaderToInstall === "vanilla"
          ? await installVanilla(versionToInstall)
          : await installModded(versionToInstall, loaderToInstall, loaderVersionToInstall);
      await refreshInstances();
      setSelectedInstanceId(installedId);
      setVersionToInstall("");
      setLoaderToInstall("vanilla");
    } catch (err) {
      console.warn("install failed:", err);
    }
  }

  async function handlePlay() {
    if (!selectedInstanceId || launching) return;
    setLaunching(true);
    setLaunchError(null);
    try {
      await invoke("launch_instance", {
        versionId: selectedInstanceId,
        username: settings.username,
        terminalMode: settings.terminalMode,
        minMemoryMb: settings.minMemoryMb,
        maxMemoryMb: settings.maxMemoryMb,
      });
    } catch (err) {
      setLaunchError(String(err));
    } finally {
      setLaunching(false);
    }
  }

  const selectedInstance = instances.find((i) => i.id === selectedInstanceId) ?? null;

  return (
    <section className="view instances-view">
      <div className="view-header">
        <h1>Instances</h1>
        <div className="view-subtitle">Downloaded Minecraft versions, ready to play.</div>
      </div>

      <div className="toolbar">
        <div className="toolbar-group instance-picker">
          <label className="eyebrow">Instance</label>
          <Select
            value={selectedInstanceId}
            onChange={setSelectedInstanceId}
            options={instanceOptions}
            placeholder={instancesLoading ? "Loading…" : "No versions yet"}
            disabled={instances.length === 0}
            aria-label="Select instance" />
        </div>

        <div className="toolbar-divider" />

        <div className="toolbar-group install-group">
          <label className="eyebrow">Install new version</label>

          <div className="chip-row">
            {TYPE_ORDER.map((type) => (
              <button
                key={type}
                type="button"
                className={activeTypes.has(type) ? `chip on-${chipVariant(type)}` : "chip"}
                aria-pressed={activeTypes.has(type)}
                onClick={() => toggleType(type)}
              >
                <span className="dot" />
                {TYPE_LABELS[type]}
              </button>
            ))}
          </div>

          <div className="chip-row">
            {LOADER_ORDER.map((loader) => (
              <button
                key={loader}
                type="button"
                className={loaderToInstall === loader ? "chip on-legacy" : "chip"}
                aria-pressed={loaderToInstall === loader}
                onClick={() => setLoaderToInstall(loader)}
              >
                {LOADER_LABELS[loader]}
              </button>
            ))}
          </div>

          <div className="install-row">
            <Select
              value={versionToInstall}
              onChange={setVersionToInstall}
              options={versionOptions}
              placeholder={versionPlaceholder}
              disabled={installing}
              className="version-select"
              aria-label="Version to install" />

            {loaderToInstall !== "vanilla" && (
              <Select
                value={loaderVersionToInstall}
                onChange={setLoaderVersionToInstall}
                options={loaderVersionOptions}
                placeholder={loaderVersionPlaceholder}
                disabled={installing || !versionToInstall || loaderVersionsLoading}
                className="version-select"
                aria-label={`${LOADER_LABELS[loaderToInstall]} version to install`} />
            )}

            {status === "unavailable" && !installing && (
                <button type="button" className="btn btn-ghost" onClick={retry}>
                  <RefreshIcon />
                    Retry
                </button>
                )}

            <button
              type="button"
              className="btn btn-primary"
              disabled={!versionToInstall || installing || (loaderToInstall !== "vanilla" && !loaderVersionToInstall)}
              onClick={handleInstall}
            >
              <DownloadIcon />
              {installing ? "Installing…" : "Install"}
            </button>
          </div>
        </div>
      </div>

      {installing && progress && (
        <ProgressBar
          label={`Installing ${installManager.installingId ?? "…"}`}
          downloadedBytes={progress.downloadedBytes}
          totalBytes={progress.totalBytes}
          filesDone={progress.filesDone}
          filesTotal={progress.filesTotal}
        />
      )}
      {installError && <div className="launch-error">{installError}</div>}

      {selectedInstance ? (
        <>
          <div className="slot instance-card">
            <CubeIcon className={`instance-cube cube-${chipVariant(selectedInstance.type)}`} />
            <div className="instance-info">
              <div className="instance-id">{selectedInstance.id}</div>
              <span className={`tag ${tagClassFor(selectedInstance.type)}`}>
                {TYPE_LABELS[selectedInstance.type]}
              </span>
              {selectedInstance.loader && selectedInstance.loader !== "vanilla" && (
                <span className="tag tag-loader">{LOADER_LABELS[selectedInstance.loader]}</span>
              )}
            </div>
            <div className="instance-actions">
              <button
                type="button"
                className="btn btn-primary"
                onClick={handlePlay}
                disabled={launching}
              >
                {launching ? <RefreshIcon className="spin" /> : <PlayIcon />}
                {launching ? "Playing…" : "Play"}
              </button>
            </div>
          </div>
          {launchError && <div className="launch-error">{launchError}</div>}
        </>
      ) : (
        <EmptyState
          icon={<CubeIcon className="icon" />}
          title="No instances yet"
          body="Pick a version above and hit Install to download your first one."
        />
      )}
    </section>
  );
}
