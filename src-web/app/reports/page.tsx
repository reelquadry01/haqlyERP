"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { getToken } from "@/lib/session";
import { apiGet } from "@/lib/api";
import { BarChart3, Download } from "lucide-react";

interface ReportOption {
  id: string;
  name: string;
  description: string;
  category: string;
}

const REPORTS: ReportOption[] = [
  { id: "trial_balance", name: "Trial Balance", description: "List of all account balances at a point in time", category: "Financial Statements" },
  { id: "profit_loss", name: "Profit & Loss", description: "Income statement for a period", category: "Financial Statements" },
  { id: "balance_sheet", name: "Balance Sheet", description: "Statement of financial position", category: "Financial Statements" },
  { id: "cash_flow", name: "Cash Flow Statement", description: "Cash inflows and outflows by activity", category: "Financial Statements" },
  { id: "general_ledger", name: "General Ledger", description: "Detailed transaction history per account", category: "Detailed Reports" },
  { id: "accounts_receivable", name: "Accounts Receivable Aging", description: "Outstanding receivables by age", category: "Detailed Reports" },
  { id: "accounts_payable", name: "Accounts Payable Aging", description: "Outstanding payables by age", category: "Detailed Reports" },
  { id: "vat_return", name: "VAT Return", description: "VAT output vs input computation", category: "Tax Reports" },
  { id: "wht_schedule", name: "WHT Schedule", description: "Withholding tax deduction schedule", category: "Tax Reports" },
  { id: "paye_schedule", name: "PAYE Schedule", description: "Pay-As-You-Earn tax schedule", category: "Tax Reports" },
  { id: "financial_ratios", name: "Financial Ratios", description: "Key financial performance ratios", category: "Analytics" },
  { id: "budget_variance", name: "Budget vs Actual", description: "Budget variance analysis", category: "Analytics" },
];

const CATEGORIES = [...new Set(REPORTS.map((r) => r.category))];

export default function ReportsPage() {
  const token = getToken();
  const [selectedReport, setSelectedReport] = useState<string | null>(null);
  const [periodFrom, setPeriodFrom] = useState(new Date().toISOString().split("T")[0].slice(0, 7));
  const [periodTo, setPeriodTo] = useState(new Date().toISOString().split("T")[0].slice(0, 7));

  async function handleExport(format: "pdf" | "xlsx") {
    if (!selectedReport) return;
    try {
      const res = await apiGet(`/reports/${selectedReport}?from=${periodFrom}&to=${periodTo}&format=${format}`, token);
      if (res.ok) {
        const blob = await res.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `${selectedReport}_${periodFrom}_${periodTo}.${format}`;
        a.click();
        URL.revokeObjectURL(url);
      }
    } catch {
      // offline
    }
  }

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <BarChart3 size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> Reports
          </h1>
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input type="month" value={periodFrom} onChange={(e) => setPeriodFrom(e.target.value)} style={{ fontSize: "0.85rem" }} />
            <span style={{ color: "#868E96", fontSize: "0.85rem" }}>to</span>
            <input type="month" value={periodTo} onChange={(e) => setPeriodTo(e.target.value)} style={{ fontSize: "0.85rem" }} />
          </div>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 2fr", gap: 20 }}>
          <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
            {CATEGORIES.map((cat) => (
              <div key={cat} style={{ marginBottom: 20 }}>
                <h3 style={{ fontSize: "0.8rem", fontWeight: 600, color: "#868E96", textTransform: "uppercase", letterSpacing: "0.05em", marginBottom: 8 }}>{cat}</h3>
                {REPORTS.filter((r) => r.category === cat).map((report) => (
                  <button
                    key={report.id}
                    onClick={() => setSelectedReport(report.id)}
                    style={{
                      display: "block",
                      width: "100%",
                      textAlign: "left",
                      padding: "10px 12px",
                      borderRadius: 8,
                      fontSize: "0.85rem",
                      fontWeight: selectedReport === report.id ? 600 : 400,
                      color: selectedReport === report.id ? "#1B4332" : "#495057",
                      background: selectedReport === report.id ? "rgba(27,67,50,0.08)" : "transparent",
                      border: selectedReport === report.id ? "1px solid rgba(27,67,50,0.2)" : "1px solid transparent",
                      marginBottom: 4,
                      transition: "all 150ms ease",
                    }}
                  >
                    {report.name}
                  </button>
                ))}
              </div>
            ))}
          </div>

          <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
            {selectedReport ? (
              <>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
                  <div>
                    <h2 style={{ fontSize: "1.1rem", fontWeight: 600, color: "#1A1A2E" }}>
                      {REPORTS.find((r) => r.id === selectedReport)?.name}
                    </h2>
                    <p style={{ fontSize: "0.8rem", color: "#868E96", marginTop: 2 }}>
                      {REPORTS.find((r) => r.id === selectedReport)?.description}
                    </p>
                  </div>
                  <div style={{ display: "flex", gap: 8 }}>
                    <button className="btn btn-secondary btn-sm" onClick={() => handleExport("pdf")} style={{ display: "flex", alignItems: "center", gap: 4 }}>
                      <Download size={14} /> PDF
                    </button>
                    <button className="btn btn-secondary btn-sm" onClick={() => handleExport("xlsx")} style={{ display: "flex", alignItems: "center", gap: 4 }}>
                      <Download size={14} /> Excel
                    </button>
                  </div>
                </div>
                <div style={{ height: 400, display: "flex", alignItems: "center", justifyContent: "center", border: "1px dashed #E9ECEF", borderRadius: 8, color: "#868E96", fontSize: "0.85rem" }}>
                  Report preview will load here
                </div>
              </>
            ) : (
              <div style={{ height: 400, display: "flex", alignItems: "center", justifyContent: "center", color: "#868E96" }}>
                Select a report to preview
              </div>
            )}
          </div>
        </div>
      </div>
    </WorkspaceShell>
  );
}
