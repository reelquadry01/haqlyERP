"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet, apiPost } from "@/lib/api";
import { FileSpreadsheet, Settings, Send, CheckCircle, XCircle, Clock, AlertTriangle } from "lucide-react";

interface EinvoiceProfile {
  id: string;
  tin: string;
  legal_name: string;
  trade_name: string;
  is_complete: boolean;
}

interface EinvoiceCredential {
  id: string;
  environment: string;
  is_active: boolean;
  last_tested_at: string | null;
}

interface EinvoiceDocument {
  id: string;
  irn: string;
  invoice_number: string;
  invoice_category: string;
  status: string;
  firs_submitted_at: string | null;
  firs_confirmed_at: string | null;
  error_message: string | null;
  created_at: string;
}

const TABS = ["profile", "credentials", "documents", "submissions"] as const;
type Tab = (typeof TABS)[number];

export default function EinvoicingPage() {
  const token = getToken();
  const [activeTab, setActiveTab] = useState<Tab>("documents");
  const [profile, setProfile] = useState<EinvoiceProfile | null>(null);
  const [credentials, setCredentials] = useState<EinvoiceCredential | null>(null);
  const [documents, setDocuments] = useState<EinvoiceDocument[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const [profRes, credRes, docRes] = await Promise.all([
          apiGet("/einvoicing/profile", token),
          apiGet("/einvoicing/credentials", token),
          apiGet("/einvoicing/documents", token),
        ]);
        if (profRes.ok) { const d = await profRes.json(); setProfile(d.data || d); }
        if (credRes.ok) { const d = await credRes.json(); setCredentials(d.data || d); }
        if (docRes.ok) setDocuments((await docRes.json()).data || []);
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  const docColumns: Column<EinvoiceDocument>[] = [
    { key: "invoice_number", label: "Invoice #", sortable: true, width: "140px" },
    { key: "irn", label: "IRN", width: "120px" },
    { key: "invoice_category", label: "Category", width: "80px" },
    {
      key: "status",
      label: "Status",
      width: "130px",
      render: (v: string) => {
        const map: Record<string, { bg: string; color: string; icon: typeof CheckCircle }> = {
          LOCAL_ONLY: { bg: "rgba(134,142,150,0.12)", color: "#868E96", icon: Clock },
          PENDING_VALIDATION: { bg: "rgba(13,202,240,0.12)", color: "#0DCAF0", icon: Clock },
          VALIDATED: { bg: "rgba(255,193,7,0.12)", color: "#B8860B", icon: CheckCircle },
          SIGNED: { bg: "rgba(25,135,84,0.12)", color: "#198754", icon: CheckCircle },
          CONFIRMED: { bg: "rgba(27,67,50,0.12)", color: "#1B4332", icon: CheckCircle },
          REJECTED: { bg: "rgba(220,53,69,0.12)", color: "#DC3545", icon: XCircle },
          ERROR: { bg: "rgba(220,53,69,0.12)", color: "#DC3545", icon: AlertTriangle },
        };
        const c = map[v] || map.LOCAL_ONLY;
        const Icon = c.icon;
        return <span style={{ display: "flex", alignItems: "center", gap: 4, fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: c.bg, color: c.color }}><Icon size={12} />{v.replace(/_/g, " ")}</span>;
      },
    },
    { key: "firs_submitted_at", label: "Submitted", width: "120px" },
    { key: "firs_confirmed_at", label: "Confirmed", width: "120px" },
    { key: "error_message", label: "Error", render: (v: string) => v ? <span style={{ color: "#DC3545", fontSize: "0.8rem" }}>{v}</span> : "-" },
  ];

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <FileSpreadsheet size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> E-Invoicing (FIRS/NRS)
          </h1>
          <button className="btn btn-primary" style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <Send size={16} /> Submit to FIRS
          </button>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: "2px solid #E9ECEF" }}>
          {TABS.map((t) => (
            <button key={t} onClick={() => setActiveTab(t)} style={{ padding: "10px 16px", fontSize: "0.85rem", fontWeight: activeTab === t ? 600 : 400, color: activeTab === t ? "#1B4332" : "#868E96", borderBottom: activeTab === t ? "2px solid #1B4332" : "2px solid transparent", marginBottom: -2, textTransform: "capitalize" }}>
              {t}
            </button>
          ))}
        </div>

        {loading ? <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
          : activeTab === "profile" ? (
            <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)", maxWidth: 600 }}>
              {profile ? (
                <div>
                  <div style={{ marginBottom: 16 }}>
                    <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#868E96", marginBottom: 2 }}>TIN</label>
                    <span style={{ fontSize: "0.95rem", color: "#1A1A2E" }}>{profile.tin}</span>
                  </div>
                  <div style={{ marginBottom: 16 }}>
                    <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#868E96", marginBottom: 2 }}>Legal Name</label>
                    <span style={{ fontSize: "0.95rem", color: "#1A1A2E" }}>{profile.legal_name}</span>
                  </div>
                  <div style={{ marginBottom: 16 }}>
                    <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#868E96", marginBottom: 2 }}>Trade Name</label>
                    <span style={{ fontSize: "0.95rem", color: "#1A1A2E" }}>{profile.trade_name || "-"}</span>
                  </div>
                  <div>
                    <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#868E96", marginBottom: 2 }}>Setup Status</label>
                    <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: profile.is_complete ? "rgba(25,135,84,0.12)" : "rgba(255,193,7,0.12)", color: profile.is_complete ? "#198754" : "#B8860B" }}>
                      {profile.is_complete ? "Complete" : "Incomplete"}
                    </span>
                  </div>
                </div>
              ) : (
                <div style={{ textAlign: "center", padding: 32, color: "#868E96" }}>
                  <Settings size={32} style={{ marginBottom: 8 }} />
                  <p>No e-invoicing profile configured</p>
                  <button className="btn btn-primary" style={{ marginTop: 12 }}>Setup Profile</button>
                </div>
              )}
            </div>
          ) : activeTab === "credentials" ? (
            <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)", maxWidth: 600 }}>
              {credentials ? (
                <div>
                  <div style={{ marginBottom: 16 }}>
                    <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#868E96", marginBottom: 2 }}>Environment</label>
                    <span style={{ fontSize: "0.95rem", color: credentials.environment === "PRODUCTION" ? "#DC3545" : "#0DCAF0" }}>{credentials.environment}</span>
                  </div>
                  <div style={{ marginBottom: 16 }}>
                    <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#868E96", marginBottom: 2 }}>Status</label>
                    <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: credentials.is_active ? "rgba(25,135,84,0.12)" : "rgba(220,53,69,0.12)", color: credentials.is_active ? "#198754" : "#DC3545" }}>
                      {credentials.is_active ? "Active" : "Inactive"}
                    </span>
                  </div>
                  <button className="btn btn-secondary">Test Connection</button>
                </div>
              ) : (
                <div style={{ textAlign: "center", padding: 32, color: "#868E96" }}>
                  <p>No credentials configured</p>
                  <button className="btn btn-primary" style={{ marginTop: 12 }}>Configure Credentials</button>
                </div>
              )}
            </div>
          ) : (
            <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
              <DataTable columns={docColumns} data={documents} pageSize={15} emptyMessage="No e-invoice documents" />
            </div>
          )}
      </div>
    </WorkspaceShell>
  );
}
