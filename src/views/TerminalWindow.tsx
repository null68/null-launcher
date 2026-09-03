import { useEffect, useRef, useState } from "react";
import { emit, listen } from "@tauri-apps/api/event";

import "../styles/tokens.css";
import "../styles/global.css";
import "../styles/TerminalWindow.css";

const MAX_LINES = 4000;

export function TerminalWindow() {
  const [lines, setLines] = useState<string[]>([]);
  const logRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const unlisten = listen<string>("mc-log", (event) => {
      setLines((prev) => {
        const next = [...prev, event.payload];
        return next.length > MAX_LINES ? next.slice(next.length - MAX_LINES) : next;
      });
    });
    emit("terminal-ready");
    return () => {
      unlisten.then((f) => f());
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
