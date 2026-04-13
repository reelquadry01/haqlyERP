"use client";

import { useState, useRef, useEffect, ReactNode } from "react";

interface ActionMenuItem {
  label: string;
  icon?: ReactNode;
  onClick: () => void;
  variant?: "default" | "danger";
  disabled?: boolean;
}

interface ActionMenuProps {
  items: ActionMenuItem[];
  trigger?: ReactNode;
  align?: "left" | "right";
}

export function ActionMenu({ items, trigger, align = "right" }: ActionMenuProps) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    if (open) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open]);

  return (
    <div ref={containerRef} style={{ position: "relative", display: "inline-block" }}>
      <button
        className="btn btn-ghost btn-sm"
        onClick={() => setOpen(!open)}
        aria-haspopup="true"
        aria-expanded={open}
      >
        {trigger || "•••"}
      </button>

      {open && (
        <div
          style={{
            position: "absolute",
            top: "100%",
            [align]: 0,
            marginTop: 4,
            background: "#1a2234",
            border: "1px solid #2a3754",
            borderRadius: 8,
            minWidth: 170,
            boxShadow: "0 8px 24px rgba(0,0,0,0.5)",
            zIndex: 200,
            overflow: "hidden",
          }}
          className="fade-in"
        >
          {items.map((item, i) => (
            <button
              key={i}
              disabled={item.disabled}
              className="btn btn-ghost"
              style={{
                width: "100%",
                justifyContent: "flex-start",
                gap: 8,
                padding: "8px 12px",
                fontSize: "0.85rem",
                color:
                  item.variant === "danger"
                    ? "#ef4444"
                    : item.disabled
                    ? "#64748b"
                    : "#f0f4f8",
                cursor: item.disabled ? "not-allowed" : "pointer",
                opacity: item.disabled ? 0.5 : 1,
              }}
              onClick={() => {
                if (!item.disabled) {
                  item.onClick();
                  setOpen(false);
                }
              }}
            >
              {item.icon}
              {item.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
