import { useEffect, useId, useRef, useState } from "react";
import type { KeyboardEvent } from "react";
import { ChevronDownIcon } from "./icons";

export interface SelectOption {
  value: string;
  label: string;
}

interface SelectProps {
  value: string;
  onChange: (value: string) => void;
  options: SelectOption[];
  placeholder: string;
  disabled?: boolean;
  className?: string;
  "aria-label"?: string;
}

export function Select({
  value,
  onChange,
  options,
  placeholder,
  disabled,
  className,
  "aria-label": ariaLabel,
}: SelectProps) {
  const [open, setOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(0);
  const rootRef = useRef<HTMLDivElement>(null);
  const optionRefs = useRef<(HTMLLIElement | null)[]>([]);
  const baseId = useId();

  const selected = options.find((o) => o.value === value) ?? null;
  const canOpen = !disabled && options.length > 0;

  useEffect(() => {
    if (!open) return;
    function onDocPointerDown(e: MouseEvent) {
      if (rootRef.current && !rootRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", onDocPointerDown);
    return () => document.removeEventListener("mousedown", onDocPointerDown);
  }, [open]);

  function openPanel() {
    if (!canOpen) return;
    const idx = Math.max(
      0,
      options.findIndex((o) => o.value === value)
    );
    setActiveIndex(idx);
    setOpen(true);
  }

  function commit(index: number) {
    const opt = options[index];
    if (!opt) return;
    onChange(opt.value);
    setOpen(false);
  }

  function move(delta: number) {
    setActiveIndex((i) => {
      const next = Math.min(options.length - 1, Math.max(0, i + delta));
      optionRefs.current[next]?.scrollIntoView({ block: "nearest" });
      return next;
    });
  }

  function onTriggerKeyDown(e: KeyboardEvent<HTMLButtonElement>) {
    if (!open) {
      if (e.key === "ArrowDown" || e.key === "ArrowUp" || e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        openPanel();
      }
      return;
    }
    switch (e.key) {
      case "Escape":
        e.preventDefault();
        setOpen(false);
        break;
      case "ArrowDown":
        e.preventDefault();
        move(1);
        break;
      case "ArrowUp":
        e.preventDefault();
        move(-1);
        break;
      case "Enter":
      case " ":
        e.preventDefault();
        commit(activeIndex);
        break;
      case "Tab":
        setOpen(false);
        break;
    }
  }

  return (
    <div className={className ? `select-wrap ${className}` : "select-wrap"} ref={rootRef}>
      <button
        type="button"
        className="select-trigger"
        disabled={disabled || options.length === 0}
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-label={ariaLabel}
        aria-activedescendant={open ? `${baseId}-opt-${activeIndex}` : undefined}
        onClick={() => (open ? setOpen(false) : openPanel())}
        onKeyDown={onTriggerKeyDown}
      >
        <span className={selected ? "select-value" : "select-value placeholder"}>
          {selected ? selected.label : placeholder}
        </span>
        <ChevronDownIcon className="chev" />
      </button>

      {open && (
        <ul className="select-panel" role="listbox" aria-label={ariaLabel}>
          {options.map((opt, i) => (
            <li
              key={opt.value}
              id={`${baseId}-opt-${i}`}
              ref={(el) => {
                optionRefs.current[i] = el;
              }}
              role="option"
              aria-selected={opt.value === value}
              className={i === activeIndex ? "select-option active" : "select-option"}
              onMouseEnter={() => setActiveIndex(i)}
              onClick={() => commit(i)}
            >
              {opt.label}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
