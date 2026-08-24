interface ProgressBarProps {
  label: string;
  downloadedBytes: number;
  totalBytes: number;
  filesDone: number;
  filesTotal: number;
}

function formatBytes(bytes: number): string {
  const mb = bytes / (1024 * 1024);
  if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
  return `${Math.round(mb)} MB`;
}

export function ProgressBar({ label, downloadedBytes, totalBytes, filesDone, filesTotal }: ProgressBarProps) {
  const pct = totalBytes > 0 ? Math.min(100, (downloadedBytes / totalBytes) * 100) : 0;

  return (
    <div className="progress-block">
      <div className="progress-top">
        <span className="progress-label">{label}</span>
        <span className="progress-meta">
          {filesDone} / {filesTotal} objects · {formatBytes(downloadedBytes)} / {formatBytes(totalBytes)}
        </span>
      </div>
      <div className="progress-track">
        <div className="progress-fill" style={{ width: `${pct}%` }} />
      </div>
    </div>
  );
}
