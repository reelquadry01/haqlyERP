"use client";

import { BrandLockup } from "@/components/ui/brand-lockup";

export function AppSplash() {
  return (
    <div
      className="flex-center full-viewport"
      style={{
        background: "#0a0e17",
        flexDirection: "column",
        gap: 24,
      }}
    >
      <BrandLockup size="large" />
      <div
        style={{
          width: 32,
          height: 32,
          border: "3px solid #2a3754",
          borderTopColor: "#3b82f6",
          borderRadius: "50%",
          animation: "spin 0.8s linear infinite",
        }}
      />
      <p style={{ color: "#64748b", fontSize: "0.85rem" }}>Loading HAQLY ERP...</p>
    </div>
  );
}
