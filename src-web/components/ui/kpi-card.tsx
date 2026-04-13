"use client";

import { ReactNode } from "react";
import { ArrowUpRight, ArrowDownRight } from "lucide-react";

interface KPICardProps {
  title: string;
  value: number;
  change: number;
  trend: "up" | "down";
  color: "primary" | "success" | "danger" | "warning" | "info";
  icon?: ReactNode;
}

const COLOR_MAP: Record<string, { bg: string; text: string; iconBg: string }> = {
  primary: { bg: "rgba(59,130,246,0.12)", text: "#3b82f6", iconBg: "rgba(59,130,246,0.2)" },
  success: { bg: "rgba(16,185,129,0.12)", text: "#10b981", iconBg: "rgba(16,185,129,0.2)" },
  danger: { bg: "rgba(239,68,68,0.12)", text: "#ef4444", iconBg: "rgba(239,68,68,0.2)" },
  warning: { bg: "rgba(245,158,11,0.12)", text: "#f59e0b", iconBg: "rgba(245,158,11,0.2)" },
  info: { bg: "rgba(6,182,212,0.12)", text: "#06b6d4", iconBg: "rgba(6,182,212,0.2)" },
};

export function KPICard({ title, value, change, trend, color, icon }: KPICardProps) {
  const colors = COLOR_MAP[color] || COLOR_MAP.primary;
  const isPositive = trend === "up";
  const formattedValue = new Intl.NumberFormat("en-NG", {
    style: "currency",
    currency: "NGN",
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(value);

  return (
    <div className="card card-hover" style={{ animation: "fadeIn 0.3s ease-out" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: 12 }}>
        <span style={{ fontSize: "0.8rem", color: "#94a3b8", fontWeight: 500 }}>{title}</span>
        {icon && (
          <div
            style={{
              width: 32,
              height: 32,
              borderRadius: 8,
              background: colors.iconBg,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              color: colors.text,
            }}
          >
            {icon}
          </div>
        )}
      </div>
      <div style={{ fontSize: "1.5rem", fontWeight: 700, fontFamily: "var(--font-mono)", marginBottom: 8 }}>
        {formattedValue}
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 4, fontSize: "0.8rem" }}>
        <span
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: 2,
            color: isPositive ? "#10b981" : "#ef4444",
          }}
        >
          {isPositive ? <ArrowUpRight size={14} /> : <ArrowDownRight size={14} />}
          {Math.abs(change).toFixed(1)}%
        </span>
        <span style={{ color: "#64748b" }}>vs last period</span>
      </div>
    </div>
  );
}
