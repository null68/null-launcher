export type VersionType = "release" | "snapshot" | "old_beta" | "old_alpha";
export type Loader = "vanilla" | "fabric" | "quilt" | "forge" | "neoforge" | "optifine";

export interface Version {
  id: string;
  url: string;
  sha1: string;
  type: VersionType;
}

export interface VersionManifest {
  versions: Version[];
}

export interface Instance {
  id: string;
  type: VersionType;
  installedAt?: string;
  loader?: Loader | null;
}

export interface InstallProgressPayload {
  downloaded_bytes: number;
  total_bytes: number;
  files_done: number;
  files_total: number;
}

export interface Screenshot {
  name: string;
  path: string;
}

export interface LauncherSettings {
  username: string;
  minMemoryMb: number;
  maxMemoryMb: number;
  terminalMode: boolean;
}
