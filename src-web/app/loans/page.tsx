"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet } from "@/lib/api";
import { Landmark, Plus } from "lucide-react";

interface Loan {
  id: string;
  loan_type: string;
  lender_name: string;
  principal_amount: number;
  interest_rate: number;
  tenure_months: number;
  outstanding_principal: number;
  start_date: string;
  maturity_date: string;
  status: string;
}

interface AmortizationEntry {
  period_number: number;
  payment_date: string;
  opening_balance: number;
  principal_payment: number;
  interest_payment: number;
  total_payment: number;
  closing_balance: number;
  is_paid: boolean;
}

export default function LoansPage() {
  const token = getToken();
  const [loans, setLoans] = useState<Loan[]>([]);
  const [amortization, setAmortization] = useState<AmortizationEntry[]>([]);
  const [selectedLoan, setSelectedLoan] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const res = await apiGet("/loans", token);
        if (res.ok) setLoans((await res.json()).data || []);
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  useEffect(() => {
    async function loadAmortization() {
      if (!selectedLoan) return;
      try {
        const res = await apiGet(`/loans/${selectedLoan}/amortization`, token);
        if (res.ok) setAmortization((await res.json()).data || []);
      } catch {
        // offline
      }
    }
    loadAmortization();
  }, [selectedLoan, token]);

  const loanColumns: Column<Loan>[] = [
    { key: "loan_type", label: "Type", sortable: true, width: "120px" },
    { key: "lender_name", label: "Lender", sortable: true },
    { key: "principal_amount", label: "Principal", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "interest_rate", label: "Rate", align: "right", width: "80px", render: (v: number) => `${v}%` },
    { key: "tenure_months", label: "Months", align: "right", width: "80px" },
    { key: "outstanding_principal", label: "Outstanding", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "maturity_date", label: "Maturity", width: "110px" },
    {
      key: "status",
      label: "Status",
      width: "90px",
      render: (v: string) => {
        const map: Record<string, { bg: string; color: string }> = { active: { bg: "rgba(25,135,84,0.12)", color: "#198754" }, completed: { bg: "rgba(134,142,150,0.12)", color: "#868E96" }, defaulted: { bg: "rgba(220,53,69,0.12)", color: "#DC3545" } };
        const c = map[v] || map.active;
        return <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: c.bg, color: c.color }}>{v}</span>;
      },
    },
  ];

  const amortColumns: Column<AmortizationEntry>[] = [
    { key: "period_number", label: "#", width: "50px" },
    { key: "payment_date", label: "Date", width: "110px" },
    { key: "opening_balance", label: "Opening", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "principal_payment", label: "Principal", align: "right", width: "120px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "interest_payment", label: "Interest", align: "right", width: "120px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "total_payment", label: "Total", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "closing_balance", label: "Closing", align: "right", width: "130px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
  ];

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <Landmark size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> Loans
          </h1>
          <button className="btn btn-primary" style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <Plus size={16} /> New Loan
          </button>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: selectedLoan ? "1fr 1fr" : "1fr", gap: 20 }}>
          <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
            {loading ? <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
              : <DataTable columns={loanColumns} data={loans} pageSize={10} emptyMessage="No loans" onRowClick={(row) => setSelectedLoan(row.id)} />}
          </div>

          {selectedLoan && (
            <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
              <h3 style={{ fontSize: "0.95rem", fontWeight: 600, marginBottom: 16, color: "#1A1A2E" }}>Amortization Schedule</h3>
              <DataTable columns={amortColumns} data={amortization} pageSize={12} emptyMessage="No schedule data" />
            </div>
          )}
        </div>
      </div>
    </WorkspaceShell>
  );
}
