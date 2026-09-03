import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import type { ComponentType } from "react";
import type { ViewId } from "../App";
import { CubeIcon, ImageIcon, SettingsIcon } from "./icons";

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

      <div className="sidebar-foot">
        {version ? `v${version}` : "v—"}
      </div>
    </aside>
  );
}
