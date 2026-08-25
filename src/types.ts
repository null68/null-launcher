export type VersionType = "release" | "snapshot" | "old_beta" | "old_alpha";

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
