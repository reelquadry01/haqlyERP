// Author: Quadri Atharu

"use client";

import { useEffect, useState, useCallback } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { getToken, getCompanyContext } from "@/lib/session";
import { apiGet } from "@/lib/api";
import {
  TrendingUp,
  TrendingDown,
  DollarSign,
  Wallet,
  BookOpen,
  FileText,
  CreditCard,
  Users,
  Receipt,
  BarChart3,
  ShoppingCart,
  AlertTriangle,
  RefreshCw,
  ChevronRight,
  Calendar,
  Clock,
} from "lucide-react";

interface KPIData {
  revenue_ytd: number;
  expenses_ytd: number;
  net_profit: number;
  cash_balance: number;
  revenue_change: number;
  expenses_change: number;
  net_profit_change: number;
  cash_balance_change: number;
}

interface TrialBalanceAccount {
  code: string;
  name: string;
  type: string;
  debit_balance: number;
  credit_balance: number;
}

interface JournalEntrySummary {
  id: string;
  reference: string;
  entry_date: string;
  narration: string;
  total_debit: number;
  total_credit: number;
  status: string;
  created_by: string;
}

interface PendingItem {
  id: string;
  type: string;
  description: string;
  date: string;
  urgency: "high" | "medium" | "low";
}

const PRIMARY = "#1B4332";
const ACCENT = "#D4AF37";
const BG = "#F8F9FA";
const SURFACE = "#FFFFFF";
const TEXT = "#1A1A2E";
const TEXT_SEC = "#495057";
const TEXT_TER = "#868E96";
const BORDER = "#DEE2E6";
const ERROR = "#DC2626";
const SUCCESS = "#16A34A";

function formatNaira(v: number) {
  return `₦${(v || 0).toLocaleString("en-NG", { minimumFractionDigits: 0, maximumFractionDigits: 0 })}`;
}

function Skeleton({ w = "100%", h = 20 }: { w?: string; h?: number }) {
  return (
    <div
      style={{
        width: w,
        height: h,
        borderRadius: 4,
        background: "linear-gradient(90deg, #E9ECEF 25%, #DEE2E6 50%, #E9ECEF 75%)",
        backgroundSize: "200% 100%",
        animation: "shimmer 1.5s infinite",
      }}
    />
  );
}

const STATUS_COLORS: Record<string, { bg: string; color: string }> = {
  draft: { bg: "rgba(134,142,150,0.12)", color: TEXT_TER },
  validated: { bg: "rgba(59,130,246,0.12)", color: "#2563EB" },
  approved: { bg: "rgba(22,163,74,0.12)", color: SUCCESS },
  posted: { bg: "rgba(212,175,55,0.15)", color: ACCENT },
  reversed: { bg: "rgba(220,38,38,0.12)", color: ERROR },
};

const quickActions = [
  { icon: BookOpen, label: "New Journal Entry", path: "/journal-entries", color: PRIMARY },
  { icon: ShoppingCart, label: "New Invoice", path: "/sales", color: "#2563EB" },
  { icon: Users, label: "Run Payroll", path: "/hr-payroll", color: "#7C3AED" },
  { icon: BarChart3, label: "View Reports", path: "/reports", color: ACCENT },
];

export default function DashboardPage() {
  const [kpi, setKpi] = useState<KPIData | null>(null);
  const [trialBalance, setTrialBalance] = useState<TrialBalanceAccount[]>([]);
  const [recentEntries, setRecentEntries] = useState<JournalEntrySummary[]>([]);
  const [pendingItems, setPendingItems] = useState<PendingItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [period, setPeriod] = useState("this_year");

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    const token = getToken();
    if (!token) {
      window.location.replace("/login");
      return;
    }
    try {
      const companyId = getCompanyContext() || "";
      const [reportsRes, tbRes, jeRes, pendingRes] = await Promise.all([
        apiGet(`/reports?companyId=${companyId}&period=${period}`, token),
        apiGet(`/reports/trial-balance?companyId=${companyId}`, token),
        apiGet(`/journal-entries?companyId=${companyId}&limit=10`, token),
        apiGet(`/reports/pending?companyId=${companyId}`, token),
      ]);
      if (reportsRes.ok) {
        const d = await reportsRes.json();
        setKpi(d.data || d);
      }
      if (tbRes.ok) {
        const d = await tbRes.json();
        const accounts = d.data || d.accounts || [];
        setTrialBalance(accounts.slice(0, 10));
      }
      if (jeRes.ok) {
        const d = await jeRes.json();
        setRecentEntries(d.data || d.entries || []);
      }
      if (pendingRes.ok) {
        const d = await pendingRes.json();
        setPendingItems(d.data || d.items || []);
      }
      if (!reportsRes.ok && !tbRes.ok && !jeRes.ok) {
        setError("Failed to load dashboard data. Please try again.");
      }
    } catch {
      setError("Network error. Check your connection and try again.");
    } finally {
      setLoading(false);
    }
  }, [period]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  function renderKPICard(
    title: string,
    value: number,
    change: number,
    icon: React.ReactNode,
    accentColor: string
  ) {
    const isPositive = change >= 0;
    return (
      <div
        style={{
          background: SURFACE,
          border: `1px solid ${BORDER}`,
          borderRadius: 8,
          padding: 20,
          boxShadow: "0 2px 8px rgba(0,0,0,0.08)",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: 12 }}>
          <span style={{ fontSize: "0.8rem", color: TEXT_SEC, fontWeight: 500 }}>{title}</span>
          <div
            style={{
              width: 36,
              height: 36,
              borderRadius: 8,
              background: `${accentColor}15`,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              color: accentColor,
            }}
          >
            {icon}
          </div>
        </div>
        <div style={{ fontSize: "1.5rem", fontWeight: 700, color: TEXT, fontFamily: '"JetBrains Mono", monospace', marginBottom: 6 }}>
          {formatNaira(value)}
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 4, fontSize: "0.78rem" }}>
          {isPositive ? (
            <TrendingUp size={14} style={{ color: SUCCESS }} />
          ) : (
            <TrendingDown size={14} style={{ color: ERROR }} />
          )}
          <span style={{ color: isPositive ? SUCCESS : ERROR, fontWeight: 600 }}>
            {Math.abs(change).toFixed(1)}%
          </span>
          <span style={{ color: TEXT_TER }}>vs last period</span>
        </div>
      </div>
    );
  }

  function renderTrialBalanceRow(acc: TrialBalanceAccount, i: number) {
    return (
      <div
        key={acc.code || i}
        style={{
          display: "flex",
          alignItems: "center",
          padding: "10px 0",
          borderBottom: i < trialBalance.length - 1 ? `1px solid #E9ECEF` : "none",
          fontSize: "0.85rem",
        }}
      >
        <span style={{ fontFamily: '"JetBrains Mono", monospace', color: PRIMARY, fontWeight: 500, width: 60 }}>
          {acc.code}
        </span>
        <span style={{ flex: 1, color: TEXT }}>{acc.name}</span>
        <span style={{ width: 120, textAlign: "right", color: TEXT_SEC, fontFamily: '"JetBrains Mono", monospace' }}>
          {acc.debit_balance ? formatNaira(acc.debit_balance) : "—"}
        </span>
        <span style={{ width: 120, textAlign: "right", color: TEXT_SEC, fontFamily: '"JetBrains Mono", monospace' }}>
          {acc.credit_balance ? formatNaira(acc.credit_balance) : "—"}
        </span>
      </div>
    );
  }

  const urgencyColors: Record<string, { bg: string; color: string }> = {
    high: { bg: "rgba(220,38,38,0.12)", color: ERROR },
    medium: { bg: "rgba(212,175,55,0.15)", color: ACCENT },
    low: { bg: "rgba(134,142,150,0.12)", color: TEXT_TER },
  };

  if (error && !loading) {
    return (
      <WorkspaceShell>
        <div style={{ padding: 24, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", minHeight: "60vh" }}>
          <AlertTriangle size={48} style={{ color: ERROR, marginBottom: 16 }} />
          <h2 style={{ fontSize: "1.1rem", fontWeight: 600, color: TEXT, marginBottom: 8 }}>Unable to Load Dashboard</h2>
          <p style={{ fontSize: "0.85rem", color: TEXT_SEC, marginBottom: 20 }}>{error}</p>
          <button
            onClick={loadData}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 8,
              padding: "10px 20px",
              borderRadius: 6,
              background: PRIMARY,
              color: "#FFFFFF",
              fontSize: "0.85rem",
              fontWeight: 500,
              border: "none",
              cursor: "pointer",
            }}
          >
            <RefreshCw size={16} />
            Retry
          </button>
        </div>
      </WorkspaceShell>
    );
  }

  return (
    <WorkspaceShell>
      <div style={{ padding: 24, background: BG, minHeight: "100%" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 24, flexWrap: "wrap", gap: 12 }}>
          <h1 style={{ fontSize: "1.5rem", fontWeight: 700, color: TEXT, fontFamily: '"DM Serif Display", serif' }}>
            Dashboard
          </h1>
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <select
              value={period}
              onChange={(e) => setPeriod(e.target.value)}
              style={{
                padding: "8px 12px",
                borderRadius: 6,
                border: `1px solid ${BORDER}`,
                background: SURFACE,
                fontSize: "0.85rem",
                color: TEXT_SEC,
              }}
            >
              <option value="this_month">This Month</option>
              <option value="this_quarter">This Quarter</option>
              <option value="this_year">This Year</option>
              <option value="last_year">Last Year</option>
            </select>
            <button
              onClick={loadData}
              style={{ padding: 8, borderRadius: 6, background: SURFACE, border: `1px solid ${BORDER}`, cursor: "pointer" }}
            >
              <RefreshCw size={16} style={{ color: TEXT_SEC }} />
            </button>
          </div>
        </div>

        {/* KPI Row */}
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))", gap: 16, marginBottom: 24 }}>
          {loading ? (
            Array.from({ length: 4 }).map((_, i) => (
              <div key={i} style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, padding: 20 }}>
                <Skeleton w="60%" h={14} />
                <div style={{ height: 12 }} />
                <Skeleton w="80%" h={28} />
                <div style={{ height: 8 }} />
                <Skeleton w="50%" h={14} />
              </div>
            ))
          ) : (
            <>
              {renderKPICard("Revenue YTD", kpi?.revenue_ytd || 0, kpi?.revenue_change || 0, <DollarSign size={18} />, SUCCESS)}
              {renderKPICard("Expenses YTD", kpi?.expenses_ytd || 0, kpi?.expenses_change || 0, <TrendingDown size={18} />, ERROR)}
              {renderKPICard("Net Profit", kpi?.net_profit || 0, kpi?.net_profit_change || 0, <TrendingUp size={18} />, PRIMARY)}
              {renderKPICard("Cash Balance", kpi?.cash_balance || 0, kpi?.cash_balance_change || 0, <Wallet size={18} />, ACCENT)}
            </>
          )}
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 20, marginBottom: 24 }}>
          {/* Trial Balance Summary */}
          <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
              <h3 style={{ fontSize: "0.95rem", fontWeight: 600, color: TEXT, fontFamily: '"DM Serif Display", serif' }}>
                Trial Balance Summary
              </h3>
              <a href="/reports" style={{ fontSize: "0.78rem", color: PRIMARY, textDecoration: "none", display: "flex", alignItems: "center", gap: 4 }}>
                Full Report <ChevronRight size={14} />
              </a>
            </div>
            <div style={{ display: "flex", padding: "8px 0", borderBottom: `1px solid ${BORDER}`, fontSize: "0.75rem", fontWeight: 600, color: TEXT_TER, textTransform: "uppercase", letterSpacing: 0.05 }}>
              <span style={{ width: 60 }}>Code</span>
              <span style={{ flex: 1 }}>Account</span>
              <span style={{ width: 120, textAlign: "right" }}>Debit</span>
              <span style={{ width: 120, textAlign: "right" }}>Credit</span>
            </div>
            {loading ? (
              Array.from({ length: 5 }).map((_, i) => (
                <div key={i} style={{ padding: "10px 0" }}>
                  <Skeleton h={14} />
                </div>
              ))
            ) : trialBalance.length === 0 ? (
              <div style={{ textAlign: "center", padding: 32, color: TEXT_TER }}>
                <BarChart3 size={32} style={{ marginBottom: 8, opacity: 0.4 }} />
                <p>No trial balance data</p>
              </div>
            ) : (
              trialBalance.map((acc, i) => renderTrialBalanceRow(acc, i))
            )}
          </div>

          {/* Quick Actions */}
          <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
            <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
              <h3 style={{ fontSize: "0.95rem", fontWeight: 600, color: TEXT, fontFamily: '"DM Serif Display", serif', marginBottom: 16 }}>
                Quick Actions
              </h3>
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
                        gap: 10,
                        padding: "12px 14px",
                        borderRadius: 6,
                        border: `1px solid ${BORDER}`,
                        background: BG,
                        fontSize: "0.82rem",
                        color: TEXT,
                        fontWeight: 500,
                        cursor: "pointer",
                        transition: "all 150ms ease",
                      }}
                      onMouseEnter={(e) => {
                        e.currentTarget.style.borderColor = action.color;
                        e.currentTarget.style.background = `${action.color}08`;
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.borderColor = BORDER;
                        e.currentTarget.style.background = BG;
                      }}
                    >
                      <div
                        style={{
                          width: 32,
                          height: 32,
                          borderRadius: 6,
                          background: `${action.color}15`,
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "center",
                        }}
                      >
                        <Icon size={16} style={{ color: action.color }} />
                      </div>
                      {action.label}
                    </button>
                  );
                })}
              </div>
            </div>

            {/* Pending Items */}
            <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)", flex: 1 }}>
              <h3 style={{ fontSize: "0.95rem", fontWeight: 600, color: TEXT, fontFamily: '"DM Serif Display", serif', marginBottom: 16 }}>
                Pending Items
              </h3>
              {loading ? (
                Array.from({ length: 3 }).map((_, i) => (
                  <div key={i} style={{ padding: "8px 0" }}>
                    <Skeleton h={14} />
                  </div>
                ))
              ) : pendingItems.length === 0 ? (
                <div style={{ textAlign: "center", padding: 20, color: TEXT_TER }}>
                  <Clock size={24} style={{ marginBottom: 8, opacity: 0.4 }} />
                  <p style={{ fontSize: "0.82rem" }}>No pending items</p>
                </div>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                  {pendingItems.slice(0, 6).map((item) => {
                    const uc = urgencyColors[item.urgency] || urgencyColors.low;
                    return (
                      <div
                        key={item.id}
                        style={{
                          display: "flex",
                          alignItems: "center",
                          gap: 10,
                          padding: "8px 0",
                          borderBottom: `1px solid #E9ECEF`,
                          fontSize: "0.82rem",
                        }}
                      >
                        <div
                          style={{
                            width: 8,
                            height: 8,
                            borderRadius: "50%",
                            background: uc.color,
                            flexShrink: 0,
                          }}
                        />
                        <span style={{ flex: 1, color: TEXT }}>{item.description}</span>
                        <span style={{ fontSize: "0.72rem", padding: "2px 8px", borderRadius: 4, background: uc.bg, color: uc.color }}>
                          {item.urgency}
                        </span>
                        <span style={{ fontSize: "0.75rem", color: TEXT_TER, display: "flex", alignItems: "center", gap: 4 }}>
                          <Calendar size={12} />
                          {item.date}
                        </span>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Recent Journal Entries */}
        <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
            <h3 style={{ fontSize: "0.95rem", fontWeight: 600, color: TEXT, fontFamily: '"DM Serif Display", serif' }}>
              Recent Journal Entries
            </h3>
            <a href="/journal-entries" style={{ fontSize: "0.78rem", color: PRIMARY, textDecoration: "none", display: "flex", alignItems: "center", gap: 4 }}>
              View All <ChevronRight size={14} />
            </a>
          </div>
          {loading ? (
            Array.from({ length: 5 }).map((_, i) => (
              <div key={i} style={{ padding: "8px 0" }}>
                <Skeleton h={14} />
              </div>
            ))
          ) : recentEntries.length === 0 ? (
            <div style={{ textAlign: "center", padding: 32, color: TEXT_TER }}>
              <FileText size={32} style={{ marginBottom: 8, opacity: 0.4 }} />
              <p>No journal entries found</p>
            </div>
          ) : (
            <div style={{ overflowX: "auto" }}>
              <table style={{ width: "100%", borderCollapse: "collapse" }}>
                <thead>
                  <tr>
                    {["Reference", "Date", "Narration", "Debit", "Credit", "Status"].map((h) => (
                      <th
                        key={h}
                        style={{
                          padding: "10px 12px",
                          textAlign: h === "Debit" || h === "Credit" ? "right" : "left",
                          fontSize: "0.75rem",
                          fontWeight: 600,
                          color: TEXT_TER,
                          textTransform: "uppercase",
                          letterSpacing: 0.05,
                          borderBottom: `1px solid ${BORDER}`,
                        }}
                      >
                        {h}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {recentEntries.map((entry) => {
                    const sc = STATUS_COLORS[entry.status] || STATUS_COLORS.draft;
                    return (
                      <tr
                        key={entry.id}
                        style={{ cursor: "pointer" }}
                        onClick={() => (window.location.href = `/journal-entries?id=${entry.id}`)}
                        onMouseEnter={(e) => (e.currentTarget.style.background = "#F8F9FA")}
                        onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
                      >
                        <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: PRIMARY, fontWeight: 500, fontFamily: '"JetBrains Mono", monospace', borderBottom: "1px solid #E9ECEF" }}>
                          {entry.reference}
                        </td>
                        <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: TEXT_SEC, borderBottom: "1px solid #E9ECEF" }}>
                          {entry.entry_date}
                        </td>
                        <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: TEXT, maxWidth: 280, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", borderBottom: "1px solid #E9ECEF" }}>
                          {entry.narration}
                        </td>
                        <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: TEXT_SEC, textAlign: "right", fontFamily: '"JetBrains Mono", monospace', borderBottom: "1px solid #E9ECEF" }}>
                          {formatNaira(entry.total_debit)}
                        </td>
                        <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: TEXT_SEC, textAlign: "right", fontFamily: '"JetBrains Mono", monospace', borderBottom: "1px solid #E9ECEF" }}>
                          {formatNaira(entry.total_credit)}
                        </td>
                        <td style={{ padding: "10px 12px", borderBottom: "1px solid #E9ECEF" }}>
                          <span style={{ fontSize: "0.75rem", padding: "3px 10px", borderRadius: 4, background: sc.bg, color: sc.color, fontWeight: 500 }}>
                            {entry.status.charAt(0).toUpperCase() + entry.status.slice(1)}
                          </span>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
        </div>

        <style>{`
          @keyframes shimmer {
            0% { background-position: 200% 0; }
            100% { background-position: -200% 0; }
          }
          @media (max-width: 768px) {
            div[style*="grid-template-columns: 1fr 1fr"] {
              grid-template-columns: 1fr !important;
            }
          }
        `}</style>
      </div>
    </WorkspaceShell>
  );
}
