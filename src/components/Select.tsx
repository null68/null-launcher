import type { ReactNode } from "react";
import { ChevronDownIcon } from "./icons";

interface SelectProps {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  className?: string;
  "aria-label"?: string;
  children: ReactNode;
}

export function Select({ value, onChange, disabled, className, children, ...rest }: SelectProps) {
  return (
    <div className={className ? `select-wrap ${className}` : "select-wrap"}>
      <select
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value)}
        {...rest}
      >
        {children}
      </select>
      <ChevronDownIcon className="chev" />
    </div>
  );
}
