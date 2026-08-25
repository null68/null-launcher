import { useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useScreenshots } from "../hooks/useScreenshots";
import { EmptyState } from "../components/EmptyState";
import { writeImage } from "@tauri-apps/plugin-clipboard-manager";
import { ContextMenu } from "../components/ContextMenu";
import { Lightbox } from "../components/LightBox";
import { ImageIcon, CopyIcon } from "../components/icons";

import type { Screenshot } from "../types";
import type { MouseEvent as ReactMouseEvent } from "react";

import "../styles/ScreenshotsView.css";

interface MenuState {
  x: number;
  y: number;
  shot: Screenshot;
}

export function ScreenshotsView() {
  const { screenshots, loading } = useScreenshots();
  const [lightboxIndex, setLightboxIndex] = useState<number | null>(null);
  const [menu, setMenu] = useState<MenuState | null>(null);

  function openMenu(e: ReactMouseEvent, shot: Screenshot) {
    e.preventDefault();
    setMenu({ x: e.clientX, y: e.clientY, shot });
  }

  async function handleCopyImage() {
    const shot = menu?.shot;
    setMenu(null);
    if (!shot) return;
    try {
      await writeImage(shot.path);
    } catch (err) {
      console.warn("copy image failed:", err);
    }
  }

  return (
    <section className="view screenshots-view">
      <div className="view-header">
        <h1>Screenshots</h1>
        <div className="view-subtitle">From your .minecraft/screenshots folder.</div>
      </div>

      {loading ? (
        <div className="view-loading">Loading…</div>
      ) : screenshots.length === 0 ? (
        <EmptyState
          icon={<ImageIcon className="icon" />}
          title="No screenshots found"
          body="Press F2 in-game to take one — it'll show up here."
        />
      ) : (
        <div className="shot-grid">
          {screenshots.map((shot, i) => (
            <button
              key={shot.path}
              type="button"
              className="shot-thumb"
              onClick={() => setLightboxIndex(i)}
              onContextMenu={(e) => openMenu(e, shot)}
            >
              <span className="frame">
                <img src={convertFileSrc(shot.path)} alt={shot.name} loading="lazy" />
              </span>
              <span className="name">{shot.name}</span>
            </button>
          ))}
        </div>
      )}

      {lightboxIndex !== null && screenshots[lightboxIndex] && (
        <Lightbox
          screenshots={screenshots}
          index={lightboxIndex}
          onClose={() => setLightboxIndex(null)}
          onIndexChange={setLightboxIndex}
          onContextMenu={openMenu}
        />
      )}

      {menu && (
        <ContextMenu x={menu.x} y={menu.y} onClose={() => setMenu(null)}>
          <button type="button" role="menuitem" className="context-menu-item" onClick={handleCopyImage}>
            <CopyIcon />
            Copy image
          </button>
        </ContextMenu>
      )}
    </section>
  );
}
