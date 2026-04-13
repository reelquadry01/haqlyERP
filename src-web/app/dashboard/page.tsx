"use client";

import { useEffect, useState } from "react";
import { apiGet } from "@/lib/api";
import { getToken, getCompanyContext } from "@/lib/session";
import { KPICard } from "@/components/ui/kpi-card";
import { BrandLockup } from "@/components/ui/brand-lockup";
import {
  DollarSign,
  TrendingDown,
  TrendingUp,
  Wallet,
} from "lucide-react";

interface DashboardMetrics {
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
  description: string;
  type: string;
  amount: number;
  status: string;
}

export default function DashboardPage() {
  const [metrics, setMetrics] = useState<DashboardMetrics | null>(null);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

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
          apiGet(`/dashboard/metrics?companyId=${companyId}`, token),
          apiGet(`/dashboard/recent-transactions?companyId=${companyId}&limit=10`, token),
        ]);

        if (!metricsRes.ok || !txRes.ok) {
          throw new Error("Failed to load dashboard data");
        }

        const metricsData = await metricsRes.json();
        const txData = await txRes.json();
        setMetrics(metricsData);
        setTransactions(txData.transactions || []);
      } catch (err: any) {
        setError(err.message || "Failed to load dashboard");
      } finally {
        setLoading(false);
      }
    }

    loadDashboard();
  }, []);

  if (loading) {
    return (
      <div className="flex-center full-viewport">
        <div className="splash-loader" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex-center full-viewport" style={{ flexDirection: "column", gap: 16 }}>
        <p className="text-danger">{error}</p>
        <button className="btn btn-primary" onClick={() => window.location.reload()}>
          Retry
        </button>
      </div>
    );
  }

  const chartPlaceholderStyle: React.CSSProperties = {
    height: 240,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    border: "1px dashed #2a3754",
    borderRadius: 8,
    color: "#64748b",
    fontSize: "0.85rem",
  };

  const thStyle: React.CSSProperties = {
    padding: "8px 12px",
    textAlign: "left",
    fontWeight: 600,
    fontSize: "0.8rem",
    color: "#94a3b8",
    textTransform: "uppercase",
    letterSpacing: "0.05em",
    borderBottom: "1px solid #2a3754",
  };

  const tdStyle: React.CSSProperties = {
    padding: "10px 12px",
    fontSize: "0.875rem",
    borderBottom: "1px solid #1e2a3e",
  };

  function statusColor(status: string) {
    if (status === "posted") return { bg: "rgba(16,185,129,0.15)", color: "#10b981" };
    if (status === "pending") return { bg: "rgba(245,158,11,0.15)", color: "#f59e0b" };
    return { bg: "rgba(100,116,139,0.15)", color: "#64748b" };
  }

  return (
    <div style={{ padding: 24, maxWidth: 1400, margin: "0 auto" }} className="fade-in">
      <div style={{ marginBottom: 24, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <h1 style={{ fontSize: "1.5rem", fontWeight: 700 }}>Dashboard</h1>
          <p className="text-muted">Financial overview and recent activity</p>
        </div>
        <BrandLockup />
      </div>

      <div className="grid-kpi" style={{ marginBottom: 24 }}>
        <KPICard
          title="Revenue"
          value={metrics?.revenue ?? 0}
          change={metrics?.revenueChange ?? 0}
          trend={metrics && metrics.revenueChange >= 0 ? "up" : "down"}
          color="success"
          icon={<DollarSign size={18} />}
        />
        <KPICard
          title="Expenses"
          value={metrics?.expenses ?? 0}
          change={metrics?.expensesChange ?? 0}
          trend={metrics && metrics.expensesChange >= 0 ? "up" : "down"}
          color="danger"
          icon={<TrendingDown size={18} />}
        />
        <KPICard
          title="Net Income"
          value={metrics?.netIncome ?? 0}
          change={metrics?.netIncomeChange ?? 0}
          trend={metrics && metrics.netIncomeChange >= 0 ? "up" : "down"}
          color="info"
          icon={<TrendingUp size={18} />}
        />
        <KPICard
          title="Cash Balance"
          value={metrics?.cashBalance ?? 0}
          change={metrics?.cashBalanceChange ?? 0}
          trend={metrics && metrics.cashBalanceChange >= 0 ? "up" : "down"}
          color="primary"
          icon={<Wallet size={18} />}
        />
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 24 }}>
        <div className="card">
          <h3 style={{ marginBottom: 16, fontSize: "1rem", fontWeight: 600 }}>Revenue vs Expenses</h3>
          <div style={chartPlaceholderStyle}>
            Chart placeholder — integrate Chart.js or Recharts
          </div>
        </div>
        <div className="card">
          <h3 style={{ marginBottom: 16, fontSize: "1rem", fontWeight: 600 }}>Cash Flow Trend</h3>
          <div style={chartPlaceholderStyle}>
            Chart placeholder — integrate Chart.js or Recharts
          </div>
        </div>
      </div>

      <div className="card">
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
          <h3 style={{ fontSize: "1rem", fontWeight: 600 }}>Recent Transactions</h3>
          <a href="/accounting" style={{ fontSize: "0.85rem" }}>View all →</a>
        </div>
        {transactions.length === 0 ? (
          <p className="text-muted" style={{ textAlign: "center", padding: 24 }}>
            No recent transactions
          </p>
        ) : (
          <table style={{ width: "100%", borderCollapse: "collapse" }}>
            <thead>
              <tr>
                <th style={thStyle}>Date</th>
                <th style={thStyle}>Description</th>
                <th style={thStyle}>Type</th>
                <th style={{ ...thStyle, textAlign: "right" }}>Amount</th>
                <th style={thStyle}>Status</th>
              </tr>
            </thead>
            <tbody>
              {transactions.map((tx) => {
                const sc = statusColor(tx.status);
                return (
                  <tr key={tx.id}>
                    <td style={tdStyle}>{tx.date}</td>
                    <td style={tdStyle}>{tx.description}</td>
                    <td style={tdStyle}>{tx.type}</td>
                    <td style={{ ...tdStyle, textAlign: "right", fontFamily: "var(--font-mono)" }}>
                      ₦{tx.amount.toLocaleString()}
                    </td>
                    <td style={tdStyle}>
                      <span
                        style={{
                          padding: "2px 8px",
                          borderRadius: 12,
                          fontSize: "0.75rem",
                          fontWeight: 600,
                          background: sc.bg,
                          color: sc.color,
                        }}
                      >
                        {tx.status}
                      </span>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
