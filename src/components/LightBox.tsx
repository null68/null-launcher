import { useEffect, useRef } from "react";
import type { MouseEvent as ReactMouseEvent } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ChevronLeftIcon, ChevronRightIcon, CloseIcon } from "./icons";
import type { Screenshot } from "../types";

interface LightboxProps {
  screenshots: Screenshot[];
  index: number;
  onClose: () => void;
  onIndexChange: (index: number) => void;
  onContextMenu: (e: ReactMouseEvent, shot: Screenshot) => void;
}

export function Lightbox({ screenshots, index, onClose, onIndexChange, onContextMenu }: LightboxProps) {
  const closeRef = useRef<HTMLButtonElement>(null);
  const shot = screenshots[index];
  const hasPrev = index > 0;
  const hasNext = index < screenshots.length - 1;

  useEffect(() => {
    const previouslyFocused = document.activeElement as HTMLElement | null;
    closeRef.current?.focus();
    return () => previouslyFocused?.focus();
  }, []);

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
      else if (e.key === "ArrowRight" && hasNext) onIndexChange(index + 1);
      else if (e.key === "ArrowLeft" && hasPrev) onIndexChange(index - 1);
    }
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [index, hasPrev, hasNext, onClose, onIndexChange]);

  if (!shot) return null;

  return (
    <div className="lightbox-backdrop" onClick={onClose}>
      <div
        className="lightbox-body"
        role="dialog"
        aria-modal="true"
        aria-label={`Screenshot viewer — ${shot.name}`}
        onClick={(e) => e.stopPropagation()}
      >
        <img
          src={convertFileSrc(shot.path)}
          alt={shot.name}
          onContextMenu={(e) => onContextMenu(e, shot)}
        />
        <div className="lightbox-caption">
          <span className="lightbox-name">{shot.name}</span>
          <span className="lightbox-count">
            {index + 1} / {screenshots.length}
          </span>
        </div>
      </div>

      <button ref={closeRef} type="button" className="lightbox-close" onClick={onClose} aria-label="Close viewer">
        <CloseIcon />
      </button>

      <button
        type="button"
        className="lightbox-nav prev"
        onClick={(e) => {
          e.stopPropagation();
          if (hasPrev) onIndexChange(index - 1);
        }}
        disabled={!hasPrev}
        aria-label="Previous screenshot"
      >
        <ChevronLeftIcon />
      </button>

      <button
        type="button"
        className="lightbox-nav next"
        onClick={(e) => {
          e.stopPropagation();
          if (hasNext) onIndexChange(index + 1);
        }}
        disabled={!hasNext}
        aria-label="Next screenshot"
      >
        <ChevronRightIcon />
      </button>
    </div>
  );
}
