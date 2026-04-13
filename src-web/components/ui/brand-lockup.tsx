"use client";

interface BrandLockupProps {
  size?: "small" | "medium" | "large";
}

export function BrandLockup({ size = "medium" }: BrandLockupProps) {
  const scale = size === "small" ? 0.85 : size === "large" ? 1.25 : 1;
  const fontSize = `${(1.1 * scale).toFixed(2)}rem`;
  const subFontSize = `${(0.7 * scale).toFixed(2)}rem`;
  const logoSize = `${(28 * scale).toFixed(0)}px`;

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 10 * scale }}>
      <div
        style={{
          width: logoSize,
          height: logoSize,
          borderRadius: 8 * scale,
          background: "linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          fontWeight: 800,
          fontSize: `${(0.8 * scale).toFixed(2)}rem`,
          color: "#fff",
          letterSpacing: "-0.02em",
          flexShrink: 0,
        }}
      >
        H
      </div>
      <div>
        <div
          style={{
            fontSize,
            fontWeight: 800,
            letterSpacing: "-0.02em",
            lineHeight: 1.2,
            color: "#f0f4f8",
          }}
        >
          HAQLY
        </div>
        <div
          style={{
            fontSize: subFontSize,
            fontWeight: 400,
            color: "#64748b",
            letterSpacing: "0.08em",
            textTransform: "uppercase",
          }}
        >
          ERP
        </div>
      </div>
    </div>
  );
}
