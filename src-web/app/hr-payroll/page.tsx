"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { KPICard } from "@/components/ui/kpi-card";
import { getToken } from "@/lib/session";
import { apiGet, apiPost } from "@/lib/api";
import { Users, Plus, FileText, Play } from "lucide-react";

interface Employee {
  id: string;
  employee_number: string;
  full_name: string;
  designation: string;
  grade_level: string;
  salary_amount: number;
  employment_type: string;
  department: string;
  is_active: boolean;
}

interface PayrollRun {
  id: string;
  run_name: string;
  period_month: number;
  period_year: number;
  total_gross: number;
  total_net: number;
  status: string;
  employee_count: number;
}

export default function HrPayrollPage() {
  const token = getToken();
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [payrollRuns, setPayrollRuns] = useState<PayrollRun[]>([]);
  const [tab, setTab] = useState<"employees" | "runs" | "payslips">("employees");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const [empRes, runRes] = await Promise.all([
          apiGet("/payroll/employees", token),
          apiGet("/payroll/runs", token),
        ]);
        if (empRes.ok) setEmployees((await empRes.json()).data || []);
        if (runRes.ok) setPayrollRuns((await runRes.json()).data || []);
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  const employeeColumns: Column<Employee>[] = [
    { key: "employee_number", label: "Emp #", sortable: true, width: "100px" },
    { key: "full_name", label: "Name", sortable: true },
    { key: "designation", label: "Designation", width: "140px" },
    { key: "grade_level", label: "Grade", width: "70px" },
    { key: "salary_amount", label: "Salary", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "employment_type", label: "Type", width: "90px" },
    { key: "department", label: "Dept", width: "120px" },
    {
      key: "is_active",
      label: "Status",
      width: "80px",
      render: (v: boolean) => (
        <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: v ? "rgba(25,135,84,0.12)" : "rgba(220,53,69,0.12)", color: v ? "#198754" : "#DC3545" }}>
          {v ? "Active" : "Inactive"}
        </span>
      ),
    },
  ];

  const runColumns: Column<PayrollRun>[] = [
    { key: "run_name", label: "Run Name", sortable: true },
    { key: "period_month", label: "Month", width: "70px" },
    { key: "period_year", label: "Year", width: "70px" },
    { key: "employee_count", label: "Employees", align: "right", width: "90px" },
    { key: "total_gross", label: "Gross", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "total_net", label: "Net Pay", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    {
      key: "status",
      label: "Status",
      width: "100px",
      render: (v: string) => {
        const map: Record<string, { bg: string; color: string }> = { draft: { bg: "rgba(134,142,150,0.12)", color: "#868E96" }, processing: { bg: "rgba(13,202,240,0.12)", color: "#0DCAF0" }, completed: { bg: "rgba(25,135,84,0.12)", color: "#198754" }, posted: { bg: "rgba(27,67,50,0.12)", color: "#1B4332" } };
        const c = map[v] || map.draft;
        return <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: c.bg, color: c.color }}>{v}</span>;
      },
    },
  ];

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <Users size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> HR & Payroll
          </h1>
          <div style={{ display: "flex", gap: 8 }}>
            <button className="btn btn-primary" style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <Plus size={16} /> Add Employee
            </button>
            <button className="btn btn-secondary" style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <Play size={16} /> Run Payroll
            </button>
          </div>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: "2px solid #E9ECEF" }}>
          {(["employees", "runs", "payslips"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{ padding: "10px 16px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400, color: tab === t ? "#1B4332" : "#868E96", borderBottom: tab === t ? "2px solid #1B4332" : "2px solid transparent", marginBottom: -2 }}>
              {t.charAt(0).toUpperCase() + t.slice(1)}
            </button>
          ))}
        </div>

        <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
          {loading ? <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
            : tab === "employees" ? <DataTable columns={employeeColumns} data={employees} pageSize={15} emptyMessage="No employees" />
            : tab === "runs" ? <DataTable columns={runColumns} data={payrollRuns} pageSize={10} emptyMessage="No payroll runs" />
            : <div style={{ textAlign: "center", padding: 40, color: "#868E96" }}>Select a payroll run to view payslips</div>}
        </div>
      </div>
    </WorkspaceShell>
  );
}
