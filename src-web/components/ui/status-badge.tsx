"use client";

type StatusType =
  | "draft"
  | "pending"
  | "submitted"
  | "approved"
  | "rejected"
  | "posted"
  | "active"
  | "inactive"
  | "paid"
  | "unpaid"
  | "overdue"
  | "partial"
  | "cancelled"
  | "completed"
  | "processing";

interface StatusBadgeProps {
  status: StatusType | string;
  size?: "sm" | "md" | "lg";
}

const STATUS_COLORS: Record<string, { bg: string; color: string }> = {
  draft: { bg: "rgba(100,116,139,0.15)", color: "#94a3b8" },
  pending: { bg: "rgba(245,158,11,0.15)", color: "#f59e0b" },
  submitted: { bg: "rgba(59,130,246,0.15)", color: "#3b82f6" },
  approved: { bg: "rgba(6,182,212,0.15)", color: "#06b6d4" },
  rejected: { bg: "rgba(239,68,68,0.15)", color: "#ef4444" },
  posted: { bg: "rgba(16,185,129,0.15)", color: "#10b981" },
  active: { bg: "rgba(16,185,129,0.15)", color: "#10b981" },
  inactive: { bg: "rgba(100,116,139,0.15)", color: "#64748b" },
  paid: { bg: "rgba(16,185,129,0.15)", color: "#10b981" },
  unpaid: { bg: "rgba(245,158,11,0.15)", color: "#f59e0b" },
  overdue: { bg: "rgba(239,68,68,0.15)", color: "#ef4444" },
  partial: { bg: "rgba(139,92,246,0.15)", color: "#8b5cf6" },
  cancelled: { bg: "rgba(100,116,139,0.15)", color: "#64748b" },
  completed: { bg: "rgba(16,185,129,0.15)", color: "#10b981" },
  processing: { bg: "rgba(59,130,246,0.15)", color: "#3b82f6" },
};

const SIZE_MAP = {
  sm: { padding: "1px 6px", fontSize: "0.65rem", dotSize: 5 },
  md: { padding: "2px 8px", fontSize: "0.75rem", dotSize: 6 },
  lg: { padding: "4px 12px", fontSize: "0.85rem", dotSize: 7 },
};

export function StatusBadge({ status, size = "md" }: StatusBadgeProps) {
  const colors = STATUS_COLORS[status] || STATUS_COLORS.draft;
  const sizeConf = SIZE_MAP[size];
  const label = status.charAt(0).toUpperCase() + status.slice(1);

  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 4,
        padding: sizeConf.padding,
        borderRadius: 12,
        fontSize: sizeConf.fontSize,
        fontWeight: 600,
        background: colors.bg,
        color: colors.color,
        whiteSpace: "nowrap",
        lineHeight: 1.4,
      }}
    >
      <span
        style={{
          width: sizeConf.dotSize,
          height: sizeConf.dotSize,
          borderRadius: "50%",
          background: colors.color,
          flexShrink: 0,
        }}
      />
      {label}
    </span>
  );
}
