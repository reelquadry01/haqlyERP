"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { KPICard } from "@/components/ui/kpi-card";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken, getCompanyContext } from "@/lib/session";
import { apiGet } from "@/lib/api";
import {
  TrendingUp,
  TrendingDown,
  DollarSign,
  ShoppingCart,
  FileText,
  CreditCard,
  BookOpen,
  Receipt,
  BarChart3,
  Users,
  FileSpreadsheet,
  Wallet,
} from "lucide-react";

interface KPIData {
  revenue: number;
  expenses: number;
  netIncome: number;
  cashBalance: number;
  revenueChange: number;
  expensesChange: number;
  netIncomeChange: number;
  cashBalanceChange: number;
}

interface Transaction {
  id: string;
  date: string;
  reference: string;
  description: string;
  account: string;
  debit: number;
  credit: number;
  status: string;
}

const quickActions = [
  { icon: ShoppingCart, label: "Create Invoice", path: "/sales" },
  { icon: CreditCard, label: "Record Payment", path: "/sales" },
  { icon: BookOpen, label: "Journal Entry", path: "/journal-entries" },
  { icon: Users, label: "Run Payroll", path: "/hr-payroll" },
  { icon: Receipt, label: "Tax Return", path: "/tax" },
  { icon: FileText, label: "Create PO", path: "/purchases" },
  { icon: BarChart3, label: "View Reports", path: "/reports" },
  { icon: FileSpreadsheet, label: "E-Invoice", path: "/einvoicing" },
];

export default function DashboardPage() {
  const [kpi, setKpi] = useState<KPIData | null>(null);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [period, setPeriod] = useState("this_month");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function loadDashboard() {
      const token = getToken();
      const company = getCompanyContext();
      if (!token) {
        window.location.replace("/login");
        return;
      }
      try {
        const companyId = company || "";
        const [metricsRes, txRes] = await Promise.all([
          apiGet(`/dashboard/metrics?companyId=${companyId}&period=${period}`, token),
          apiGet(`/dashboard/recent-transactions?companyId=${companyId}&limit=15`, token),
        ]);
        if (metricsRes.ok) {
          const data = await metricsRes.json();
          setKpi(data);
        }
        if (txRes.ok) {
          const data = await txRes.json();
          setTransactions(data.transactions || []);
        }
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    loadDashboard();
  }, [period]);

  const transactionColumns: Column<Transaction>[] = [
    { key: "date", label: "Date", sortable: true, width: "100px" },
    { key: "reference", label: "Reference", sortable: true, width: "120px" },
    { key: "description", label: "Description" },
    { key: "account", label: "Account", width: "160px" },
    {
      key: "debit",
      label: "Debit",
      align: "right",
      width: "120px",
      render: (v: number) => (v ? `₦${v.toLocaleString("en-NG")}` : ""),
    },
    {
      key: "credit",
      label: "Credit",
      align: "right",
      width: "120px",
      render: (v: number) => (v ? `₦${v.toLocaleString("en-NG")}` : ""),
    },
    {
      key: "status",
      label: "Status",
      width: "90px",
      render: (v: string) => {
        const colors: Record<string, { bg: string; color: string }> = {
          posted: { bg: "rgba(25,135,84,0.12)", color: "#198754" },
          pending: { bg: "rgba(255,193,7,0.12)", color: "#B8860B" },
          draft: { bg: "rgba(134,142,150,0.12)", color: "#868E96" },
        };
        const c = colors[v] || colors.draft;
        return (
          <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: c.bg, color: c.color }}>
            {v.charAt(0).toUpperCase() + v.slice(1)}
          </span>
        );
      },
    },
  ];

  if (loading) {
    return (
      <div className="flex-center full-viewport">
        <div className="splash-loader" />
      </div>
    );
  }

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>Dashboard</h1>
          <select
            value={period}
            onChange={(e) => setPeriod(e.target.value)}
            style={{ padding: "6px 12px", borderRadius: 8, border: "1px solid #DEE2E6", background: "#FFFFFF", fontSize: "0.85rem", color: "#495057" }}
          >
            <option value="this_month">This Month</option>
            <option value="this_quarter">This Quarter</option>
            <option value="this_year">This Year</option>
            <option value="last_month">Last Month</option>
          </select>
        </div>

        <div className="grid-kpi" style={{ marginBottom: 24 }}>
          <KPICard title="Revenue" value={kpi?.revenue || 0} change={kpi?.revenueChange || 0} trend={kpi?.revenueChange >= 0 ? "up" : "down"} color="success" icon={<DollarSign size={16} />} />
          <KPICard title="Expenses" value={kpi?.expenses || 0} change={kpi?.expensesChange || 0} trend={kpi?.expensesChange <= 0 ? "up" : "down"} color="warning" icon={<TrendingDown size={16} />} />
          <KPICard title="Net Income" value={kpi?.netIncome || 0} change={kpi?.netIncomeChange || 0} trend={kpi?.netIncomeChange >= 0 ? "up" : "down"} color="primary" icon={<TrendingUp size={16} />} />
          <KPICard title="Cash Balance" value={kpi?.cashBalance || 0} change={kpi?.cashBalanceChange || 0} trend={kpi?.cashBalanceChange >= 0 ? "up" : "down"} color="info" icon={<Wallet size={16} />} />
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "3fr 2fr", gap: 20, marginBottom: 24 }}>
          <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
            <h3 style={{ fontSize: "0.95rem", fontWeight: 600, marginBottom: 16, color: "#1A1A2E" }}>Recent Transactions</h3>
            <DataTable columns={transactionColumns} data={transactions} pageSize={8} emptyMessage="No recent transactions" />
          </div>

          <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
            <h3 style={{ fontSize: "0.95rem", fontWeight: 600, marginBottom: 16, color: "#1A1A2E" }}>Quick Actions</h3>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10 }}>
              {quickActions.map((action) => {
                const Icon = action.icon;
                return (
                  <button
                    key={action.label}
                    onClick={() => (window.location.href = action.path)}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 8,
                      padding: "10px 12px",
                      borderRadius: 8,
                      border: "1px solid #E9ECEF",
                      background: "#F8F9FA",
                      fontSize: "0.8rem",
                      color: "#495057",
                      transition: "all 150ms ease",
                    }}
                    onMouseEnter={(e) => { e.currentTarget.style.background = "#F1F3F5"; e.currentTarget.style.borderColor = "#DEE2E6"; }}
                    onMouseLeave={(e) => { e.currentTarget.style.background = "#F8F9FA"; e.currentTarget.style.borderColor = "#E9ECEF"; }}
                  >
                    <Icon size={16} />
                    {action.label}
                  </button>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </WorkspaceShell>
  );
}
