import { useEffect, useRef, useState } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";

import "../styles/tokens.css";
import "../styles/global.css";
import "../styles/TerminalWindow.css";

const MAX_LINES = 4000;

export function TerminalWindow() {
  const [lines, setLines] = useState<string[]>([]);
  const logRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setLines([]); // the backlog is always replayed in full on attach
    const channel = new Channel<string>();
    channel.onmessage = (line) => {
      setLines((prev) => {
        const next = [...prev, line];
        return next.length > MAX_LINES ? next.slice(next.length - MAX_LINES) : next;
      });
    };
    invoke("console_attach", { onLine: channel });
    return () => {
      channel.onmessage = () => {};
    };
  }, []);

  useEffect(() => {
    const el = logRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [lines]);

  return (
    <div className="terminal-window">
      <div className="terminal-log" ref={logRef}>
        {lines.length === 0 ? (
          <div className="terminal-empty">Waiting for output…</div>
        ) : (
          lines.map((line, i) => (
            <div className="terminal-line" key={i}>
              {line}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
