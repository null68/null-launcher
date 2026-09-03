import type { ChangeEvent } from "react";
import { useSettings } from "../hooks/useSettings";
import "../styles/SettingsView.css";

const MEM_MIN = 512;
const MEM_MAX = 16384;
const MEM_STEP = 256;
const MEM_SCALE_MAX = 16384;

function formatMb(mb: number): string {
  return `${mb} MB`;
}

export function SettingsView() {
  const { settings, update } = useSettings();

  function handleUsername(e: ChangeEvent<HTMLInputElement>) {
    update({ username: e.target.value });
  }

  function handleMin(e: ChangeEvent<HTMLInputElement>) {
    const value = Number(e.target.value);
    update({ minMemoryMb: value, maxMemoryMb: Math.max(value, settings.maxMemoryMb) });
  }

  function handleMax(e: ChangeEvent<HTMLInputElement>) {
    const value = Number(e.target.value);
    update({ maxMemoryMb: value, minMemoryMb: Math.min(value, settings.minMemoryMb) });
  }

  const barLeft = (settings.minMemoryMb / MEM_SCALE_MAX) * 100;
  const barWidth = ((settings.maxMemoryMb - settings.minMemoryMb) / MEM_SCALE_MAX) * 100;

  return (
    <section className="view settings-view">
      <div className="view-header">
        <h1>Settings</h1>
      </div>

      <div className="settings-section">
        <div className="section-title">Profile</div>
        <div className="section-hint">
          Used as your in-game name — this launcher runs in offline mode, so there's no account login.
        </div>
        <div className="field">
          <label className="eyebrow" htmlFor="username">
            Username
          </label>
          <input
            id="username"
            className="text-input"
            value={settings.username}
            onChange={handleUsername}
            placeholder="Steve"
            maxLength={16}
            spellCheck={false}
            autoComplete="off"
          />
        </div>
      </div>

      <div className="settings-section">
        <div className="section-title">Memory allocation</div>
        <div className="section-hint">How much RAM the game can use. Applies to whichever instance you launch.</div>

        <div className="mem-row">
          <div className="mem-top">
            <span className="mem-name">
              Minimum <span className="sub">−Xms</span>
            </span>
            <span className="mem-value">{formatMb(settings.minMemoryMb)}</span>
          </div>
          <input
            type="range"
            min={MEM_MIN}
            max={MEM_MAX}
            step={MEM_STEP}
            value={settings.minMemoryMb}
            onChange={handleMin}
            aria-label="Minimum memory, Xms"
          />
        </div>

        <div className="mem-row">
          <div className="mem-top">
            <span className="mem-name">
              Maximum <span className="sub">−Xmx</span>
            </span>
            <span className="mem-value">{formatMb(settings.maxMemoryMb)}</span>
          </div>
          <input
            type="range"
            min={MEM_MIN}
            max={MEM_MAX}
            step={MEM_STEP}
            value={settings.maxMemoryMb}
            onChange={handleMax}
            aria-label="Maximum memory, Xmx"
          />
        </div>

        <div className="mem-bar" aria-hidden="true">
          <div className="mem-bar-fill" style={{ left: `${barLeft}%`, width: `${barWidth}%` }} />
        </div>
        <div className="mem-ticks" aria-hidden="true">
          <span>0</span>
          <span>4 GB</span>
          <span>8 GB</span>
          <span>12 GB</span>
          <span>16 GB</span>
        </div>
      </div>

      <div className="settings-section">
        <div className="section-title">Terminal mode</div>
        <div className="section-hint">Opens a console window with live output whenever you launch a version.</div>
        <button
          type="button"
          role="switch"
          aria-checked={settings.terminalMode}
          className={settings.terminalMode ? "switch on" : "switch"}
          onClick={() => update({ terminalMode: !settings.terminalMode })}
        >
          <span className="switch-knob" />
        </button>
      </div>
    </section>
  );
}
