import { CubeIcon } from "./icons";
import { ProgressBar } from "./ProgressBar";
import type { InstallProgressState } from "../hooks/useInstallManager";

interface LaunchOverlayProps {
  status: string;
  progress: InstallProgressState | null;
}

export function LaunchOverlay({ status, progress }: LaunchOverlayProps) {
  return (
    <div className="launch-overlay">
      <CubeIcon className="launch-overlay-cube spin" />
      <div className="launch-overlay-status">{status}</div>
      {progress && (
        <ProgressBar
          label="Getting everything ready"
          downloadedBytes={progress.downloadedBytes}
          totalBytes={progress.totalBytes}
          filesDone={progress.filesDone}
          filesTotal={progress.filesTotal}
        />
      )}
    </div>
  );
}
