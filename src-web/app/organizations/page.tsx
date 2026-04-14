"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet } from "@/lib/api";
import { Building2, Plus } from "lucide-react";

interface Company {
  id: string;
  name: string;
  rc_number: string;
  tin: string;
  industry: string;
  is_active: boolean;
}

interface Branch {
  id: string;
  name: string;
  company_name: string;
  city: string;
  is_active: boolean;
}

interface Department {
  id: string;
  name: string;
  branch_name: string;
  head_name: string;
  is_active: boolean;
}

export default function OrganizationsPage() {
  const token = getToken();
  const [companies, setCompanies] = useState<Company[]>([]);
  const [branches, setBranches] = useState<Branch[]>([]);
  const [departments, setDepartments] = useState<Department[]>([]);
  const [tab, setTab] = useState<"companies" | "branches" | "departments">("companies");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const [cRes, bRes, dRes] = await Promise.all([
          apiGet("/org/companies", token),
          apiGet("/org/branches", token),
          apiGet("/org/departments", token),
        ]);
        if (cRes.ok) setCompanies((await cRes.json()).data || []);
        if (bRes.ok) setBranches((await bRes.json()).data || []);
        if (dRes.ok) setDepartments((await dRes.json()).data || []);
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  const companyColumns: Column<Company>[] = [
    { key: "name", label: "Company", sortable: true },
    { key: "rc_number", label: "RC Number", width: "120px" },
    { key: "tin", label: "TIN", width: "120px" },
    { key: "industry", label: "Industry", width: "140px" },
    { key: "is_active", label: "Status", width: "80px", render: (v: boolean) => <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: v ? "rgba(25,135,84,0.12)" : "rgba(134,142,150,0.12)", color: v ? "#198754" : "#868E96" }}>{v ? "Active" : "Inactive"}</span> },
  ];

  const branchColumns: Column<Branch>[] = [
    { key: "name", label: "Branch", sortable: true },
    { key: "company_name", label: "Company", width: "160px" },
    { key: "city", label: "City", width: "120px" },
    { key: "is_active", label: "Status", width: "80px", render: (v: boolean) => <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: v ? "rgba(25,135,84,0.12)" : "rgba(134,142,150,0.12)", color: v ? "#198754" : "#868E96" }}>{v ? "Active" : "Inactive"}</span> },
  ];

  const deptColumns: Column<Department>[] = [
    { key: "name", label: "Department", sortable: true },
    { key: "branch_name", label: "Branch", width: "140px" },
    { key: "head_name", label: "Head", width: "140px" },
    { key: "is_active", label: "Status", width: "80px", render: (v: boolean) => <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: v ? "rgba(25,135,84,0.12)" : "rgba(134,142,150,0.12)", color: v ? "#198754" : "#868E96" }}>{v ? "Active" : "Inactive"}</span> },
  ];

  const columnMap: Record<string, Column<any>[]> = { companies: companyColumns, branches: branchColumns, departments: deptColumns };
  const dataMap: Record<string, any[]> = { companies, branches, departments };

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <Building2 size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> Organizations
          </h1>
          <button className="btn btn-primary" style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <Plus size={16} /> New {tab.slice(0, -1)}
          </button>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: "2px solid #E9ECEF" }}>
          {(["companies", "branches", "departments"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{ padding: "10px 16px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400, color: tab === t ? "#1B4332" : "#868E96", borderBottom: tab === t ? "2px solid #1B4332" : "2px solid transparent", marginBottom: -2 }}>
              {t.charAt(0).toUpperCase() + t.slice(1)}
            </button>
          ))}
        </div>

        <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
          {loading ? <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
            : <DataTable columns={columnMap[tab]} data={dataMap[tab]} pageSize={15} emptyMessage={`No ${tab}`} />}
        </div>
      </div>
    </WorkspaceShell>
  );
}
