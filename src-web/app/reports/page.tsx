// Author: Quadri Atharu

"use client";

import React, { useEffect, useState, useCallback, Fragment } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { getToken } from "@/lib/session";
import { apiGet } from "@/lib/api";
import {
  BarChart3,
  Download,
  Printer,
  FileText,
  ChevronDown,
  ChevronRight,
  CheckCircle2,
  AlertCircle,
  Loader2,
} from "lucide-react";

const TOKENS = {
  primary: "#1B4332",
  primaryHover: "#2D6A4F",
  primaryLight: "rgba(27,67,50,0.08)",
  accent: "#D4AF37",
  accentLight: "rgba(212,175,55,0.12)",
  bg: "#F8F9FA",
  surface: "#FFFFFF",
  surfaceHover: "#F1F3F5",
  border: "#DEE2E6",
  borderSubtle: "#E9ECEF",
  text: "#1A1A2E",
  textSecondary: "#495057",
  textTertiary: "#868E96",
  error: "#DC2626",
  errorLight: "rgba(220,38,38,0.08)",
  success: "#16A34A",
  successLight: "rgba(22,163,74,0.08)",
  radiusMd: 8,
  radiusSm: 6,
  shadowSm: "0 2px 8px rgba(0,0,0,0.08)",
  shadowMd: "0 4px 16px rgba(0,0,0,0.10)",
  fontUi: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  fontHeading: '"DM Serif Display", Georgia, serif',
};

type ReportTab = "trial_balance" | "income_statement" | "balance_sheet" | "cash_flow" | "retained_earnings";

interface Company {
  id: string;
  name: string;
  rc_number: string;
}

interface TrialBalanceRow {
  account_code: string;
  account_name: string;
  debit: number;
  credit: number;
}

interface IncomeStatementSection {
  label: string;
  items: { name: string; amount: number }[];
  subtotal: number;
  isSubtotalBold?: boolean;
  isTotal?: boolean;
  indent?: boolean;
}

interface BalanceSheetSection {
  label: string;
  items: { name: string; amount: number }[];
  subtotal: number;
}

interface CashFlowSection {
  label: string;
  items: { name: string; amount: number }[];
  subtotal: number;
}

interface RetainedEarningsRow {
  description: string;
  amount: number;
}

const REPORT_TABS: { id: ReportTab; label: string }[] = [
  { id: "trial_balance", label: "Trial Balance" },
  { id: "income_statement", label: "Income Statement" },
  { id: "balance_sheet", label: "Balance Sheet" },
  { id: "cash_flow", label: "Cash Flow" },
  { id: "retained_earnings", label: "Retained Earnings" },
];

const NIGERIAN_INDUSTRIES = [
  "Oil & Gas", "Banking & Finance", "Telecommunications", "Manufacturing",
  "Agriculture", "Real Estate", "Construction", "Healthcare",
  "Education", "Transportation", "Retail & Wholesale", "Mining",
  "Insurance", "Legal Services", "Technology",
];

function formatCurrency(value: number): string {
  return new Intl.NumberFormat("en-NG", {
    style: "currency",
    currency: "NGN",
    minimumFractionDigits: 2,
  }).format(value);
}

function SectionRow({ label, amount, bold, indent, color }: { label: string; amount: number; bold?: boolean; indent?: boolean; color?: string }) {
  return (
    <tr>
      <td style={{ padding: "8px 12px", fontWeight: bold ? 700 : 400, paddingLeft: indent ? 32 : 12, fontSize: "0.85rem", color: color || TOKENS.text, borderBottom: bold ? "2px solid " + TOKENS.border : "1px solid " + TOKENS.borderSubtle }}>
        {label}
      </td>
      <td style={{ padding: "8px 12px", textAlign: "right", fontWeight: bold ? 700 : 400, fontSize: "0.85rem", color: color || TOKENS.text, borderBottom: bold ? "2px solid " + TOKENS.border : "1px solid " + TOKENS.borderSubtle, fontFamily: TOKENS.fontUi }}>
        {formatCurrency(amount)}
      </td>
    </tr>
  );
}

function StatusIndicator({ balanced }: { balanced: boolean }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6, fontSize: "0.8rem", color: balanced ? TOKENS.success : TOKENS.error }}>
      {balanced ? <CheckCircle2 size={16} /> : <AlertCircle size={16} />}
      {balanced ? "Balanced" : "Out of Balance"}
    </div>
  );
}

export default function ReportsPage() {
  const token = getToken();
  const [activeTab, setActiveTab] = useState<ReportTab>("trial_balance");
  const [companies, setCompanies] = useState<Company[]>([]);
  const [selectedCompany, setSelectedCompany] = useState<string>("");
  const [periodFrom, setPeriodFrom] = useState(new Date().toISOString().split("T")[0].slice(0, 7));
  const [periodTo, setPeriodTo] = useState(new Date().toISOString().split("T")[0].slice(0, 7));
  const [comparisonFrom, setComparisonFrom] = useState("");
  const [comparisonTo, setComparisonTo] = useState("");
  const [showComparison, setShowComparison] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [trialBalance, setTrialBalance] = useState<TrialBalanceRow[]>([]);
  const [incomeStatement, setIncomeStatement] = useState<IncomeStatementSection[]>([]);
  const [balanceSheet, setBalanceSheet] = useState<BalanceSheetSection[]>([]);
  const [cashFlow, setCashFlow] = useState<CashFlowSection[]>([]);
  const [retainedEarnings, setRetainedEarnings] = useState<RetainedEarningsRow[]>([]);

  useEffect(() => {
    async function loadCompanies() {
      try {
        const res = await apiGet("/org/companies", token);
        if (res.ok) {
          const data = (await res.json()).data || [];
          setCompanies(data);
          if (data.length > 0) setSelectedCompany(data[0].id);
        }
      } catch {}
    }
    loadCompanies();
  }, [token]);

  const fetchReport = useCallback(async () => {
    if (!selectedCompany) return;
    setLoading(true);
    setError(null);
    try {
      let path = `/reports/${activeTab}?company=${selectedCompany}&from=${periodFrom}&to=${periodTo}`;
      if (showComparison && comparisonFrom && comparisonTo) {
        path += `&comp_from=${comparisonFrom}&comp_to=${comparisonTo}`;
      }
      const res = await apiGet(path, token);
      if (!res.ok) {
        setError("Failed to load report data");
        return;
      }
      const data = await res.json();
      switch (activeTab) {
        case "trial_balance":
          setTrialBalance(data.data || []);
          break;
        case "income_statement":
          setIncomeStatement(data.data || []);
          break;
        case "balance_sheet":
          setBalanceSheet(data.data || []);
          break;
        case "cash_flow":
          setCashFlow(data.data || []);
          break;
        case "retained_earnings":
          setRetainedEarnings(data.data || []);
          break;
      }
    } catch {
      setError("Network error. Please check your connection.");
    } finally {
      setLoading(false);
    }
  }, [activeTab, selectedCompany, periodFrom, periodTo, comparisonFrom, comparisonTo, showComparison, token]);

  useEffect(() => {
    fetchReport();
  }, [fetchReport]);

  async function handleExport(format: "pdf" | "xlsx" | "csv") {
    if (!selectedCompany) return;
    try {
      const res = await apiGet(`/reports/${activeTab}?company=${selectedCompany}&from=${periodFrom}&to=${periodTo}&format=${format}`, token);
      if (res.ok) {
        const blob = await res.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `${activeTab}_${periodFrom}_${periodTo}.${format === "xlsx" ? "xlsx" : format === "csv" ? "csv" : "pdf"}`;
        a.click();
        URL.revokeObjectURL(url);
      }
    } catch {}
  }

  function handlePrint() {
    window.print();
  }

  const totalDebit = trialBalance.reduce((s, r) => s + r.debit, 0);
  const totalCredit = trialBalance.reduce((s, r) => s + r.credit, 0);

  const inputStyle: React.CSSProperties = {
    background: TOKENS.surface,
    border: `1px solid ${TOKENS.border}`,
    borderRadius: TOKENS.radiusSm,
    padding: "6px 10px",
    fontSize: "0.85rem",
    color: TOKENS.text,
    fontFamily: TOKENS.fontUi,
  };

  const selectStyle: React.CSSProperties = {
    ...inputStyle,
    minWidth: 180,
  };

  const tabButtonStyle = (isActive: boolean): React.CSSProperties => ({
    padding: "10px 16px",
    fontSize: "0.85rem",
    fontWeight: isActive ? 600 : 400,
    color: isActive ? TOKENS.primary : TOKENS.textTertiary,
    borderBottom: isActive ? `2px solid ${TOKENS.primary}` : "2px solid transparent",
    marginBottom: -2,
    transition: "all 150ms ease",
    cursor: "pointer",
  });

  const exportButtonStyle: React.CSSProperties = {
    display: "inline-flex",
    alignItems: "center",
    gap: 4,
    padding: "6px 12px",
    borderRadius: TOKENS.radiusSm,
    fontSize: "0.8rem",
    fontWeight: 500,
    background: TOKENS.surface,
    border: `1px solid ${TOKENS.border}`,
    color: TOKENS.textSecondary,
    cursor: "pointer",
    transition: "all 150ms ease",
  };

  function renderTrialBalance() {
    if (trialBalance.length === 0 && !loading) {
      return (
        <div style={{ textAlign: "center", padding: 40, color: TOKENS.textTertiary }}>
          <BarChart3 size={32} style={{ marginBottom: 8 }} />
          <p>No trial balance data for the selected period</p>
        </div>
      );
    }
    return (
      <div style={{ overflowX: "auto" }}>
        <table style={{ width: "100%", borderCollapse: "collapse" }}>
          <thead>
            <tr>
              <th style={{ padding: "10px 12px", textAlign: "left", fontWeight: 600, fontSize: "0.8rem", color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: `2px solid ${TOKENS.border}`, width: 120 }}>Account Code</th>
              <th style={{ padding: "10px 12px", textAlign: "left", fontWeight: 600, fontSize: "0.8rem", color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: `2px solid ${TOKENS.border}` }}>Account Name</th>
              <th style={{ padding: "10px 12px", textAlign: "right", fontWeight: 600, fontSize: "0.8rem", color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: `2px solid ${TOKENS.border}`, width: 160 }}>Debit (₦)</th>
              <th style={{ padding: "10px 12px", textAlign: "right", fontWeight: 600, fontSize: "0.8rem", color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: `2px solid ${TOKENS.border}`, width: 160 }}>Credit (₦)</th>
            </tr>
          </thead>
          <tbody>
            {trialBalance.map((row, i) => (
              <tr key={i} style={{ background: i % 2 === 0 ? TOKENS.surface : TOKENS.bg }}>
                <td style={{ padding: "8px 12px", fontSize: "0.85rem", color: TOKENS.textSecondary, borderBottom: `1px solid ${TOKENS.borderSubtle}`, fontFamily: TOKENS.fontUi }}>{row.account_code}</td>
                <td style={{ padding: "8px 12px", fontSize: "0.85rem", color: TOKENS.text, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>{row.account_name}</td>
                <td style={{ padding: "8px 12px", textAlign: "right", fontSize: "0.85rem", color: TOKENS.text, borderBottom: `1px solid ${TOKENS.borderSubtle}`, fontFamily: TOKENS.fontUi }}>{row.debit ? formatCurrency(row.debit) : ""}</td>
                <td style={{ padding: "8px 12px", textAlign: "right", fontSize: "0.85rem", color: TOKENS.text, borderBottom: `1px solid ${TOKENS.borderSubtle}`, fontFamily: TOKENS.fontUi }}>{row.credit ? formatCurrency(row.credit) : ""}</td>
              </tr>
            ))}
            <tr style={{ background: TOKENS.primaryLight }}>
              <td colSpan={2} style={{ padding: "10px 12px", fontWeight: 700, fontSize: "0.85rem", color: TOKENS.primary, borderTop: `2px solid ${TOKENS.primary}` }}>TOTAL</td>
              <td style={{ padding: "10px 12px", textAlign: "right", fontWeight: 700, fontSize: "0.85rem", color: TOKENS.primary, borderTop: `2px solid ${TOKENS.primary}`, fontFamily: TOKENS.fontUi }}>{formatCurrency(totalDebit)}</td>
              <td style={{ padding: "10px 12px", textAlign: "right", fontWeight: 700, fontSize: "0.85rem", color: TOKENS.primary, borderTop: `2px solid ${TOKENS.primary}`, fontFamily: TOKENS.fontUi }}>{formatCurrency(totalCredit)}</td>
            </tr>
          </tbody>
        </table>
        <div style={{ display: "flex", alignItems: "center", gap: 12, marginTop: 12, padding: "8px 12px", borderRadius: TOKENS.radiusSm, background: totalDebit === totalCredit ? TOKENS.successLight : TOKENS.errorLight }}>
          <StatusIndicator balanced={totalDebit === totalCredit} />
          <span style={{ fontSize: "0.8rem", color: TOKENS.textTertiary }}>
            Difference: {formatCurrency(Math.abs(totalDebit - totalCredit))}
          </span>
        </div>
      </div>
    );
  }

  function renderIncomeStatement() {
    if (incomeStatement.length === 0 && !loading) {
      return (
        <div style={{ textAlign: "center", padding: 40, color: TOKENS.textTertiary }}>
          <FileText size={32} style={{ marginBottom: 8 }} />
          <p>No income statement data for the selected period</p>
        </div>
      );
    }
    return (
      <div style={{ overflowX: "auto" }}>
        <table style={{ width: "100%", borderCollapse: "collapse" }}>
          <thead>
            <tr>
              <th style={{ padding: "10px 12px", textAlign: "left", fontWeight: 600, fontSize: "0.8rem", color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: `2px solid ${TOKENS.border}` }}>Description</th>
              <th style={{ padding: "10px 12px", textAlign: "right", fontWeight: 600, fontSize: "0.8rem", color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: `2px solid ${TOKENS.border}`, width: 180 }}>Amount (₦)</th>
            </tr>
          </thead>
          <tbody>
            {incomeStatement.map((section, si) => (
              <Fragment key={si}>
                <tr style={{ background: TOKENS.primaryLight }}>
                  <td colSpan={2} style={{ padding: "10px 12px", fontWeight: 700, fontSize: "0.9rem", color: TOKENS.primary, borderBottom: `1px solid ${TOKENS.border}`, fontFamily: TOKENS.fontHeading }}>{section.label}</td>
                </tr>
                {section.items.map((item, ii) => (
                  <tr key={ii} style={{ background: TOKENS.surface }}>
                    <SectionRow label={item.name} amount={item.amount} indent={section.indent} />
                  </tr>
                ))}
                <tr style={{ background: section.isTotal ? TOKENS.accentLight : TOKENS.bg }}>
                  <td style={{ padding: "8px 12px", fontWeight: 700, fontSize: "0.85rem", color: section.isTotal ? TOKENS.accent : TOKENS.primary, borderTop: `2px solid ${section.isTotal ? TOKENS.accent : TOKENS.primary}`, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>
                    {section.isSubtotalBold !== false ? `Total ${section.label}` : section.label}
                  </td>
                  <td style={{ padding: "8px 12px", textAlign: "right", fontWeight: 700, fontSize: "0.85rem", color: section.isTotal ? TOKENS.accent : TOKENS.primary, borderTop: `2px solid ${section.isTotal ? TOKENS.accent : TOKENS.primary}`, borderBottom: `1px solid ${TOKENS.borderSubtle}`, fontFamily: TOKENS.fontUi }}>
                    {formatCurrency(section.subtotal)}
                  </td>
                </tr>
              </Fragment>
            ))}
          </tbody>
        </table>
      </div>
    );
  }

  function renderBalanceSheet() {
    if (balanceSheet.length === 0 && !loading) {
      return (
        <div style={{ textAlign: "center", padding: 40, color: TOKENS.textTertiary }}>
          <FileText size={32} style={{ marginBottom: 8 }} />
          <p>No balance sheet data for the selected period</p>
        </div>
      );
    }
    const totalAssets = balanceSheet.filter(s => s.label.includes("Asset")).reduce((s, sec) => s + sec.subtotal, 0);
    const totalLiabilities = balanceSheet.filter(s => s.label.includes("Liabilit")).reduce((s, sec) => s + sec.subtotal, 0);
    const equity = balanceSheet.filter(s => s.label.includes("Equit")).reduce((s, sec) => s + sec.subtotal, 0);
    const balanced = totalAssets === totalLiabilities + equity;

    return (
      <div style={{ overflowX: "auto" }}>
        <table style={{ width: "100%", borderCollapse: "collapse" }}>
          <thead>
            <tr>
              <th style={{ padding: "10px 12px", textAlign: "left", fontWeight: 600, fontSize: "0.8rem", color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: `2px solid ${TOKENS.border}` }}>Description</th>
              <th style={{ padding: "10px 12px", textAlign: "right", fontWeight: 600, fontSize: "0.8rem", color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: `2px solid ${TOKENS.border}`, width: 180 }}>Amount (₦)</th>
            </tr>
          </thead>
          <tbody>
            {balanceSheet.map((section, si) => (
              <Fragment key={si}>
                <tr style={{ background: TOKENS.primaryLight }}>
                  <td colSpan={2} style={{ padding: "10px 12px", fontWeight: 700, fontSize: "0.9rem", color: TOKENS.primary, borderBottom: `1px solid ${TOKENS.border}`, fontFamily: TOKENS.fontHeading }}>{section.label}</td>
                </tr>
                {section.items.map((item, ii) => (
                  <tr key={ii}>
                    <SectionRow label={item.name} amount={item.amount} indent />
                  </tr>
                ))}
                <tr style={{ background: TOKENS.bg }}>
                  <td style={{ padding: "8px 12px", fontWeight: 700, fontSize: "0.85rem", color: TOKENS.primary, borderTop: `1px solid ${TOKENS.primary}`, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>
                    Total {section.label}
                  </td>
                  <td style={{ padding: "8px 12px", textAlign: "right", fontWeight: 700, fontSize: "0.85rem", color: TOKENS.primary, borderTop: `1px solid ${TOKENS.primary}`, borderBottom: `1px solid ${TOKENS.borderSubtle}`, fontFamily: TOKENS.fontUi }}>
                    {formatCurrency(section.subtotal)}
                  </td>
                </tr>
              </Fragment>
            ))}
            <tr style={{ background: balanced ? TOKENS.successLight : TOKENS.errorLight }}>
              <td style={{ padding: "12px", fontWeight: 700, fontSize: "0.9rem", color: balanced ? TOKENS.success : TOKENS.error, borderTop: `2px solid ${balanced ? TOKENS.success : TOKENS.error}` }}>
                Balance Check
              </td>
              <td style={{ padding: "12px", textAlign: "right", fontWeight: 700, fontSize: "0.85rem", color: balanced ? TOKENS.success : TOKENS.error, borderTop: `2px solid ${balanced ? TOKENS.success : TOKENS.error}`, fontFamily: TOKENS.fontUi }}>
                <StatusIndicator balanced={balanced} />
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    );
  }

  function renderCashFlow() {
    if (cashFlow.length === 0 && !loading) {
      return (
        <div style={{ textAlign: "center", padding: 40, color: TOKENS.textTertiary }}>
          <BarChart3 size={32} style={{ marginBottom: 8 }} />
          <p>No cash flow data for the selected period</p>
        </div>
      );
    }
    const netChange = cashFlow.reduce((s, sec) => s + sec.subtotal, 0);
    return (
      <div style={{ overflowX: "auto" }}>
        <table style={{ width: "100%", borderCollapse: "collapse" }}>
          <thead>
            <tr>
              <th style={{ padding: "10px 12px", textAlign: "left", fontWeight: 600, fontSize: "0.8rem", color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: `2px solid ${TOKENS.border}` }}>Description</th>
              <th style={{ padding: "10px 12px", textAlign: "right", fontWeight: 600, fontSize: "0.8rem", color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: `2px solid ${TOKENS.border}`, width: 180 }}>Amount (₦)</th>
            </tr>
          </thead>
          <tbody>
            {cashFlow.map((section, si) => (
              <Fragment key={si}>
                <tr style={{ background: TOKENS.primaryLight }}>
                  <td colSpan={2} style={{ padding: "10px 12px", fontWeight: 700, fontSize: "0.9rem", color: TOKENS.primary, borderBottom: `1px solid ${TOKENS.border}`, fontFamily: TOKENS.fontHeading }}>
                    {section.label === "operating" ? "Cash Flows from Operating Activities" : section.label === "investing" ? "Cash Flows from Investing Activities" : section.label === "financing" ? "Cash Flows from Financing Activities" : section.label}
                  </td>
                </tr>
                {section.items.map((item, ii) => (
                  <tr key={ii}>
                    <SectionRow label={item.name} amount={item.amount} indent />
                  </tr>
                ))}
                <tr style={{ background: TOKENS.bg }}>
                  <td style={{ padding: "8px 12px", fontWeight: 700, fontSize: "0.85rem", color: TOKENS.primary, borderTop: `1px solid ${TOKENS.primary}`, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>
                    Net Cash from {section.label.charAt(0).toUpperCase() + section.label.slice(1)} Activities
                  </td>
                  <td style={{ padding: "8px 12px", textAlign: "right", fontWeight: 700, fontSize: "0.85rem", color: TOKENS.primary, borderTop: `1px solid ${TOKENS.primary}`, borderBottom: `1px solid ${TOKENS.borderSubtle}`, fontFamily: TOKENS.fontUi }}>
                    {formatCurrency(section.subtotal)}
                  </td>
                </tr>
              </Fragment>
            ))}
            <tr style={{ background: TOKENS.accentLight }}>
              <td style={{ padding: "12px", fontWeight: 700, fontSize: "0.9rem", color: TOKENS.accent, borderTop: `2px solid ${TOKENS.accent}`, fontFamily: TOKENS.fontHeading }}>
                Net Change in Cash and Cash Equivalents
              </td>
              <td style={{ padding: "12px", textAlign: "right", fontWeight: 700, fontSize: "0.9rem", color: netChange >= 0 ? TOKENS.success : TOKENS.error, borderTop: `2px solid ${TOKENS.accent}`, fontFamily: TOKENS.fontUi }}>
                {formatCurrency(netChange)}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    );
  }

  function renderRetainedEarnings() {
    if (retainedEarnings.length === 0 && !loading) {
      return (
        <div style={{ textAlign: "center", padding: 40, color: TOKENS.textTertiary }}>
          <FileText size={32} style={{ marginBottom: 8 }} />
          <p>No retained earnings data for the selected period</p>
        </div>
      );
    }
    return (
      <div style={{ overflowX: "auto" }}>
        <table style={{ width: "100%", borderCollapse: "collapse" }}>
          <thead>
            <tr>
              <th style={{ padding: "10px 12px", textAlign: "left", fontWeight: 600, fontSize: "0.8rem", color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: `2px solid ${TOKENS.border}` }}>Description</th>
              <th style={{ padding: "10px 12px", textAlign: "right", fontWeight: 600, fontSize: "0.8rem", color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: `2px solid ${TOKENS.border}`, width: 180 }}>Amount (₦)</th>
            </tr>
          </thead>
          <tbody>
            {retainedEarnings.map((row, i) => (
              <tr key={i} style={{ background: i === retainedEarnings.length - 1 ? TOKENS.accentLight : TOKENS.surface }}>
                <td style={{ padding: "8px 12px", fontWeight: i === retainedEarnings.length - 1 ? 700 : 400, fontSize: "0.85rem", color: i === retainedEarnings.length - 1 ? TOKENS.accent : TOKENS.text, borderBottom: i === retainedEarnings.length - 1 ? `2px solid ${TOKENS.accent}` : `1px solid ${TOKENS.borderSubtle}` }}>
                  {row.description}
                </td>
                <td style={{ padding: "8px 12px", textAlign: "right", fontWeight: i === retainedEarnings.length - 1 ? 700 : 400, fontSize: "0.85rem", color: i === retainedEarnings.length - 1 ? TOKENS.accent : TOKENS.text, borderBottom: i === retainedEarnings.length - 1 ? `2px solid ${TOKENS.accent}` : `1px solid ${TOKENS.borderSubtle}`, fontFamily: TOKENS.fontUi }}>
                  {formatCurrency(row.amount)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  }

  function renderReportContent() {
    switch (activeTab) {
      case "trial_balance": return renderTrialBalance();
      case "income_statement": return renderIncomeStatement();
      case "balance_sheet": return renderBalanceSheet();
      case "cash_flow": return renderCashFlow();
      case "retained_earnings": return renderRetainedEarnings();
    }
  }

  return (
    <WorkspaceShell>
      <div style={{ padding: 24, minHeight: "100%" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#f0f4f8", display: "flex", alignItems: "center", gap: 8 }}>
            <BarChart3 size={20} /> Financial Reports
          </h1>
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <button onClick={handlePrint} style={exportButtonStyle}>
              <Printer size={14} /> Print
            </button>
            <button onClick={() => handleExport("pdf")} style={exportButtonStyle}>
              <Download size={14} /> PDF
            </button>
            <button onClick={() => handleExport("xlsx")} style={exportButtonStyle}>
              <Download size={14} /> Excel
            </button>
            <button onClick={() => handleExport("csv")} style={exportButtonStyle}>
              <Download size={14} /> CSV
            </button>
          </div>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 16, borderBottom: `2px solid #2a3754`, overflowX: "auto" }}>
          {REPORT_TABS.map((t) => (
            <button key={t.id} onClick={() => setActiveTab(t.id)} style={tabButtonStyle(activeTab === t.id)}>
              {t.label}
            </button>
          ))}
        </div>

        <div style={{ background: TOKENS.surface, border: `1px solid ${TOKENS.border}`, borderRadius: TOKENS.radiusMd, padding: 16, marginBottom: 16, boxShadow: TOKENS.shadowSm, display: "flex", flexWrap: "wrap", gap: 12, alignItems: "flex-end" }}>
          <div>
            <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em" }}>Company</label>
            <select value={selectedCompany} onChange={(e) => setSelectedCompany(e.target.value)} style={selectStyle}>
              <option value="">Select Company</option>
              {companies.map((c) => (
                <option key={c.id} value={c.id}>{c.name} ({c.rc_number})</option>
              ))}
            </select>
          </div>
          <div>
            <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em" }}>Period From</label>
            <input type="month" value={periodFrom} onChange={(e) => setPeriodFrom(e.target.value)} style={inputStyle} />
          </div>
          <div>
            <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase", letterSpacing: "0.05em" }}>Period To</label>
            <input type="month" value={periodTo} onChange={(e) => setPeriodTo(e.target.value)} style={inputStyle} />
          </div>
          <div>
            <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: "0.8rem", color: TOKENS.textSecondary, cursor: "pointer", marginBottom: 4 }}>
              <input type="checkbox" checked={showComparison} onChange={(e) => setShowComparison(e.target.checked)} style={{ accentColor: TOKENS.primary }} />
              Comparison Period
            </label>
            {showComparison && (
              <div style={{ display: "flex", gap: 8 }}>
                <input type="month" value={comparisonFrom} onChange={(e) => setComparisonFrom(e.target.value)} style={inputStyle} placeholder="From" />
                <input type="month" value={comparisonTo} onChange={(e) => setComparisonTo(e.target.value)} style={inputStyle} placeholder="To" />
              </div>
            )}
          </div>
          <button onClick={fetchReport} style={{ padding: "6px 16px", borderRadius: TOKENS.radiusSm, fontSize: "0.85rem", fontWeight: 600, background: TOKENS.primary, color: "#FFFFFF", border: "none", cursor: "pointer", transition: "all 150ms ease" }}>
            Generate
          </button>
        </div>

        <div style={{ background: TOKENS.surface, border: `1px solid ${TOKENS.border}`, borderRadius: TOKENS.radiusMd, padding: 20, boxShadow: TOKENS.shadowSm }}>
          {error && (
            <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "10px 14px", marginBottom: 16, background: TOKENS.errorLight, border: `1px solid ${TOKENS.error}`, borderRadius: TOKENS.radiusSm, color: TOKENS.error, fontSize: "0.85rem" }}>
              <AlertCircle size={16} />
              {error}
            </div>
          )}

          {loading ? (
            <div style={{ display: "flex", justifyContent: "center", padding: 40 }}>
              <Loader2 size={24} style={{ animation: "spin 0.8s linear infinite", color: TOKENS.primary }} />
            </div>
          ) : (
            <>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
                <div>
                  <h2 style={{ fontSize: "1.1rem", fontWeight: 600, color: TOKENS.text, fontFamily: TOKENS.fontHeading }}>
                    {REPORT_TABS.find((t) => t.id === activeTab)?.label}
                  </h2>
                  <p style={{ fontSize: "0.8rem", color: TOKENS.textTertiary, marginTop: 2 }}>
                    {periodFrom} to {periodTo} {selectedCompany ? `— ${companies.find(c => c.id === selectedCompany)?.name || ""}` : ""}
                  </p>
                </div>
              </div>
              {renderReportContent()}
            </>
          )}
        </div>

        <style>{`
          @media print {
            aside, header { display: none !important; }
            main { background: #FFFFFF !important; }
            div { box-shadow: none !important; }
          }
          @keyframes spin { to { transform: rotate(360deg); } }
        `}</style>
      </div>
    </WorkspaceShell>
  );
}
