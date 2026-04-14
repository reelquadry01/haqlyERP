"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet } from "@/lib/api";
import { Receipt, Plus, AlertTriangle, CheckCircle, Clock } from "lucide-react";

interface TaxConfig {
  id: string;
  tax_type: string;
  name: string;
  rate: number;
  effective_from: string;
  effective_to: string | null;
  is_active: boolean;
}

interface TaxComputation {
  id: string;
  tax_type: string;
  period: string;
  taxable_amount: number;
  computed_tax: number;
  effective_rate: number;
  risk_level: string;
}

interface TaxSchedule {
  id: string;
  tax_type: string;
  period: string;
  due_date: string;
  amount: number;
  status: string;
}

const TABS = ["configurations", "computations", "schedules", "risk"] as const;
type Tab = (typeof TABS)[number];

export default function TaxPage() {
  const token = getToken();
  const [activeTab, setActiveTab] = useState<Tab>("configurations");
  const [configs, setConfigs] = useState<TaxConfig[]>([]);
  const [computations, setComputations] = useState<TaxComputation[]>([]);
  const [schedules, setSchedules] = useState<TaxSchedule[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const [cfgRes, compRes, schRes] = await Promise.all([
          apiGet("/tax/configs", token),
          apiGet("/tax/computations", token),
          apiGet("/tax/schedules", token),
        ]);
        if (cfgRes.ok) setConfigs((await cfgRes.json()).data || []);
        if (compRes.ok) setComputations((await compRes.json()).data || []);
        if (schRes.ok) setSchedules((await schRes.json()).data || []);
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  const configColumns: Column<TaxConfig>[] = [
    { key: "tax_type", label: "Type", sortable: true, width: "80px" },
    { key: "name", label: "Name", sortable: true },
    { key: "rate", label: "Rate", align: "right", width: "80px", render: (v: number) => `${v}%` },
    { key: "effective_from", label: "From", width: "110px" },
    {
      key: "is_active",
      label: "Status",
      width: "80px",
      render: (v: boolean) => (
        <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: v ? "rgba(25,135,84,0.12)" : "rgba(134,142,150,0.12)", color: v ? "#198754" : "#868E96" }}>
          {v ? "Active" : "Inactive"}
        </span>
      ),
    },
  ];

  const computationColumns: Column<TaxComputation>[] = [
    { key: "tax_type", label: "Type", width: "80px" },
    { key: "period", label: "Period", sortable: true, width: "120px" },
    { key: "taxable_amount", label: "Taxable", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "computed_tax", label: "Tax", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "effective_rate", label: "Eff. Rate", align: "right", width: "90px", render: (v: number) => `${v}%` },
    {
      key: "risk_level",
      label: "Risk",
      width: "80px",
      render: (v: string) => {
        const map: Record<string, { bg: string; color: string }> = { high: { bg: "rgba(220,53,69,0.12)", color: "#DC3545" }, medium: { bg: "rgba(255,193,7,0.12)", color: "#B8860B" }, low: { bg: "rgba(25,135,84,0.12)", color: "#198754" } };
        const c = map[v] || map.low;
        return <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: c.bg, color: c.color }}>{v}</span>;
      },
    },
  ];

  const scheduleColumns: Column<TaxSchedule>[] = [
    { key: "tax_type", label: "Type", width: "80px" },
    { key: "period", label: "Period", width: "120px" },
    { key: "due_date", label: "Due Date", sortable: true, width: "110px" },
    { key: "amount", label: "Amount", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    {
      key: "status",
      label: "Status",
      width: "100px",
      render: (v: string) => {
        const map: Record<string, { bg: string; color: string; icon: typeof CheckCircle }> = { filed: { bg: "rgba(25,135,84,0.12)", color: "#198754", icon: CheckCircle }, overdue: { bg: "rgba(220,53,69,0.12)", color: "#DC3545", icon: AlertTriangle }, pending: { bg: "rgba(255,193,7,0.12)", color: "#B8860B", icon: Clock } };
        const c = map[v] || map.pending;
        const Icon = c.icon;
        return <span style={{ display: "flex", alignItems: "center", gap: 4, fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: c.bg, color: c.color }}><Icon size={12} />{v}</span>;
      },
    },
  ];

  const columnMap: Record<Tab, Column<any>[]> = {
    configurations: configColumns,
    computations: computationColumns,
    schedules: scheduleColumns,
    risk: computationColumns,
  };
  const dataMap: Record<Tab, any[]> = {
    configurations: configs,
    computations: computations,
    schedules: schedules,
    risk: computations.filter((c) => c.risk_level !== "low"),
  };

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <Receipt size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> Tax Management
          </h1>
          <button className="btn btn-primary" style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <Plus size={16} /> New Config
          </button>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: "2px solid #E9ECEF" }}>
          {TABS.map((t) => (
            <button key={t} onClick={() => setActiveTab(t)} style={{ padding: "10px 16px", fontSize: "0.85rem", fontWeight: activeTab === t ? 600 : 400, color: activeTab === t ? "#1B4332" : "#868E96", borderBottom: activeTab === t ? "2px solid #1B4332" : "2px solid transparent", marginBottom: -2, textTransform: "capitalize" }}>
              {t}
            </button>
          ))}
        </div>

        <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
          {loading ? <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
            : <DataTable columns={columnMap[activeTab]} data={dataMap[activeTab]} pageSize={15} emptyMessage={`No ${activeTab}`} />}
        </div>
      </div>
    </WorkspaceShell>
  );
}
