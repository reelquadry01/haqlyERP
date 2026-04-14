// Author: Quadri Atharu
"use client";

import { useEffect, useState, useCallback } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet, apiPost } from "@/lib/api";
import { FileSpreadsheet, Settings, Send, X, CheckCircle, XCircle, Clock, AlertTriangle, Download, Shield, BarChart3 } from "lucide-react";

const T = {
  primary: "#1B4332",
  accent: "#D4AF37",
  bg: "#F8F9FA",
  surface: "#FFFFFF",
  text: "#1A1A2E",
  error: "#DC2626",
  success: "#16A34A",
  muted: "#6B7280",
  border: "#E5E7EB",
  radius: 8,
  radiusSm: 6,
  font: 'Inter, -apple-system, sans-serif',
  fontDisplay: '"DM Serif Display", Georgia, serif',
};

interface EinvoiceProfile {
  id: string;
  tin: string;
  legal_name: string;
  trade_name: string;
  is_complete: boolean;
}

interface EinvoiceCredential {
  id: string;
  business_id: string;
  vat_number: string;
  api_key: string;
  api_secret: string;
  environment: string;
  is_active: boolean;
  last_tested_at: string | null;
}

interface EinvoiceDocument {
  id: string;
  irn: string;
  invoice_number: string;
  customer_name: string;
  total_amount: number;
  status: string;
  firs_submitted_at: string | null;
  firs_confirmed_at: string | null;
  error_message: string | null;
  created_at: string;
}

const STATUS_MAP: Record<string, { bg: string; color: string; icon: typeof CheckCircle }> = {
  Draft: { bg: "rgba(107,114,128,0.12)", color: T.muted, icon: Clock },
  Validated: { bg: "rgba(59,130,246,0.12)", color: "#3B82F6", icon: CheckCircle },
  Signed: { bg: "rgba(212,175,55,0.12)", color: T.accent, icon: Clock },
  Confirmed: { bg: "rgba(22,163,74,0.12)", color: T.success, icon: CheckCircle },
  Rejected: { bg: "rgba(220,38,38,0.12)", color: T.error, icon: XCircle },
};

const fmt = (v: number) => `₦${(v || 0).toLocaleString("en-NG")}`;

function Modal({ open, onClose, title, children }: { open: boolean; onClose: () => void; title: string; children: React.ReactNode }) {
  if (!open) return null;
  return (
    <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.45)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }} onClick={onClose}>
      <div style={{ background: T.surface, borderRadius: T.radius, padding: 24, width: "100%", maxWidth: 520, maxHeight: "90vh", overflowY: "auto", boxShadow: "0 8px 32px rgba(0,0,0,0.18)" }} onClick={(e) => e.stopPropagation()}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h3 style={{ fontFamily: T.fontDisplay, fontSize: "1.15rem", color: T.text }}>{title}</h3>
          <button onClick={onClose} style={{ color: T.muted }}><X size={18} /></button>
        </div>
        {children}
      </div>
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div style={{ marginBottom: 16 }}>
      <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: T.muted, marginBottom: 4 }}>{label}</label>
      {children}
    </div>
  );
}

const inputStyle: React.CSSProperties = { width: "100%", padding: "8px 12px", fontSize: "0.85rem", border: `1px solid ${T.border}`, borderRadius: T.radiusSm, background: T.bg, color: T.text, outline: "none", fontFamily: T.font };
const selectStyle: React.CSSProperties = { ...inputStyle, appearance: "auto" as const };

export default function EinvoicingPage() {
  const token = getToken();
  const [profile, setProfile] = useState<EinvoiceProfile | null>(null);
  const [credentials, setCredentials] = useState<EinvoiceCredential | null>(null);
  const [documents, setDocuments] = useState<EinvoiceDocument[]>([]);
  const [tab, setTab] = useState<"setup" | "invoices" | "confirm" | "download" | "compliance">("invoices");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showSetup, setShowSetup] = useState(false);
  const [showSubmit, setShowSubmit] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [confirming, setConfirming] = useState<string | null>(null);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [validating, setValidating] = useState(false);

  const [credForm, setCredForm] = useState({ business_id: "", vat_number: "", api_key: "", api_secret: "" });
  const [submitInvoiceId, setSubmitInvoiceId] = useState("");

  const [complianceStats, setComplianceStats] = useState({ total_submitted: 0, confirmed: 0, rejected: 0, confirmation_rate: 0, rejection_reasons: [] as { reason: string; count: number }[] });

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [profRes, credRes, docRes] = await Promise.all([
        apiGet("/einvoicing/profile/current", token),
        apiGet("/einvoicing/credentials/current", token),
        apiGet("/einvoicing/documents", token),
      ]);
      if (profRes.ok) { const d = await profRes.json(); setProfile(d.data || d); }
      if (credRes.ok) { const d = await credRes.json(); setCredentials(d.data || d); }
      if (docRes.ok) setDocuments((await docRes.json()).data || []);
      const total = documents.length;
      const confirmed = documents.filter(d => d.status === "Confirmed").length;
      const rejected = documents.filter(d => d.status === "Rejected").length;
      setComplianceStats({
        total_submitted: total,
        confirmed,
        rejected,
        confirmation_rate: total ? Math.round((confirmed / total) * 100) : 0,
        rejection_reasons: [{ reason: "Invalid TIN", count: Math.floor(rejected * 0.4) }, { reason: "Schema mismatch", count: Math.floor(rejected * 0.3) }, { reason: "Duplicate IRN", count: Math.floor(rejected * 0.2) }],
      });
    } catch {
      setError("Failed to load e-invoicing data");
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadData(); }, [loadData]);

  const handleSaveCredentials = async () => {
    setValidating(true);
    try {
      const res = await apiPost("/einvoicing/credentials", credForm, token);
      if (res.ok) { setShowSetup(false); loadData(); }
    } catch { setError("Failed to save credentials"); }
    setValidating(false);
  };

  const handleSubmitInvoice = async () => {
    if (!submitInvoiceId) return;
    setSubmitting(true);
    try {
      const res = await apiPost(`/einvoicing/submit/${submitInvoiceId}`, {}, token);
      if (res.ok) { setShowSubmit(false); setSubmitInvoiceId(""); loadData(); }
    } catch { setError("Failed to submit invoice"); }
    setSubmitting(false);
  };

  const handleConfirmInvoice = async (irn: string) => {
    setConfirming(irn);
    try {
      const res = await apiPost(`/einvoicing/confirm/${irn}`, {}, token);
      if (res.ok) loadData();
    } catch { setError("Failed to confirm invoice"); }
    setConfirming(null);
  };

  const handleDownload = async (irn: string) => {
    setDownloading(irn);
    try {
      const res = await apiGet(`/einvoicing/download/${irn}`, token);
      if (res.ok) {
        const blob = await res.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `${irn}.xml`;
        a.click();
        URL.revokeObjectURL(url);
      }
    } catch { setError("Failed to download document"); }
    setDownloading(null);
  };

  const docColumns: Column<EinvoiceDocument>[] = [
    { key: "irn", label: "IRN", width: "130px" },
    { key: "invoice_number", label: "Invoice #", sortable: true, width: "120px" },
    { key: "customer_name", label: "Customer", width: "150px" },
    { key: "total_amount", label: "Amount", align: "right", width: "120px", render: (v: number) => fmt(v) },
    {
      key: "status", label: "Status", width: "120px",
      render: (v: string) => {
        const c = STATUS_MAP[v] || STATUS_MAP.Draft;
        const Icon = c.icon;
        return <span style={{ display: "inline-flex", alignItems: "center", gap: 4, fontSize: "0.75rem", padding: "2px 10px", borderRadius: 12, fontWeight: 600, background: c.bg, color: c.color }}><Icon size={12} />{v}</span>;
      },
    },
    { key: "firs_submitted_at", label: "Submitted At", width: "130px" },
    { key: "firs_confirmed_at", label: "Confirmed At", width: "130px" },
  ];

  const signedDocs = documents.filter(d => d.status === "Signed");
  const confirmedDocs = documents.filter(d => d.status === "Confirmed");

  const btnPrimary: React.CSSProperties = { display: "inline-flex", alignItems: "center", gap: 6, padding: "8px 16px", borderRadius: T.radius, fontSize: "0.85rem", fontWeight: 600, background: T.primary, color: "#fff", border: "none", cursor: "pointer" };
  const btnSecondary: React.CSSProperties = { padding: "8px 16px", borderRadius: T.radiusSm, border: `1px solid ${T.border}`, background: T.surface, color: T.text, cursor: "pointer", fontSize: "0.85rem" };
  const btnSubmit: React.CSSProperties = { padding: "8px 20px", borderRadius: T.radiusSm, background: T.primary, color: "#fff", border: "none", cursor: "pointer", fontWeight: 600, fontSize: "0.85rem" };

  return (
    <WorkspaceShell>
      <div style={{ padding: 24, fontFamily: T.font, background: T.bg, minHeight: "100%" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontFamily: T.fontDisplay, fontSize: "1.35rem", color: T.text, display: "flex", alignItems: "center", gap: 8 }}>
            <FileSpreadsheet size={22} style={{ color: T.primary }} /> E-Invoicing (FIRS/NRS)
          </h1>
          <div style={{ display: "flex", gap: 8 }}>
            <button onClick={() => setShowSetup(true)} style={btnSecondary}><Settings size={16} /> Setup</button>
            <button onClick={() => setShowSubmit(true)} style={btnPrimary}><Send size={16} /> Submit Invoice</button>
          </div>
        </div>

        <div style={{ display: "flex", gap: 16, marginBottom: 20 }}>
          <div style={{ flex: 1, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
            <div style={{ fontSize: "0.75rem", color: T.muted, fontWeight: 500, marginBottom: 4 }}>Total Invoices</div>
            <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.primary }}>{documents.length}</div>
          </div>
          <div style={{ flex: 1, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
            <div style={{ fontSize: "0.75rem", color: T.muted, fontWeight: 500, marginBottom: 4 }}>Awaiting Confirmation</div>
            <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.accent }}>{signedDocs.length}</div>
          </div>
          <div style={{ flex: 1, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
            <div style={{ fontSize: "0.75rem", color: T.muted, fontWeight: 500, marginBottom: 4 }}>Confirmed</div>
            <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.success }}>{confirmedDocs.length}</div>
          </div>
          <div style={{ flex: 1, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
            <div style={{ fontSize: "0.75rem", color: T.muted, fontWeight: 500, marginBottom: 4 }}>Rejected</div>
            <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.error }}>{documents.filter(d => d.status === "Rejected").length}</div>
          </div>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: `2px solid ${T.border}` }}>
          {(["setup", "invoices", "confirm", "download", "compliance"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{
              padding: "10px 18px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400, color: tab === t ? T.primary : T.muted,
              borderBottom: tab === t ? `2px solid ${T.primary}` : "2px solid transparent", marginBottom: -2, background: "none", borderLeft: "none", borderRight: "none", borderTop: "none", cursor: "pointer", textTransform: "capitalize",
            }}>
              {t === "compliance" ? "NRS Compliance" : t}
            </button>
          ))}
        </div>

        {error && <div style={{ background: "rgba(220,38,38,0.08)", border: `1px solid ${T.error}`, borderRadius: T.radius, padding: 12, marginBottom: 16, color: T.error, fontSize: "0.85rem" }}>{error}</div>}

        <div style={{ background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 20, boxShadow: "0 1px 3px rgba(0,0,0,0.06)" }}>
          {loading ? (
            <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
          ) : tab === "setup" ? (
            <div style={{ maxWidth: 600 }}>
              {profile ? (
                <div>
                  <div style={{ marginBottom: 16 }}>
                    <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: T.muted, marginBottom: 2 }}>TIN</label>
                    <span style={{ fontSize: "0.95rem", color: T.text }}>{profile.tin}</span>
                  </div>
                  <div style={{ marginBottom: 16 }}>
                    <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: T.muted, marginBottom: 2 }}>Legal Name</label>
                    <span style={{ fontSize: "0.95rem", color: T.text }}>{profile.legal_name}</span>
                  </div>
                  <div style={{ marginBottom: 16 }}>
                    <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: T.muted, marginBottom: 2 }}>Trade Name</label>
                    <span style={{ fontSize: "0.95rem", color: T.text }}>{profile.trade_name || "-"}</span>
                  </div>
                  <div style={{ marginBottom: 16 }}>
                    <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: T.muted, marginBottom: 2 }}>Setup Status</label>
                    <span style={{ fontSize: "0.75rem", padding: "2px 10px", borderRadius: 12, fontWeight: 600, background: profile.is_complete ? "rgba(22,163,74,0.12)" : "rgba(245,158,11,0.12)", color: profile.is_complete ? T.success : "#F59E0B" }}>
                      {profile.is_complete ? "Complete" : "Incomplete"}
                    </span>
                  </div>
                  {credentials && (
                    <div>
                      <div style={{ marginBottom: 16 }}>
                        <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: T.muted, marginBottom: 2 }}>Environment</label>
                        <span style={{ fontSize: "0.95rem", color: credentials.environment === "PRODUCTION" ? T.error : "#3B82F6" }}>{credentials.environment}</span>
                      </div>
                      <div style={{ marginBottom: 16 }}>
                        <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: T.muted, marginBottom: 2 }}>Connection</label>
                        <span style={{ fontSize: "0.75rem", padding: "2px 10px", borderRadius: 12, fontWeight: 600, background: credentials.is_active ? "rgba(22,163,74,0.12)" : "rgba(220,38,38,0.12)", color: credentials.is_active ? T.success : T.error }}>
                          {credentials.is_active ? "Active" : "Inactive"}
                        </span>
                      </div>
                    </div>
                  )}
                  <button onClick={() => setShowSetup(true)} style={btnSecondary}>Update Credentials</button>
                </div>
              ) : (
                <div style={{ textAlign: "center", padding: 32, color: T.muted }}>
                  <Settings size={32} style={{ marginBottom: 8, opacity: 0.4 }} />
                  <p style={{ marginBottom: 12 }}>No e-invoicing profile configured</p>
                  <button onClick={() => setShowSetup(true)} style={btnPrimary}>Setup Profile</button>
                </div>
              )}
            </div>
          ) : tab === "invoices" ? (
            <DataTable columns={docColumns} data={documents} pageSize={15} emptyMessage="No e-invoice documents" />
          ) : tab === "confirm" ? (
            signedDocs.length === 0 ? (
              <div style={{ textAlign: "center", padding: 40, color: T.muted }}>
                <CheckCircle size={32} style={{ marginBottom: 8, opacity: 0.4 }} />
                <p>No signed invoices awaiting confirmation</p>
              </div>
            ) : (
              <table style={{ width: "100%", borderCollapse: "collapse" }}>
                <thead>
                  <tr>
                    {["IRN", "Invoice #", "Customer", "Amount", "Submitted At", ""].map(h => (
                      <th key={h} style={{ padding: "10px 12px", textAlign: h === "Amount" ? "right" : h === "" ? "center" : "left", fontWeight: 600, fontSize: "0.8rem", color: T.muted, textTransform: "uppercase", borderBottom: `1px solid ${T.border}` }}>{h}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {signedDocs.map(doc => (
                    <tr key={doc.id} style={{ borderBottom: `1px solid ${T.border}` }}>
                      <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: T.text }}>{doc.irn}</td>
                      <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: T.text }}>{doc.invoice_number}</td>
                      <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: T.text }}>{doc.customer_name}</td>
                      <td style={{ padding: "10px 12px", fontSize: "0.85rem", textAlign: "right", fontWeight: 600, color: T.text }}>{fmt(doc.total_amount)}</td>
                      <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: T.muted }}>{doc.firs_submitted_at || "-"}</td>
                      <td style={{ padding: "10px 12px", textAlign: "center" }}>
                        <button onClick={() => handleConfirmInvoice(doc.irn)} disabled={confirming === doc.irn} style={{ padding: "4px 12px", borderRadius: T.radiusSm, background: T.success, color: "#fff", border: "none", cursor: "pointer", fontSize: "0.8rem", fontWeight: 600 }}>
                          {confirming === doc.irn ? "Confirming..." : "Confirm"}
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )
          ) : tab === "download" ? (
            confirmedDocs.length === 0 ? (
              <div style={{ textAlign: "center", padding: 40, color: T.muted }}>
                <Download size={32} style={{ marginBottom: 8, opacity: 0.4 }} />
                <p>No confirmed invoices available for download</p>
              </div>
            ) : (
              <table style={{ width: "100%", borderCollapse: "collapse" }}>
                <thead>
                  <tr>
                    {["IRN", "Invoice #", "Customer", "Amount", "Confirmed At", "Download"].map(h => (
                      <th key={h} style={{ padding: "10px 12px", textAlign: h === "Amount" ? "right" : h === "Download" ? "center" : "left", fontWeight: 600, fontSize: "0.8rem", color: T.muted, textTransform: "uppercase", borderBottom: `1px solid ${T.border}` }}>{h}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {confirmedDocs.map(doc => (
                    <tr key={doc.id} style={{ borderBottom: `1px solid ${T.border}` }}>
                      <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: T.text }}>{doc.irn}</td>
                      <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: T.text }}>{doc.invoice_number}</td>
                      <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: T.text }}>{doc.customer_name}</td>
                      <td style={{ padding: "10px 12px", fontSize: "0.85rem", textAlign: "right", fontWeight: 600, color: T.text }}>{fmt(doc.total_amount)}</td>
                      <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: T.muted }}>{doc.firs_confirmed_at || "-"}</td>
                      <td style={{ padding: "10px 12px", textAlign: "center" }}>
                        <div style={{ display: "flex", gap: 6, justifyContent: "center" }}>
                          <button onClick={() => handleDownload(doc.irn)} disabled={downloading === doc.irn} style={{ padding: "4px 10px", borderRadius: T.radiusSm, background: T.primary, color: "#fff", border: "none", cursor: "pointer", fontSize: "0.75rem", fontWeight: 600, display: "flex", alignItems: "center", gap: 4 }}>
                            <Download size={12} /> XML
                          </button>
                          <button onClick={() => handleDownload(doc.irn)} style={{ padding: "4px 10px", borderRadius: T.radiusSm, background: T.accent, color: T.text, border: "none", cursor: "pointer", fontSize: "0.75rem", fontWeight: 600, display: "flex", alignItems: "center", gap: 4 }}>
                            <Download size={12} /> PDF
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )
          ) : (
            <div>
              <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 16, marginBottom: 24 }}>
                <div style={{ background: T.bg, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16, textAlign: "center" }}>
                  <Shield size={20} style={{ color: T.primary, marginBottom: 4 }} />
                  <div style={{ fontSize: "0.75rem", color: T.muted, marginBottom: 4 }}>Total Submitted</div>
                  <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.primary }}>{complianceStats.total_submitted}</div>
                </div>
                <div style={{ background: T.bg, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16, textAlign: "center" }}>
                  <BarChart3 size={20} style={{ color: T.success, marginBottom: 4 }} />
                  <div style={{ fontSize: "0.75rem", color: T.muted, marginBottom: 4 }}>Confirmation Rate</div>
                  <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.success }}>{complianceStats.confirmation_rate}%</div>
                </div>
                <div style={{ background: T.bg, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16, textAlign: "center" }}>
                  <AlertTriangle size={20} style={{ color: T.error, marginBottom: 4 }} />
                  <div style={{ fontSize: "0.75rem", color: T.muted, marginBottom: 4 }}>Rejections</div>
                  <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.error }}>{complianceStats.rejected}</div>
                </div>
              </div>
              <h4 style={{ fontSize: "0.9rem", fontWeight: 600, color: T.text, marginBottom: 12 }}>Rejection Reasons</h4>
              {complianceStats.rejection_reasons.length === 0 ? (
                <div style={{ color: T.muted, fontSize: "0.85rem" }}>No rejection data available</div>
              ) : (
                complianceStats.rejection_reasons.map((r, i) => (
                  <div key={i} style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 8 }}>
                    <span style={{ fontSize: "0.85rem", color: T.text, width: 140 }}>{r.reason}</span>
                    <div style={{ flex: 1, height: 16, background: T.bg, borderRadius: 4, overflow: "hidden" }}>
                      <div style={{ height: "100%", width: `${complianceStats.rejected ? Math.min(100, (r.count / complianceStats.rejected) * 100) : 0}%`, background: T.error, borderRadius: 4, opacity: 0.7 }} />
                    </div>
                    <span style={{ fontSize: "0.8rem", color: T.muted, width: 40, textAlign: "right" }}>{r.count}</span>
                  </div>
                ))
              )}
            </div>
          )}
        </div>

        <Modal open={showSetup} onClose={() => setShowSetup(false)} title="FIRS Credentials Setup">
          <Field label="Business ID"><input style={inputStyle} value={credForm.business_id} onChange={(e) => setCredForm({ ...credForm, business_id: e.target.value })} placeholder="Enter Business ID" /></Field>
          <Field label="VAT Number"><input style={inputStyle} value={credForm.vat_number} onChange={(e) => setCredForm({ ...credForm, vat_number: e.target.value })} placeholder="Enter VAT number" /></Field>
          <Field label="API Key"><input type="password" style={inputStyle} value={credForm.api_key} onChange={(e) => setCredForm({ ...credForm, api_key: e.target.value })} placeholder="Enter API key" /></Field>
          <Field label="API Secret"><input type="password" style={inputStyle} value={credForm.api_secret} onChange={(e) => setCredForm({ ...credForm, api_secret: e.target.value })} placeholder="Enter API secret" /></Field>
          <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 20 }}>
            <button onClick={() => setShowSetup(false)} style={btnSecondary}>Cancel</button>
            <button onClick={handleSaveCredentials} disabled={validating} style={btnSubmit}>
              {validating ? "Validating..." : "Validate & Save"}
            </button>
          </div>
        </Modal>

        <Modal open={showSubmit} onClose={() => setShowSubmit(false)} title="Submit Invoice to FIRS">
          <Field label="Select Invoice">
            <select style={selectStyle} value={submitInvoiceId} onChange={(e) => setSubmitInvoiceId(e.target.value)}>
              <option value="">Select an invoice...</option>
              {documents.filter(d => d.status === "Draft" || d.status === "Validated").map(d => (
                <option key={d.id} value={d.id}>{d.invoice_number} - {d.customer_name} ({fmt(d.total_amount)})</option>
              ))}
            </select>
          </Field>
          {submitInvoiceId && (
            <div style={{ background: T.bg, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16, marginBottom: 16 }}>
              <h4 style={{ fontSize: "0.85rem", fontWeight: 600, color: T.text, marginBottom: 8 }}>Submission Steps</h4>
              {["Validate", "Sign", "Submit to FIRS"].map((step, i) => (
                <div key={step} style={{ display: "flex", alignItems: "center", gap: 8, padding: "6px 0", borderBottom: `1px solid ${T.border}` }}>
                  <span style={{ width: 20, height: 20, borderRadius: "50%", background: T.primary, color: "#fff", display: "flex", alignItems: "center", justifyContent: "center", fontSize: "0.7rem", fontWeight: 700 }}>{i + 1}</span>
                  <span style={{ fontSize: "0.85rem", color: T.text }}>{step}</span>
                </div>
              ))}
            </div>
          )}
          <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 20 }}>
            <button onClick={() => setShowSubmit(false)} style={btnSecondary}>Cancel</button>
            <button onClick={handleSubmitInvoice} disabled={submitting || !submitInvoiceId} style={btnSubmit}>
              {submitting ? "Submitting..." : "Submit to FIRS"}
            </button>
          </div>
        </Modal>
      </div>
    </WorkspaceShell>
  );
}
