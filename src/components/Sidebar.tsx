import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import type { ComponentType } from "react";
import type { ViewId } from "../App";
import { CubeIcon, ImageIcon, SettingsIcon, UpdateIcon } from "./icons";
import { useUpdater } from "../hooks/useUpdater";

interface SidebarProps {
  active: ViewId;
  onSelect: (view: ViewId) => void;
  installing?: boolean;
}

const NAV_ITEMS: {
  id: ViewId;
  label: string;
  Icon: ComponentType<{ className?: string }>;
}[] = [
  {
    id: "instances",
    label: "Instances",
    Icon: CubeIcon,
  },
  {
    id: "screenshots",
    label: "Screenshots",
    Icon: ImageIcon,
  },
  {
    id: "settings",
    label: "Settings",
    Icon: SettingsIcon,
  },
];

export function Sidebar({
  active,
  onSelect,
  installing,
}: SidebarProps) {
  const [version, setVersion] = useState<string | null>(null);
  const updater = useUpdater();

  const updateBusy =
    updater.status === "checking" ||
    updater.status === "downloading" ||
    updater.status === "relaunching";

  const updateLabel =
    updater.status === "downloading"
      ? `Updating… ${updater.progress}%`
      : updater.status === "relaunching"
        ? "Restarting…"
        : "Update";

  const handleUpdateClick = () => {
    if (updater.status === "available") {
      updater.installAndRelaunch();
    } else if (updater.status === "idle" || updater.status === "error") {
      updater.checkForUpdate();
    }
  };

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch((error) => {
        console.error(
          "Failed to get application version:",
          error
        );
      });
  }, []);

  return (
    <aside className="sidebar">
      <div className="brand">
        <img src="/copper_block.png" alt="" />

        <span className="brand-name">
          null-launcher
          <span className="brand-cursor" />
        </span>
      </div>

      <ul>
        {NAV_ITEMS.map(({ id, label, Icon }) => (
          <li key={id}>
            <button
              type="button"
              className={
                active === id
                  ? "nav-item active"
                  : "nav-item"
              }
              aria-current={
                active === id ? "page" : undefined
              }
              onClick={() => onSelect(id)}
            >
              <Icon />

              <span>{label}</span>

              {id === "instances" && installing && (
                <span
                  className="nav-badge"
                  title="Installing…"
                />
              )}
            </button>
          </li>
        ))}
      </ul>

      <div className="sidebar-bottom">
        <button
          type="button"
          className={
            updater.status === "available"
              ? "nav-item update-item has-update"
              : "nav-item update-item"
          }
          disabled={updateBusy}
          onClick={handleUpdateClick}
          title={
            updater.status === "available"
              ? `Update to v${updater.version} available`
              : undefined
          }
        >
          <UpdateIcon className={updateBusy ? "spin" : undefined} />
          <span>{updateLabel}</span>
          {updater.status === "available" && (
            <span className="nav-badge update-badge" title="Update available" />
          )}
        </button>

        <div className="sidebar-foot">
          {version ? `v${version}` : "v—"}
        </div>
      </div>
    </aside>
  );
}
