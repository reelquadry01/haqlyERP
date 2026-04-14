"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet } from "@/lib/api";
import { Building2, Plus } from "lucide-react";

interface FixedAsset {
  id: string;
  asset_code: string;
  name: string;
  category: string;
  acquisition_date: string;
  acquisition_cost: number;
  accumulated_depreciation: number;
  net_book_value: number;
  depreciation_method: string;
  useful_life_months: number;
  status: string;
}

export default function FixedAssetsPage() {
  const token = getToken();
  const [assets, setAssets] = useState<FixedAsset[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const res = await apiGet("/fixed-assets", token);
        if (res.ok) setAssets((await res.json()).data || []);
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  const columns: Column<FixedAsset>[] = [
    { key: "asset_code", label: "Code", sortable: true, width: "100px" },
    { key: "name", label: "Asset Name", sortable: true },
    { key: "category", label: "Category", width: "120px" },
    { key: "acquisition_date", label: "Acq. Date", width: "110px" },
    { key: "acquisition_cost", label: "Cost", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "accumulated_depreciation", label: "Acc. Dep.", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "net_book_value", label: "NBV", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "depreciation_method", label: "Method", width: "100px" },
    {
      key: "status",
      label: "Status",
      width: "90px",
      render: (v: string) => {
        const map: Record<string, { bg: string; color: string }> = {
          active: { bg: "rgba(25,135,84,0.12)", color: "#198754" },
          disposed: { bg: "rgba(220,53,69,0.12)", color: "#DC3545" },
          fully_depreciated: { bg: "rgba(134,142,150,0.12)", color: "#868E96" },
        };
        const c = map[v] || map.active;
        return <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: c.bg, color: c.color }}>{v.replace("_", " ")}</span>;
      },
    },
  ];

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <Building2 size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> Fixed Assets
          </h1>
          <button className="btn btn-primary" style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <Plus size={16} /> New Asset
          </button>
        </div>
        <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
          {loading ? <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
            : <DataTable columns={columns} data={assets} pageSize={15} emptyMessage="No fixed assets" />}
        </div>
      </div>
    </WorkspaceShell>
  );
}
