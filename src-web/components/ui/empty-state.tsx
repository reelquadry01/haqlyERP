"use client";

import { ReactNode } from "react";
import { Inbox } from "lucide-react";

interface EmptyStateProps {
  icon?: ReactNode;
  title: string;
  description?: string;
  action?: ReactNode;
}

export function EmptyState({ icon, title, description, action }: EmptyStateProps) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        padding: "48px 24px",
        textAlign: "center",
      }}
    >
      <div
        style={{
          width: 56,
          height: 56,
          borderRadius: 16,
          background: "rgba(59,130,246,0.12)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "#3b82f6",
          marginBottom: 16,
        }}
      >
        {icon || <Inbox size={24} />}
      </div>
      <h3
        style={{
          fontSize: "1rem",
          fontWeight: 600,
          color: "#f0f4f8",
          marginBottom: 4,
        }}
      >
        {title}
      </h3>
      {description && (
        <p
          style={{
            fontSize: "0.85rem",
            color: "#64748b",
            maxWidth: 360,
            lineHeight: 1.5,
          }}
        >
          {description}
        </p>
      )}
      {action && <div style={{ marginTop: 16 }}>{action}</div>}
    </div>
  );
}
