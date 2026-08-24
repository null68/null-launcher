import type { ReactNode } from "react";

interface EmptyStateProps {
  icon: ReactNode;
  title: string;
  body: ReactNode;
}

export function EmptyState({ icon, title, body }: EmptyStateProps) {
  return (
    <div className="empty-state">
      {icon}
      <h3>{title}</h3>
      <p>{body}</p>
    </div>
  );
}
