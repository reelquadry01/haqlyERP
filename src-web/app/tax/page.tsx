// Author: Quadri Atharu

"use client";

import { useEffect, useState, useCallback, useMemo } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { getToken, getCompanyContext } from "@/lib/session";
import { apiGet, apiPost } from "@/lib/api";
import {
  Receipt,
  RefreshCw,
  AlertTriangle,
  CheckCircle,
  Clock,
  DollarSign,
  Calculator,
  Calendar,
  FileDown,
  Info,
  TrendingUp,
  Shield,
} from "lucide-react";

interface TaxSummary {
  vat_collected: number;
  vat_paid: number;
  wht_deducted: number;
  cit_estimate: number;
  education_tax: number;
  total_tax_liability: number;
}

interface VatEntry {
  id: string;
  date: string;
  type: "sales" | "purchase";
  description: string;
  taxable_amount: number;
  vat_rate: number;
  vat_amount: number;
  invoice_ref: string;
}

interface WhtEntry {
  id: string;
  date: string;
  vendor_name: string;
  payment_type: string;
  gross_amount: number;
  wht_rate: number;
  wht_amount: number;
  certificate_no: string;
  status: string;
}

interface TaxDeadline {
  id: string;
  tax_type: string;
  period: string;
  due_date: string;
  amount: number;
  status: "overdue" | "upcoming" | "filed";
  description: string;
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

const TABS = ["overview", "vat", "wht", "cit", "calendar"] as const;
type TabType = (typeof TABS)[number];

const TAB_LABELS: Record<TabType, string> = {
  overview: "Tax Summary",
  vat: "VAT Return",
  wht: "WHT Register",
  cit: "CIT Calculator",
  calendar: "Filing Calendar",
};

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

export default function TaxPage() {
  const [activeTab, setActiveTab] = useState<TabType>("overview");
  const [summary, setSummary] = useState<TaxSummary | null>(null);
  const [vatEntries, setVatEntries] = useState<VatEntry[]>([]);
  const [whtEntries, setWhtEntries] = useState<WhtEntry[]>([]);
  const [deadlines, setDeadlines] = useState<TaxDeadline[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [generatingReturn, setGeneratingReturn] = useState<string | null>(null);

  const [citProfit, setCitProfit] = useState<number>(0);
  const [citSmallBusiness, setCitSmallBusiness] = useState(false);

  const citCalculation = useMemo(() => {
    if (!citProfit || citProfit <= 0) return { taxable: 0, tax: 0, rate: 0, exemption: 0 };
    const smallExemptLimit = 25000000;
    const isSmall = citSmallBusiness && citProfit <= smallExemptLimit;
    if (isSmall) {
      const taxable = citProfit * 0.2;
      const tax = taxable * 0.2;
      return { taxable, tax, rate: 20, exemption: taxable * 0.8 };
    }
    const first25m = Math.min(citProfit, 25000000);
    const above25m = Math.max(citProfit - 25000000, 0);
    const tax = first25m * 0.15 + above25m * 0.30;
    const effectiveRate = (tax / citProfit) * 100;
    return { taxable: citProfit, tax, rate: effectiveRate, exemption: 0 };
  }, [citProfit, citSmallBusiness]);

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
      const [sumRes, vatRes, whtRes, calRes] = await Promise.all([
        apiGet(`/tax/summary?companyId=${companyId}`, token),
        apiGet(`/tax/vat?companyId=${companyId}`, token),
        apiGet(`/tax/wht?companyId=${companyId}`, token),
        apiGet(`/tax/deadlines?companyId=${companyId}`, token),
      ]);
      if (sumRes.ok) {
        const d = await sumRes.json();
        setSummary(d.data || d);
      }
      if (vatRes.ok) {
        const d = await vatRes.json();
        setVatEntries(d.data || d.entries || []);
      }
      if (whtRes.ok) {
        const d = await whtRes.json();
        setWhtEntries(d.data || d.entries || []);
      }
      if (calRes.ok) {
        const d = await calRes.json();
        setDeadlines(d.data || d.deadlines || []);
      }
      if (!sumRes.ok && !vatRes.ok) setError("Failed to load tax data.");
    } catch {
      setError("Network error. Please try again.");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  async function handleGenerateReturn(taxType: string) {
    setGeneratingReturn(taxType);
    const token = getToken();
    try {
      const companyId = getCompanyContext() || "";
      const res = await apiPost(`/tax/generate-return`, { tax_type: taxType, companyId }, token);
      if (res.ok) {
        const d = await res.json();
        if (d.download_url) {
          window.open(d.download_url, "_blank");
        }
      }
    } catch {
      // handle error silently
    } finally {
      setGeneratingReturn(null);
    }
  }

  const vatSales = vatEntries.filter((e) => e.type === "sales");
  const vatPurchases = vatEntries.filter((e) => e.type === "purchase");
  const totalVatSales = vatSales.reduce((s, e) => s + e.vat_amount, 0);
  const totalVatPurchases = vatPurchases.reduce((s, e) => s + e.vat_amount, 0);
  const netVat = totalVatSales - totalVatPurchases;

  const deadlineStatusColors: Record<string, { bg: string; color: string }> = {
    overdue: { bg: "rgba(220,38,38,0.12)", color: ERROR },
    upcoming: { bg: "rgba(212,175,55,0.15)", color: ACCENT },
    filed: { bg: "rgba(22,163,74,0.12)", color: SUCCESS },
  };

  function renderSummaryCard(
    title: string,
    value: number,
    icon: React.ReactNode,
    accent: string
  ) {
    return (
      <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: 12 }}>
          <span style={{ fontSize: "0.8rem", color: TEXT_SEC, fontWeight: 500 }}>{title}</span>
          <div style={{ width: 36, height: 36, borderRadius: 8, background: `${accent}15`, display: "flex", alignItems: "center", justifyContent: "center", color: accent }}>{icon}</div>
        </div>
        <div style={{ fontSize: "1.5rem", fontWeight: 700, color: TEXT, fontFamily: '"JetBrains Mono", monospace' }}>{formatNaira(value)}</div>
      </div>
    );
  }

  if (error && !loading) {
    return (
      <WorkspaceShell>
        <div style={{ padding: 24, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", minHeight: "60vh" }}>
          <AlertTriangle size={48} style={{ color: ERROR, marginBottom: 16 }} />
          <h2 style={{ fontSize: "1.1rem", fontWeight: 600, color: TEXT, marginBottom: 8 }}>Failed to Load Tax Data</h2>
          <p style={{ fontSize: "0.85rem", color: TEXT_SEC, marginBottom: 20 }}>{error}</p>
          <button onClick={loadData} style={{ display: "flex", alignItems: "center", gap: 8, padding: "10px 20px", borderRadius: 6, background: PRIMARY, color: "#FFFFFF", fontSize: "0.85rem", fontWeight: 500, border: "none", cursor: "pointer" }}>
            <RefreshCw size={16} /> Retry
          </button>
        </div>
      </WorkspaceShell>
    );
  }

  return (
    <WorkspaceShell>
      <div style={{ padding: 24, background: BG, minHeight: "100%" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 24, flexWrap: "wrap", gap: 12 }}>
          <h1 style={{ fontSize: "1.5rem", fontWeight: 700, color: TEXT, fontFamily: '"DM Serif Display", serif', display: "flex", alignItems: "center", gap: 8 }}>
            <Receipt size={22} style={{ color: PRIMARY }} />
            Tax Dashboard
          </h1>
          <button onClick={loadData} style={{ padding: 8, borderRadius: 6, background: SURFACE, border: `1px solid ${BORDER}`, cursor: "pointer" }}>
            <RefreshCw size={16} style={{ color: TEXT_SEC }} />
          </button>
        </div>

        {/* Tabs */}
        <div style={{ display: "flex", gap: 2, marginBottom: 20, borderBottom: `2px solid ${BORDER}`, overflowX: "auto" }}>
          {TABS.map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              style={{
                padding: "10px 16px",
                fontSize: "0.82rem",
                fontWeight: activeTab === tab ? 600 : 400,
                color: activeTab === tab ? PRIMARY : TEXT_TER,
                borderBottom: activeTab === tab ? `2px solid ${PRIMARY}` : "2px solid transparent",
                marginBottom: -2,
                whiteSpace: "nowrap",
                cursor: "pointer",
                background: "transparent",
                borderLeft: "none",
                borderRight: "none",
                borderTop: "none",
              }}
            >
              {TAB_LABELS[tab]}
            </button>
          ))}
        </div>

        {/* Overview Tab */}
        {activeTab === "overview" && (
          <div>
            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))", gap: 16, marginBottom: 24 }}>
              {loading ? (
                Array.from({ length: 4 }).map((_, i) => (
                  <div key={i} style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, padding: 20 }}>
                    <Skeleton w="60%" h={14} />
                    <div style={{ height: 12 }} />
                    <Skeleton w="80%" h={28} />
                  </div>
                ))
              ) : (
                <>
                  {renderSummaryCard("VAT Collected", summary?.vat_collected || 0, <DollarSign size={18} />, ACCENT)}
                  {renderSummaryCard("WHT Deducted", summary?.wht_deducted || 0, <Shield size={18} />, "#2563EB")}
                  {renderSummaryCard("CIT Estimate", summary?.cit_estimate || 0, <TrendingUp size={18} />, PRIMARY)}
                  {renderSummaryCard("Education Tax", summary?.education_tax || 0, <Info size={18} />, "#7C3AED")}
                </>
              )}
            </div>

            <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
              <h3 style={{ fontSize: "0.95rem", fontWeight: 600, color: TEXT, fontFamily: '"DM Serif Display", serif', marginBottom: 16 }}>Total Tax Liability</h3>
              <div style={{ display: "flex", alignItems: "center", gap: 12, padding: 16, background: `${PRIMARY}08`, borderRadius: 8, border: `1px solid ${PRIMARY}20` }}>
                <div style={{ width: 48, height: 48, borderRadius: 8, background: PRIMARY, display: "flex", alignItems: "center", justifyContent: "center", color: "#FFFFFF" }}>
                  <Receipt size={22} />
                </div>
                <div>
                  <div style={{ fontSize: "0.78rem", color: TEXT_SEC, marginBottom: 2 }}>Total Estimated Tax Liability</div>
                  <div style={{ fontSize: "2rem", fontWeight: 700, color: PRIMARY, fontFamily: '"JetBrains Mono", monospace' }}>
                    {formatNaira(summary?.total_tax_liability || 0)}
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* VAT Return Tab */}
        {activeTab === "vat" && (
          <div>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
              <h3 style={{ fontSize: "0.95rem", fontWeight: 600, color: TEXT, fontFamily: '"DM Serif Display", serif' }}>VAT Return Summary</h3>
              <button
                onClick={() => handleGenerateReturn("vat")}
                disabled={generatingReturn === "vat"}
                style={{ display: "flex", alignItems: "center", gap: 6, padding: "8px 14px", borderRadius: 6, background: PRIMARY, color: "#FFFFFF", fontSize: "0.82rem", fontWeight: 500, border: "none", cursor: generatingReturn === "vat" ? "not-allowed" : "pointer" }}
              >
                <FileDown size={14} /> {generatingReturn === "vat" ? "Generating..." : "Generate VAT Return"}
              </button>
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", gap: 12, marginBottom: 20 }}>
              <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, padding: 16 }}>
                <div style={{ fontSize: "0.75rem", color: TEXT_TER, marginBottom: 4, textTransform: "uppercase" }}>Output VAT (Sales)</div>
                <div style={{ fontSize: "1.2rem", fontWeight: 700, color: TEXT, fontFamily: '"JetBrains Mono", monospace' }}>{formatNaira(totalVatSales)}</div>
              </div>
              <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, padding: 16 }}>
                <div style={{ fontSize: "0.75rem", color: TEXT_TER, marginBottom: 4, textTransform: "uppercase" }}>Input VAT (Purchases)</div>
                <div style={{ fontSize: "1.2rem", fontWeight: 700, color: TEXT, fontFamily: '"JetBrains Mono", monospace' }}>{formatNaira(totalVatPurchases)}</div>
              </div>
              <div style={{ background: netVat >= 0 ? `${ACCENT}10` : `${ERROR}10`, border: `1px solid ${netVat >= 0 ? ACCENT : ERROR}30`, borderRadius: 8, padding: 16 }}>
                <div style={{ fontSize: "0.75rem", color: TEXT_TER, marginBottom: 4, textTransform: "uppercase" }}>
                  {netVat >= 0 ? "Net VAT Payable" : "Net VAT Refundable"}
                </div>
                <div style={{ fontSize: "1.2rem", fontWeight: 700, color: netVat >= 0 ? ACCENT : ERROR, fontFamily: '"JetBrains Mono", monospace' }}>
                  {formatNaira(Math.abs(netVat))}
                </div>
              </div>
            </div>

            {/* VAT Sales Table */}
            <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, marginBottom: 16, overflow: "hidden" }}>
              <div style={{ padding: "12px 16px", borderBottom: `1px solid ${BORDER}`, fontWeight: 600, fontSize: "0.82rem", color: PRIMARY }}>
                Sales VAT Collected
              </div>
              {loading ? (
                <div style={{ padding: 20 }}><Skeleton h={14} /></div>
              ) : vatSales.length === 0 ? (
                <div style={{ padding: 20, textAlign: "center", color: TEXT_TER, fontSize: "0.82rem" }}>No sales VAT entries</div>
              ) : (
                <table style={{ width: "100%", borderCollapse: "collapse" }}>
                  <thead>
                    <tr>
                      {["Date", "Description", "Ref", "Taxable", "VAT"].map((h) => (
                        <th key={h} style={{ padding: "8px 12px", textAlign: h === "Taxable" || h === "VAT" ? "right" : "left", fontSize: "0.72rem", fontWeight: 600, color: TEXT_TER, textTransform: "uppercase", borderBottom: `1px solid ${BORDER}` }}>{h}</th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {vatSales.map((e) => (
                      <tr key={e.id} style={{ borderBottom: "1px solid #E9ECEF" }}>
                        <td style={{ padding: "8px 12px", fontSize: "0.82rem", color: TEXT_SEC }}>{e.date}</td>
                        <td style={{ padding: "8px 12px", fontSize: "0.82rem", color: TEXT }}>{e.description}</td>
                        <td style={{ padding: "8px 12px", fontSize: "0.82rem", color: TEXT_TER, fontFamily: '"JetBrains Mono", monospace' }}>{e.invoice_ref}</td>
                        <td style={{ padding: "8px 12px", fontSize: "0.82rem", textAlign: "right", color: TEXT_SEC, fontFamily: '"JetBrains Mono", monospace' }}>{formatNaira(e.taxable_amount)}</td>
                        <td style={{ padding: "8px 12px", fontSize: "0.82rem", textAlign: "right", color: ACCENT, fontFamily: '"JetBrains Mono", monospace', fontWeight: 500 }}>{formatNaira(e.vat_amount)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>

            {/* VAT Purchases Table */}
            <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, overflow: "hidden" }}>
              <div style={{ padding: "12px 16px", borderBottom: `1px solid ${BORDER}`, fontWeight: 600, fontSize: "0.82rem", color: PRIMARY }}>
                Purchase VAT Paid
              </div>
              {loading ? (
                <div style={{ padding: 20 }}><Skeleton h={14} /></div>
              ) : vatPurchases.length === 0 ? (
                <div style={{ padding: 20, textAlign: "center", color: TEXT_TER, fontSize: "0.82rem" }}>No purchase VAT entries</div>
              ) : (
                <table style={{ width: "100%", borderCollapse: "collapse" }}>
                  <thead>
                    <tr>
                      {["Date", "Description", "Ref", "Taxable", "VAT"].map((h) => (
                        <th key={h} style={{ padding: "8px 12px", textAlign: h === "Taxable" || h === "VAT" ? "right" : "left", fontSize: "0.72rem", fontWeight: 600, color: TEXT_TER, textTransform: "uppercase", borderBottom: `1px solid ${BORDER}` }}>{h}</th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {vatPurchases.map((e) => (
                      <tr key={e.id} style={{ borderBottom: "1px solid #E9ECEF" }}>
                        <td style={{ padding: "8px 12px", fontSize: "0.82rem", color: TEXT_SEC }}>{e.date}</td>
                        <td style={{ padding: "8px 12px", fontSize: "0.82rem", color: TEXT }}>{e.description}</td>
                        <td style={{ padding: "8px 12px", fontSize: "0.82rem", color: TEXT_TER, fontFamily: '"JetBrains Mono", monospace' }}>{e.invoice_ref}</td>
                        <td style={{ padding: "8px 12px", fontSize: "0.82rem", textAlign: "right", color: TEXT_SEC, fontFamily: '"JetBrains Mono", monospace' }}>{formatNaira(e.taxable_amount)}</td>
                        <td style={{ padding: "8px 12px", fontSize: "0.82rem", textAlign: "right", color: "#2563EB", fontFamily: '"JetBrains Mono", monospace', fontWeight: 500 }}>{formatNaira(e.vat_amount)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          </div>
        )}

        {/* WHT Register Tab */}
        {activeTab === "wht" && (
          <div>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
              <h3 style={{ fontSize: "0.95rem", fontWeight: 600, color: TEXT, fontFamily: '"DM Serif Display", serif' }}>Withholding Tax Register</h3>
              <button
                onClick={() => handleGenerateReturn("wht")}
                disabled={generatingReturn === "wht"}
                style={{ display: "flex", alignItems: "center", gap: 6, padding: "8px 14px", borderRadius: 6, background: PRIMARY, color: "#FFFFFF", fontSize: "0.82rem", fontWeight: 500, border: "none", cursor: generatingReturn === "wht" ? "not-allowed" : "pointer" }}
              >
                <FileDown size={14} /> {generatingReturn === "wht" ? "Generating..." : "Generate WHT Schedule"}
              </button>
            </div>

            {loading ? (
              <div style={{ padding: 40, textAlign: "center" }}><div className="splash-loader" style={{ margin: "0 auto 16px" }} /></div>
            ) : whtEntries.length === 0 ? (
              <div style={{ textAlign: "center", padding: 40, color: TEXT_TER, background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8 }}>
                <Receipt size={40} style={{ marginBottom: 12, opacity: 0.4 }} />
                <p style={{ fontSize: "0.95rem", fontWeight: 500 }}>No WHT deductions recorded</p>
              </div>
            ) : (
              <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, overflow: "hidden" }}>
                <table style={{ width: "100%", borderCollapse: "collapse" }}>
                  <thead>
                    <tr>
                      {["Date", "Vendor", "Payment Type", "Gross", "WHT Rate", "WHT Amount", "Certificate", "Status"].map((h) => (
                        <th key={h} style={{ padding: "8px 10px", textAlign: h === "Gross" || h === "WHT Rate" || h === "WHT Amount" ? "right" : "left", fontSize: "0.72rem", fontWeight: 600, color: TEXT_TER, textTransform: "uppercase", borderBottom: `1px solid ${BORDER}`, whiteSpace: "nowrap" }}>{h}</th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {whtEntries.map((e) => {
                      const statusColors: Record<string, { bg: string; color: string }> = {
                        collected: { bg: "rgba(22,163,74,0.12)", color: SUCCESS },
                        pending: { bg: "rgba(212,175,55,0.15)", color: ACCENT },
                        remitted: { bg: "rgba(59,130,246,0.12)", color: "#2563EB" },
                      };
                      const sc = statusColors[e.status] || statusColors.pending;
                      return (
                        <tr key={e.id} style={{ borderBottom: "1px solid #E9ECEF" }}>
                          <td style={{ padding: "8px 10px", fontSize: "0.82rem", color: TEXT_SEC }}>{e.date}</td>
                          <td style={{ padding: "8px 10px", fontSize: "0.82rem", color: TEXT }}>{e.vendor_name}</td>
                          <td style={{ padding: "8px 10px", fontSize: "0.78rem", color: TEXT_TER }}>{e.payment_type}</td>
                          <td style={{ padding: "8px 10px", fontSize: "0.82rem", textAlign: "right", color: TEXT_SEC, fontFamily: '"JetBrains Mono", monospace' }}>{formatNaira(e.gross_amount)}</td>
                          <td style={{ padding: "8px 10px", fontSize: "0.82rem", textAlign: "right", color: TEXT_TER }}>{e.wht_rate}%</td>
                          <td style={{ padding: "8px 10px", fontSize: "0.82rem", textAlign: "right", color: PRIMARY, fontFamily: '"JetBrains Mono", monospace', fontWeight: 500 }}>{formatNaira(e.wht_amount)}</td>
                          <td style={{ padding: "8px 10px", fontSize: "0.78rem", color: TEXT_SEC, fontFamily: '"JetBrains Mono", monospace' }}>{e.certificate_no || "—"}</td>
                          <td style={{ padding: "8px 10px" }}>
                            <span style={{ fontSize: "0.72rem", padding: "2px 8px", borderRadius: 4, background: sc.bg, color: sc.color, fontWeight: 500 }}>{e.status}</span>
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        )}

        {/* CIT Calculator Tab */}
        {activeTab === "cit" && (
          <div>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
              <h3 style={{ fontSize: "0.95rem", fontWeight: 600, color: TEXT, fontFamily: '"DM Serif Display", serif' }}>Companies Income Tax Calculator</h3>
              <button
                onClick={() => handleGenerateReturn("cit")}
                disabled={generatingReturn === "cit"}
                style={{ display: "flex", alignItems: "center", gap: 6, padding: "8px 14px", borderRadius: 6, background: PRIMARY, color: "#FFFFFF", fontSize: "0.82rem", fontWeight: 500, border: "none", cursor: generatingReturn === "cit" ? "not-allowed" : "pointer" }}
              >
                <FileDown size={14} /> {generatingReturn === "cit" ? "Generating..." : "Generate CIT Return"}
              </button>
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 20, marginBottom: 20 }}>
              <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, padding: 24 }}>
                <h4 style={{ fontSize: "0.88rem", fontWeight: 600, color: TEXT, marginBottom: 16, display: "flex", alignItems: "center", gap: 8 }}>
                  <Calculator size={16} style={{ color: PRIMARY }} /> Input
                </h4>
                <div style={{ marginBottom: 16 }}>
                  <label style={{ display: "block", fontSize: "0.78rem", fontWeight: 500, color: TEXT_SEC, marginBottom: 4 }}>Assessable Profit (₦)</label>
                  <input
                    type="number"
                    value={citProfit || ""}
                    onChange={(e) => setCitProfit(parseFloat(e.target.value) || 0)}
                    placeholder="Enter assessable profit"
                    style={{ width: "100%", padding: "10px 12px", borderRadius: 6, border: `1px solid ${BORDER}`, background: BG, color: TEXT, fontSize: "0.95rem", fontFamily: '"JetBrains Mono", monospace' }}
                  />
                </div>
                <label style={{ display: "flex", alignItems: "center", gap: 8, fontSize: "0.85rem", color: TEXT, cursor: "pointer", marginBottom: 8 }}>
                  <input type="checkbox" checked={citSmallBusiness} onChange={(e) => setCitSmallBusiness(e.target.checked)} style={{ accentColor: PRIMARY }} />
                  Small Business Exemption (Turnover ≤ ₦25M)
                </label>
                <div style={{ fontSize: "0.75rem", color: TEXT_TER, lineHeight: 1.5, padding: 10, background: BG, borderRadius: 6, border: `1px solid #E9ECEF` }}>
                  <Info size={12} style={{ verticalAlign: "middle", marginRight: 4 }} />
                  Small companies (≤₦25M turnover) pay 20% on 20% of profit. Others pay 15% on first ₦25M, 30% on balance per CAMA 2020.
                </div>
              </div>

              <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, padding: 24 }}>
                <h4 style={{ fontSize: "0.88rem", fontWeight: 600, color: TEXT, marginBottom: 16, display: "flex", alignItems: "center", gap: 8 }}>
                  <DollarSign size={16} style={{ color: ACCENT }} /> Computation
                </h4>
                <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                  <div style={{ display: "flex", justifyContent: "space-between", padding: "8px 0", borderBottom: "1px solid #E9ECEF" }}>
                    <span style={{ fontSize: "0.82rem", color: TEXT_SEC }}>Assessable Profit</span>
                    <span style={{ fontSize: "0.82rem", color: TEXT, fontFamily: '"JetBrains Mono", monospace', fontWeight: 500 }}>{formatNaira(citProfit)}</span>
                  </div>
                  <div style={{ display: "flex", justifyContent: "space-between", padding: "8px 0", borderBottom: "1px solid #E9ECEF" }}>
                    <span style={{ fontSize: "0.82rem", color: TEXT_SEC }}>Taxable Amount</span>
                    <span style={{ fontSize: "0.82rem", color: TEXT, fontFamily: '"JetBrains Mono", monospace', fontWeight: 500 }}>{formatNaira(citCalculation.taxable)}</span>
                  </div>
                  {citCalculation.exemption > 0 && (
                    <div style={{ display: "flex", justifyContent: "space-between", padding: "8px 0", borderBottom: "1px solid #E9ECEF" }}>
                      <span style={{ fontSize: "0.82rem", color: SUCCESS }}>Small Business Exemption</span>
                      <span style={{ fontSize: "0.82rem", color: SUCCESS, fontFamily: '"JetBrains Mono", monospace', fontWeight: 500 }}>-{formatNaira(citCalculation.exemption)}</span>
                    </div>
                  )}
                  <div style={{ display: "flex", justifyContent: "space-between", padding: "8px 0", borderBottom: "1px solid #E9ECEF" }}>
                    <span style={{ fontSize: "0.82rem", color: TEXT_SEC }}>Effective Rate</span>
                    <span style={{ fontSize: "0.82rem", color: TEXT, fontFamily: '"JetBrains Mono", monospace' }}>{citCalculation.rate.toFixed(1)}%</span>
                  </div>
                  <div style={{ display: "flex", justifyContent: "space-between", background: `${PRIMARY}08`, margin: "0 -12px", padding: "12px", borderRadius: 6 }}>
                    <span style={{ fontSize: "0.92rem", fontWeight: 600, color: TEXT }}>CIT Payable</span>
                    <span style={{ fontSize: "1.1rem", fontWeight: 700, color: PRIMARY, fontFamily: '"JetBrains Mono", monospace' }}>{formatNaira(citCalculation.tax)}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Filing Calendar Tab */}
        {activeTab === "calendar" && (
          <div>
            <h3 style={{ fontSize: "0.95rem", fontWeight: 600, color: TEXT, fontFamily: '"DM Serif Display", serif', marginBottom: 16, display: "flex", alignItems: "center", gap: 8 }}>
              <Calendar size={18} style={{ color: PRIMARY }} /> Filing Calendar
            </h3>

            {loading ? (
              <div style={{ padding: 40, textAlign: "center" }}><div className="splash-loader" style={{ margin: "0 auto 16px" }} /></div>
            ) : deadlines.length === 0 ? (
              <div style={{ textAlign: "center", padding: 40, color: TEXT_TER, background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8 }}>
                <Calendar size={40} style={{ marginBottom: 12, opacity: 0.4 }} />
                <p style={{ fontSize: "0.95rem", fontWeight: 500 }}>No upcoming deadlines</p>
              </div>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                {deadlines.map((d) => {
                  const sc = deadlineStatusColors[d.status] || deadlineStatusColors.upcoming;
                  const StatusIcon = d.status === "filed" ? CheckCircle : d.status === "overdue" ? AlertTriangle : Clock;
                  return (
                    <div
                      key={d.id}
                      style={{
                        background: SURFACE,
                        border: `1px solid ${BORDER}`,
                        borderRadius: 8,
                        padding: 16,
                        display: "flex",
                        alignItems: "center",
                        gap: 14,
                        borderLeft: `3px solid ${sc.color}`,
                      }}
                    >
                      <div style={{ width: 40, height: 40, borderRadius: 8, background: sc.bg, display: "flex", alignItems: "center", justifyContent: "center", color: sc.color }}>
                        <StatusIcon size={18} />
                      </div>
                      <div style={{ flex: 1 }}>
                        <div style={{ fontSize: "0.88rem", fontWeight: 600, color: TEXT, marginBottom: 2 }}>{d.description}</div>
                        <div style={{ fontSize: "0.78rem", color: TEXT_TER }}>
                          {d.tax_type} — Period: {d.period}
                        </div>
                      </div>
                      <div style={{ textAlign: "right" }}>
                        <div style={{ fontSize: "0.88rem", fontWeight: 500, color: TEXT, fontFamily: '"JetBrains Mono", monospace' }}>{formatNaira(d.amount)}</div>
                        <div style={{ fontSize: "0.78rem", color: TEXT_SEC }}>Due: {d.due_date}</div>
                      </div>
                      <span style={{ fontSize: "0.72rem", padding: "3px 10px", borderRadius: 4, background: sc.bg, color: sc.color, fontWeight: 500, textTransform: "capitalize" }}>
                        {d.status}
                      </span>
                    </div>
                  );
                })}
              </div>
            )}

            <div style={{ marginTop: 20, padding: 16, background: `${ACCENT}10`, border: `1px solid ${ACCENT}30`, borderRadius: 8 }}>
              <h4 style={{ fontSize: "0.82rem", fontWeight: 600, color: ACCENT, marginBottom: 8, display: "flex", alignItems: "center", gap: 6 }}>
                <Info size={14} /> Nigerian Tax Filing Reminders
              </h4>
              <ul style={{ fontSize: "0.78rem", color: TEXT_SEC, lineHeight: 1.8, paddingLeft: 16 }}>
                <li>VAT & WHT returns: 21st of the month following the tax period</li>
                <li>CIT returns: Within 6 months after the end of the accounting year</li>
                <li>Education Tax: Filed alongside CIT returns</li>
                <li>Penalty for late filing: ₦25,000 for first month, ₦5,000 per additional month</li>
              </ul>
            </div>
          </div>
        )}

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
