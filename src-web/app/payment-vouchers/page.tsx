"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet, apiPost, apiPatch } from "@/lib/api";
import { FileText, Plus, CheckCircle, Clock, Send } from "lucide-react";

interface PaymentVoucher {
  id: string;
  voucher_number: string;
  payee_name: string;
  amount: number;
  payment_date: string;
  narration: string;
  status: "draft" | "submitted" | "approved" | "paid" | "cancelled";
  approved_by: string | null;
  created_by: string;
  created_at: string;
}

export default function PaymentVouchersPage() {
  const token = getToken();
  const [vouchers, setVouchers] = useState<PaymentVoucher[]>([]);
  const [tab, setTab] = useState<"all" | "draft" | "pending" | "paid">("all");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const res = await apiGet("/payment-vouchers", token);
        if (res.ok) setVouchers((await res.json()).data || []);
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  const filteredVouchers = tab === "all"
    ? vouchers
    : tab === "draft"
    ? vouchers.filter((v) => v.status === "draft")
    : tab === "pending"
    ? vouchers.filter((v) => v.status === "submitted")
    : vouchers.filter((v) => v.status === "paid");

  const columns: Column<PaymentVoucher>[] = [
    { key: "voucher_number", label: "Voucher #", sortable: true, width: "140px" },
    { key: "payee_name", label: "Payee", sortable: true },
    { key: "amount", label: "Amount", align: "right", sortable: true, width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "payment_date", label: "Date", sortable: true, width: "110px" },
    { key: "narration", label: "Narration" },
    {
      key: "status",
      label: "Status",
      width: "100px",
      render: (v: string) => {
        const map: Record<string, { bg: string; color: string; icon: typeof CheckCircle }> = {
          draft: { bg: "rgba(134,142,150,0.12)", color: "#868E96", icon: FileText },
          submitted: { bg: "rgba(255,193,7,0.12)", color: "#B8860B", icon: Clock },
          approved: { bg: "rgba(13,202,240,0.12)", color: "#0DCAF0", icon: CheckCircle },
          paid: { bg: "rgba(25,135,84,0.12)", color: "#198754", icon: CheckCircle },
          cancelled: { bg: "rgba(220,53,69,0.12)", color: "#DC3545", icon: FileText },
        };
        const c = map[v] || map.draft;
        const Icon = c.icon;
        return <span style={{ display: "flex", alignItems: "center", gap: 4, fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: c.bg, color: c.color }}><Icon size={12} />{v}</span>;
      },
    },
  ];

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <FileText size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> Payment Vouchers
          </h1>
          <button className="btn btn-primary" style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <Plus size={16} /> New Voucher
          </button>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: "2px solid #E9ECEF" }}>
          {(["all", "draft", "pending", "paid"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{ padding: "10px 16px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400, color: tab === t ? "#1B4332" : "#868E96", borderBottom: tab === t ? "2px solid #1B4332" : "2px solid transparent", marginBottom: -2, textTransform: "capitalize" }}>
              {t}
            </button>
          ))}
        </div>

        <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
          {loading ? <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
            : <DataTable columns={columns} data={filteredVouchers} pageSize={15} emptyMessage="No payment vouchers" />}
        </div>
      </div>
    </WorkspaceShell>
  );
}
