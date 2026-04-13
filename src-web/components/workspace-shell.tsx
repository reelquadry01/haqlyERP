"use client";

import { useState, ReactNode } from "react";
import { NAV_ITEMS, NavItem } from "@/lib/navigation";
import { getToken, getCompanyContext, clearToken } from "@/lib/session";
import { BrandLockup } from "@/components/ui/brand-lockup";
import {
  ChevronDown,
  Bell,
  LogOut,
  User,
  Building2,
  Menu,
  X,
} from "lucide-react";

interface WorkspaceShellProps {
  children: ReactNode;
}

export function WorkspaceShell({ children }: WorkspaceShellProps) {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [userMenuOpen, setUserMenuOpen] = useState(false);
  const [companyMenuOpen, setCompanyMenuOpen] = useState(false);
  const [currentPath, setCurrentPath] = useState(
    typeof window !== "undefined" ? window.location.pathname : "/dashboard"
  );

  function handleNav(path: string) {
    setCurrentPath(path);
    window.location.href = path;
  }

  function handleLogout() {
    clearToken();
    window.location.replace("/login");
  }

  return (
    <div style={{ display: "flex", height: "100vh", overflow: "hidden" }}>
      {/* Sidebar */}
      <aside
        style={{
          width: sidebarOpen ? 260 : 0,
          minWidth: sidebarOpen ? 260 : 0,
          background: "#111827",
          borderRight: "1px solid #2a3754",
          overflow: "hidden",
          transition: "width 0.2s, min-width 0.2s",
          display: "flex",
          flexDirection: "column",
        }}
      >
        <div style={{ padding: "16px 16px 12px", borderBottom: "1px solid #1e2a3e" }}>
          <BrandLockup size="small" />
        </div>

        <nav className="scroll-y" style={{ flex: 1, padding: "8px 0" }}>
          {NAV_ITEMS.map((item: NavItem) => {
            const Icon = item.icon;
            const isActive = currentPath.startsWith(item.path);
            return (
              <button
                key={item.path}
                onClick={() => handleNav(item.path)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 10,
                  width: "100%",
                  padding: "9px 16px",
                  fontSize: "0.85rem",
                  color: isActive ? "#f0f4f8" : "#94a3b8",
                  background: isActive ? "#2a3754" : "transparent",
                  borderLeft: isActive ? "3px solid #3b82f6" : "3px solid transparent",
                  transition: "all 0.1s",
                }}
                onMouseEnter={(e) => {
                  if (!isActive) (e.currentTarget.style.background = "#1a2234");
                }}
                onMouseLeave={(e) => {
                  if (!isActive) (e.currentTarget.style.background = "transparent");
                }}
              >
                <Icon size={16} />
                {item.label}
              </button>
            );
          })}
        </nav>

        <div style={{ padding: 12, borderTop: "1px solid #1e2a3e" }}>
          <button
            onClick={handleLogout}
            className="btn btn-ghost"
            style={{ width: "100%", justifyContent: "flex-start", gap: 8, fontSize: "0.8rem" }}
          >
            <LogOut size={14} />
            Sign Out
          </button>
        </div>
      </aside>

      {/* Main Content Area */}
      <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
        {/* Top Bar */}
        <header
          style={{
            height: 56,
            minHeight: 56,
            background: "#111827",
            borderBottom: "1px solid #2a3754",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "0 20px",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
            <button
              className="btn btn-ghost btn-sm"
              onClick={() => setSidebarOpen(!sidebarOpen)}
              aria-label="Toggle sidebar"
            >
              {sidebarOpen ? <X size={18} /> : <Menu size={18} />}
            </button>

            <div style={{ position: "relative" }}>
              <button
                className="btn btn-ghost btn-sm"
                onClick={() => setCompanyMenuOpen(!companyMenuOpen)}
                style={{ display: "flex", alignItems: "center", gap: 6 }}
              >
                <Building2 size={14} />
                <span style={{ fontSize: "0.85rem" }}>
                  {getCompanyContext() || "Select Company"}
                </span>
                <ChevronDown size={12} />
              </button>

              {companyMenuOpen && (
                <div
                  style={{
                    position: "absolute",
                    top: "100%",
                    left: 0,
                    marginTop: 4,
                    background: "#1a2234",
                    border: "1px solid #2a3754",
                    borderRadius: 8,
                    minWidth: 200,
                    boxShadow: "0 8px 24px rgba(0,0,0,0.5)",
                    zIndex: 100,
                  }}
                >
                  <div style={{ padding: "8px 12px", fontSize: "0.75rem", color: "#64748b" }}>
                    Companies
                  </div>
                  <button
                    className="btn btn-ghost"
                    style={{ width: "100%", justifyContent: "flex-start", padding: "8px 12px", fontSize: "0.85rem" }}
                    onClick={() => setCompanyMenuOpen(false)}
                  >
                    Current Company
                  </button>
                </div>
              )}
            </div>
          </div>

          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <button className="btn btn-ghost btn-sm" style={{ position: "relative" }}>
              <Bell size={16} />
              <span
                style={{
                  position: "absolute",
                  top: 4,
                  right: 4,
                  width: 6,
                  height: 6,
                  borderRadius: "50%",
                  background: "#ef4444",
                }}
              />
            </button>

            <div style={{ position: "relative" }}>
              <button
                className="btn btn-ghost btn-sm"
                onClick={() => setUserMenuOpen(!userMenuOpen)}
                style={{ display: "flex", alignItems: "center", gap: 6 }}
              >
                <div
                  style={{
                    width: 28,
                    height: 28,
                    borderRadius: "50%",
                    background: "#2a3754",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                  }}
                >
                  <User size={14} />
                </div>
                <ChevronDown size={12} />
              </button>

              {userMenuOpen && (
                <div
                  style={{
                    position: "absolute",
                    top: "100%",
                    right: 0,
                    marginTop: 4,
                    background: "#1a2234",
                    border: "1px solid #2a3754",
                    borderRadius: 8,
                    minWidth: 180,
                    boxShadow: "0 8px 24px rgba(0,0,0,0.5)",
                    zIndex: 100,
                  }}
                >
                  <button
                    className="btn btn-ghost"
                    style={{ width: "100%", justifyContent: "flex-start", padding: "8px 12px", fontSize: "0.85rem" }}
                    onClick={() => setUserMenuOpen(false)}
                  >
                    <User size={14} />
                    Profile
                  </button>
                  <button
                    className="btn btn-ghost"
                    style={{
                      width: "100%",
                      justifyContent: "flex-start",
                      padding: "8px 12px",
                      fontSize: "0.85rem",
                      color: "#ef4444",
                    }}
                    onClick={handleLogout}
                  >
                    <LogOut size={14} />
                    Sign Out
                  </button>
                </div>
              )}
            </div>
          </div>
        </header>

        {/* Content */}
        <main className="scroll-y" style={{ flex: 1, background: "#0a0e17" }}>
          {children}
        </main>
      </div>
    </div>
  );
}
