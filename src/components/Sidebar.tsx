import type { ComponentType } from "react";
import type { ViewId } from "../App";
import { CubeIcon, ImageIcon, SettingsIcon } from "./icons";

interface SidebarProps {
  active: ViewId;
  onSelect: (view: ViewId) => void;
}

const NAV_ITEMS: { id: ViewId; label: string; Icon: ComponentType<{ className?: string }> }[] = [
  { id: "instances", label: "Instances", Icon: CubeIcon },
  { id: "screenshots", label: "Screenshots", Icon: ImageIcon },
  { id: "settings", label: "Settings", Icon: SettingsIcon },
];

export function Sidebar({ active, onSelect }: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <img src="/copper_block.png" alt="" />
        <span className="brand-name">
          null-launcher<span className="brand-cursor" />
        </span>
      </div>

      <ul>
        {NAV_ITEMS.map(({ id, label, Icon }) => (
          <li key={id}>
            <button
              type="button"
              className={active === id ? "nav-item active" : "nav-item"}
              aria-current={active === id ? "page" : undefined}
              onClick={() => onSelect(id)}
            >
              <Icon />
              <span>{label}</span>
            </button>
          </li>
        ))}
      </ul>

      <div className="sidebar-foot">v0.1.0</div>
    </aside>
  );
}
