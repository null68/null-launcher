import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useVersions } from "../hooks/useVersions";
import { useInstances } from "../hooks/useInstances";
import { Select } from "../components/Select";
import { EmptyState } from "../components/EmptyState";
import { ProgressBar } from "../components/ProgressBar";
import { CubeIcon, DownloadIcon, PlayIcon, RefreshIcon } from "../components/icons";
import type { InstallProgressPayload, VersionType } from "../types";
import type { SelectOption } from "../components/Select";

import "../styles/InstancesView.css";

const TYPE_LABELS: Record<VersionType, string> = {
  release: "Release",
  snapshot: "Snapshot",
  old_beta: "Beta",
  old_alpha: "Alpha",
};

const TYPE_ORDER: VersionType[] = ["release", "snapshot", "old_beta", "old_alpha"];

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

interface ProgressState {
  downloadedBytes: number;
  totalBytes: number;
  filesDone: number;
  filesTotal: number;
}

export function InstancesView() {
  const { manifest, status, retry } = useVersions();
  const { instances, loading: instancesLoading, refresh: refreshInstances } = useInstances();

  const [activeTypes, setActiveTypes] = useState<Set<VersionType>>(new Set(["release"]));
  const [selectedInstanceId, setSelectedInstanceId] = useState("");
  const [versionToInstall, setVersionToInstall] = useState("");
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState<ProgressState | null>(null);

  useEffect(() => {
    if (instances.length > 0 && !instances.some((i) => i.id === selectedInstanceId)) {
      setSelectedInstanceId(instances[0].id);
    }
    if (instances.length === 0 && selectedInstanceId) {
      setSelectedInstanceId("");
    }
  }, [instances, selectedInstanceId]);

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

  const filteredVersions = useMemo(() => {
     if (!manifest) return [];
     return manifest.versions.filter((v) => activeTypes.has(v.type));
   }, [manifest, activeTypes]);

   const instanceOptions: SelectOption[] = instances.map((i) => ({ value: i.id, label: i.id }));
   const versionOptions: SelectOption[] = filteredVersions.map((v) => ({ value: v.id, label: v.id }));

   const versionPlaceholder =
     status === "loading" ? "Loading versions…" : status === "unavailable" ? "Couldn't load versions" : "Choose a version…";

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
    setInstalling(true);
    setProgress(null);
    try {
      await invoke("install_version", { versionId: versionToInstall });
      await refreshInstances();
      setSelectedInstanceId(versionToInstall);
      setVersionToInstall("");
    } catch (err) {
      console.warn("install_version failed:", err);
    } finally {
      setInstalling(false);
    }
  }

  async function handlePlay() {
    if (!selectedInstanceId) return;
    try {
      await invoke("launch_instance", { versionId: selectedInstanceId });
    } catch (err) {
      console.warn("launch_instance failed:", err);
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

          <div className="install-row">
            <Select
              value={versionToInstall}
              onChange={setVersionToInstall}
              options={versionOptions}
              placeholder={versionPlaceholder}
              disabled={installing}
              className="version-select"
              aria-label="Version to install" />

            {status === "unavailable" && !installing && (
                <button type="button" className="btn btn-ghost" onClick={retry}>
                  <RefreshIcon />
                    Retry
                </button>
                )}

            <button
              type="button"
              className="btn btn-primary"
              disabled={!versionToInstall || installing}
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
          label={`Installing ${versionToInstall}…`}
          downloadedBytes={progress.downloadedBytes}
          totalBytes={progress.totalBytes}
          filesDone={progress.filesDone}
          filesTotal={progress.filesTotal}
        />
      )}

      {selectedInstance ? (
        <div className="slot instance-card">
          <CubeIcon className={`instance-cube cube-${chipVariant(selectedInstance.type)}`} />
          <div className="instance-info">
            <div className="instance-id">{selectedInstance.id}</div>
            <span className={`tag ${tagClassFor(selectedInstance.type)}`}>
              {TYPE_LABELS[selectedInstance.type]}
            </span>
          </div>
          <div className="instance-actions">
            <button type="button" className="btn btn-primary" onClick={handlePlay}>
              <PlayIcon />
              Play
            </button>
          </div>
        </div>
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
