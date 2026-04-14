"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet, apiPost, apiPatch } from "@/lib/api";
import { Settings, Plus, Shield, Calendar, Database } from "lucide-react";

interface User {
  id: string;
  email: string;
  full_name: string;
  role: string;
  is_active: boolean;
  mfa_enabled: boolean;
  last_login_at: string | null;
}

interface Role {
  id: string;
  name: string;
  description: string;
  is_system: boolean;
  permissions_count: number;
}

interface FiscalYear {
  id: string;
  name: string;
  start_date: string;
  end_date: string;
  status: string;
}

const TABS = ["users", "roles", "fiscal-years", "settings"] as const;
type Tab = (typeof TABS)[number];

export default function AdministrationPage() {
  const token = getToken();
  const [activeTab, setActiveTab] = useState<Tab>("users");
  const [users, setUsers] = useState<User[]>([]);
  const [roles, setRoles] = useState<Role[]>([]);
  const [fiscalYears, setFiscalYears] = useState<FiscalYear[]>([]);
  const [loading, setLoading] = useState(true);
  const [fetchError, setFetchError] = useState<string | null>(null);

  useEffect(() => {
    async function load() {
      try {
        const [uRes, rRes, fRes] = await Promise.all([
          apiGet("/admin/users", token),
          apiGet("/admin/roles", token),
          apiGet("/admin/fiscal-years", token),
        ]);
        if (uRes.ok) setUsers((await uRes.json()).data || []);
        if (rRes.ok) setRoles((await rRes.json()).data || []);
        if (fRes.ok) setFiscalYears((await fRes.json()).data || []);
      } catch (err) {
        setFetchError(err instanceof Error ? err.message : "Failed to load administration data");
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  const userColumns: Column<User>[] = [
    { key: "full_name", label: "Name", sortable: true },
    { key: "email", label: "Email", sortable: true, width: "200px" },
    { key: "role", label: "Role", width: "120px" },
    { key: "is_active", label: "Active", width: "70px", render: (v: boolean) => <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: v ? "rgba(25,135,84,0.12)" : "rgba(220,53,69,0.12)", color: v ? "#198754" : "#DC3545" }}>{v ? "Yes" : "No"}</span> },
    { key: "mfa_enabled", label: "MFA", width: "60px", render: (v: boolean) => v ? <Shield size={14} style={{ color: "#198754" }} /> : <span style={{ color: "#868E96", fontSize: "0.8rem" }}>Off</span> },
    { key: "last_login_at", label: "Last Login", width: "140px", render: (v: string) => v || "Never" },
  ];

  const roleColumns: Column<Role>[] = [
    { key: "name", label: "Role", sortable: true },
    { key: "description", label: "Description" },
    { key: "is_system", label: "System", width: "70px", render: (v: boolean) => <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: v ? "rgba(27,67,50,0.12)" : "rgba(134,142,150,0.12)", color: v ? "#1B4332" : "#868E96" }}>{v ? "Yes" : "No"}</span> },
    { key: "permissions_count", label: "Permissions", align: "right", width: "100px" },
  ];

  const fyColumns: Column<FiscalYear>[] = [
    { key: "name", label: "Year", sortable: true },
    { key: "start_date", label: "Start", width: "120px" },
    { key: "end_date", label: "End", width: "120px" },
    { key: "status", label: "Status", width: "100px", render: (v: string) => { const c: Record<string, { bg: string; color: string }> = { open: { bg: "rgba(25,135,84,0.12)", color: "#198754" }, closed: { bg: "rgba(134,142,150,0.12)", color: "#868E96" }, locked: { bg: "rgba(220,53,69,0.12)", color: "#DC3545" } }; const s = c[v] || c.open; return <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: s.bg, color: s.color }}>{v}</span>; } },
  ];

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <Settings size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> Administration
          </h1>
          <button className="btn btn-primary" style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <Plus size={16} /> New {activeTab === "users" ? "User" : activeTab === "roles" ? "Role" : activeTab === "fiscal-years" ? "Fiscal Year" : "Setting"}
          </button>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: "2px solid #E9ECEF" }}>
          {(["users", "roles", "fiscal-years", "settings"] as const).map((t) => (
            <button key={t} onClick={() => setActiveTab(t)} style={{ padding: "10px 16px", fontSize: "0.85rem", fontWeight: activeTab === t ? 600 : 400, color: activeTab === t ? "#1B4332" : "#868E96", borderBottom: activeTab === t ? "2px solid #1B4332" : "2px solid transparent", marginBottom: -2, textTransform: "capitalize" }}>
              {t.replace("-", " ")}
            </button>
          ))}
        </div>

        <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
          {loading ? <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
            : fetchError ? <div style={{ textAlign: "center", padding: 40, color: "#DC3545" }}><p>{fetchError}</p></div>
            : activeTab === "users" ? <DataTable columns={userColumns} data={users} pageSize={15} emptyMessage="No users" />
            : activeTab === "roles" ? <DataTable columns={roleColumns} data={roles} pageSize={15} emptyMessage="No roles" />
            : activeTab === "fiscal-years" ? <DataTable columns={fyColumns} data={fiscalYears} pageSize={10} emptyMessage="No fiscal years" />
            : (
              <div style={{ textAlign: "center", padding: 40, color: "#868E96" }}>
                <Database size={32} style={{ marginBottom: 8 }} />
                <p>System settings configuration</p>
              </div>
            )}
        </div>
      </div>
    </WorkspaceShell>
  );
}
