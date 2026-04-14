// Author: Quadri Atharu

"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { getToken, saveCompanyContext, getCompanyContext } from "@/lib/session";
import { apiGet, apiPost, apiPatch } from "@/lib/api";
import {
  Building2,
  Plus,
  X,
  ChevronRight,
  MapPin,
  Users,
  Settings,
  Briefcase,
  Globe,
  Calendar,
  Landmark,
  Loader2,
  AlertCircle,
  Check,
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
  fontUi: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  fontHeading: '"DM Serif Display", Georgia, serif',
};

const INDUSTRIES = [
  "Oil & Gas", "Banking & Finance", "Telecommunications", "Manufacturing",
  "Agriculture", "Real Estate", "Construction", "Healthcare",
  "Education", "Transportation", "Retail & Wholesale", "Mining",
  "Insurance", "Legal Services", "Technology",
];

const CURRENCIES = ["NGN", "USD", "GBP", "EUR"];
const MONTHS = ["January","February","March","April","May","June","July","August","September","October","November","December"];
const TAX_AUTHORITIES = ["FIRS", "State IRS"];

interface Company {
  id: string;
  name: string;
  rc_number: string;
  tin: string;
  industry: string;
  address: string;
  base_currency: string;
  fiscal_year_end: number;
  tax_authority: string;
  is_active: boolean;
  vat_registered: boolean;
  wht_agent: boolean;
  einvoicing_enabled: boolean;
}

interface Department {
  id: string;
  name: string;
  company_id: string;
  head_name: string;
  is_active: boolean;
}

interface Branch {
  id: string;
  name: string;
  company_id: string;
  city: string;
  address: string;
  is_active: boolean;
}

type DetailTab = "settings" | "departments" | "branches" | "business";

const emptyCompany: Partial<Company> = {
  name: "", rc_number: "", tin: "", industry: "", address: "",
  base_currency: "NGN", fiscal_year_end: 12, tax_authority: "FIRS",
  vat_registered: false, wht_agent: false, einvoicing_enabled: false,
};

export default function OrganizationsPage() {
  const token = getToken();
  const [companies, setCompanies] = useState<Company[]>([]);
  const [departments, setDepartments] = useState<Department[]>([]);
  const [branches, setBranches] = useState<Branch[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);
  const [selectedCompany, setSelectedCompany] = useState<Company | null>(null);
  const [detailTab, setDetailTab] = useState<DetailTab>("settings");
  const [addForm, setAddForm] = useState<Partial<Company>>({ ...emptyCompany });
  const [saving, setSaving] = useState(false);
  const [switchingCompany, setSwitchingCompany] = useState(false);

  useEffect(() => {
    async function load() {
      try {
        const [cRes, dRes, bRes] = await Promise.all([
          apiGet("/org/companies", token),
          apiGet("/org/departments", token),
          apiGet("/org/branches", token),
        ]);
        if (cRes.ok) setCompanies((await cRes.json()).data || []);
        if (dRes.ok) setDepartments((await dRes.json()).data || []);
        if (bRes.ok) setBranches((await bRes.json()).data || []);
      } catch {
        setError("Failed to load organization data");
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  async function handleAddCompany(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    try {
      const res = await apiPost("/org/companies", addForm, token);
      if (res.ok) {
        const newCo = (await res.json()).data;
        setCompanies((prev) => [...prev, newCo]);
        setShowAddModal(false);
        setAddForm({ ...emptyCompany });
      } else {
        setError("Failed to create company");
      }
    } catch {
      setError("Network error");
    } finally {
      setSaving(false);
    }
  }

  async function handleSwitchCompany(companyId: string) {
    setSwitchingCompany(true);
    saveCompanyContext(companyId);
    setTimeout(() => {
      setSwitchingCompany(false);
      window.location.reload();
    }, 500);
  }

  const activeCompanyId = getCompanyContext();
  const companyDepts = selectedCompany ? departments.filter((d) => d.company_id === selectedCompany.id) : [];
  const companyBranches = selectedCompany ? branches.filter((b) => b.company_id === selectedCompany.id) : [];

  const cardStyle: React.CSSProperties = {
    background: TOKENS.surface,
    border: `1px solid ${TOKENS.border}`,
    borderRadius: TOKENS.radiusMd,
    padding: 20,
    boxShadow: TOKENS.shadowSm,
    cursor: "pointer",
    transition: "all 150ms ease",
  };

  const inputStyle: React.CSSProperties = {
    width: "100%",
    background: TOKENS.bg,
    border: `1px solid ${TOKENS.border}`,
    borderRadius: TOKENS.radiusSm,
    padding: "8px 12px",
    fontSize: "0.85rem",
    color: TOKENS.text,
    fontFamily: TOKENS.fontUi,
  };

  const selectStyle: React.CSSProperties = { ...inputStyle, minWidth: "100%" };

  const detailTabStyle = (isActive: boolean): React.CSSProperties => ({
    padding: "8px 16px",
    fontSize: "0.85rem",
    fontWeight: isActive ? 600 : 400,
    color: isActive ? TOKENS.primary : TOKENS.textTertiary,
    borderBottom: isActive ? `2px solid ${TOKENS.primary}` : "2px solid transparent",
    marginBottom: -2,
    cursor: "pointer",
    transition: "all 150ms ease",
  });

  if (loading) {
    return (
      <WorkspaceShell>
        <div style={{ display: "flex", justifyContent: "center", alignItems: "center", height: "50vh" }}>
          <Loader2 size={24} style={{ animation: "spin 0.8s linear infinite", color: TOKENS.primary }} />
        </div>
      </WorkspaceShell>
    );
  }

  return (
    <WorkspaceShell>
      <div style={{ padding: 24, minHeight: "100%" }}>
        {error && (
          <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "10px 14px", marginBottom: 16, background: TOKENS.errorLight, border: `1px solid ${TOKENS.error}`, borderRadius: TOKENS.radiusSm, color: TOKENS.error, fontSize: "0.85rem" }}>
            <AlertCircle size={16} />
            {error}
          </div>
        )}

        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#f0f4f8", display: "flex", alignItems: "center", gap: 8 }}>
            <Building2 size={20} /> Organizations
          </h1>
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <select
              value={activeCompanyId || ""}
              onChange={(e) => handleSwitchCompany(e.target.value)}
              style={{ background: "#1a2234", border: "1px solid #2a3754", borderRadius: 6, padding: "6px 10px", fontSize: "0.85rem", color: "#f0f4f8", cursor: "pointer" }}
            >
              <option value="">Switch Company</option>
              {companies.map((c) => (
                <option key={c.id} value={c.id}>{c.name}</option>
              ))}
            </select>
            {switchingCompany && <Loader2 size={14} style={{ animation: "spin 0.8s linear infinite", color: TOKENS.accent }} />}
            <button
              onClick={() => setShowAddModal(true)}
              style={{ display: "flex", alignItems: "center", gap: 6, padding: "8px 16px", borderRadius: TOKENS.radiusSm, fontSize: "0.85rem", fontWeight: 600, background: TOKENS.primary, color: "#FFFFFF", border: "none", cursor: "pointer" }}
            >
              <Plus size={16} /> Add Company
            </button>
          </div>
        </div>

        {selectedCompany ? (
          <div>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 16 }}>
              <button onClick={() => setSelectedCompany(null)} style={{ display: "flex", alignItems: "center", gap: 4, fontSize: "0.85rem", color: TOKENS.textTertiary, cursor: "pointer" }}>
                Companies <ChevronRight size={14} />
              </button>
              <span style={{ fontSize: "0.85rem", fontWeight: 600, color: TOKENS.text }}>{selectedCompany.name}</span>
            </div>

            <div style={{ display: "flex", gap: 4, marginBottom: 16, borderBottom: `2px solid ${TOKENS.border}` }}>
              {(["settings", "departments", "branches", "business"] as DetailTab[]).map((t) => (
                <button key={t} onClick={() => setDetailTab(t)} style={detailTabStyle(detailTab === t)}>
                  {t.charAt(0).toUpperCase() + t.slice(1)}
                </button>
              ))}
            </div>

            <div style={{ background: TOKENS.surface, border: `1px solid ${TOKENS.border}`, borderRadius: TOKENS.radiusMd, padding: 20, boxShadow: TOKENS.shadowSm }}>
              {detailTab === "settings" && (
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>Company Name</label>
                    <div style={{ fontSize: "0.9rem", color: TOKENS.text, fontWeight: 500 }}>{selectedCompany.name}</div>
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>RC Number</label>
                    <div style={{ fontSize: "0.9rem", color: TOKENS.text, fontFamily: TOKENS.fontUi }}>{selectedCompany.rc_number}</div>
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>TIN</label>
                    <div style={{ fontSize: "0.9rem", color: TOKENS.text, fontFamily: TOKENS.fontUi }}>{selectedCompany.tin}</div>
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>Address</label>
                    <div style={{ fontSize: "0.9rem", color: TOKENS.text }}>{selectedCompany.address || "—"}</div>
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>Industry</label>
                    <div style={{ fontSize: "0.9rem", color: TOKENS.text }}>{selectedCompany.industry}</div>
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>Base Currency</label>
                    <div style={{ fontSize: "0.9rem", color: TOKENS.text, fontFamily: TOKENS.fontUi }}>{selectedCompany.base_currency}</div>
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>Fiscal Year End</label>
                    <div style={{ fontSize: "0.9rem", color: TOKENS.text }}>{MONTHS[(selectedCompany.fiscal_year_end || 12) - 1]}</div>
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>Tax Authority</label>
                    <div style={{ fontSize: "0.9rem", color: TOKENS.text }}>{selectedCompany.tax_authority}</div>
                  </div>
                </div>
              )}

              {detailTab === "departments" && (
                <div>
                  <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
                    <h3 style={{ fontSize: "0.95rem", fontWeight: 600, color: TOKENS.text }}>Departments ({companyDepts.length})</h3>
                    <button style={{ display: "flex", alignItems: "center", gap: 4, padding: "4px 12px", borderRadius: TOKENS.radiusSm, fontSize: "0.8rem", background: TOKENS.primaryLight, color: TOKENS.primary, border: `1px solid ${TOKENS.primary}`, cursor: "pointer" }}>
                      <Plus size={14} /> Add Department
                    </button>
                  </div>
                  {companyDepts.length === 0 ? (
                    <p style={{ color: TOKENS.textTertiary, textAlign: "center", padding: 20 }}>No departments configured</p>
                  ) : (
                    <table style={{ width: "100%", borderCollapse: "collapse" }}>
                      <thead>
                        <tr>
                          <th style={{ padding: "8px 12px", textAlign: "left", fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase", borderBottom: `2px solid ${TOKENS.border}` }}>Name</th>
                          <th style={{ padding: "8px 12px", textAlign: "left", fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase", borderBottom: `2px solid ${TOKENS.border}` }}>Head</th>
                          <th style={{ padding: "8px 12px", textAlign: "center", fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase", borderBottom: `2px solid ${TOKENS.border}`, width: 80 }}>Status</th>
                        </tr>
                      </thead>
                      <tbody>
                        {companyDepts.map((d) => (
                          <tr key={d.id}>
                            <td style={{ padding: "8px 12px", fontSize: "0.85rem", color: TOKENS.text, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>{d.name}</td>
                            <td style={{ padding: "8px 12px", fontSize: "0.85rem", color: TOKENS.textSecondary, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>{d.head_name || "—"}</td>
                            <td style={{ padding: "8px 12px", textAlign: "center", borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>
                              <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: d.is_active ? TOKENS.successLight : TOKENS.errorLight, color: d.is_active ? TOKENS.success : TOKENS.error }}>{d.is_active ? "Active" : "Inactive"}</span>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  )}
                </div>
              )}

              {detailTab === "branches" && (
                <div>
                  <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
                    <h3 style={{ fontSize: "0.95rem", fontWeight: 600, color: TOKENS.text }}>Branches ({companyBranches.length})</h3>
                    <button style={{ display: "flex", alignItems: "center", gap: 4, padding: "4px 12px", borderRadius: TOKENS.radiusSm, fontSize: "0.8rem", background: TOKENS.primaryLight, color: TOKENS.primary, border: `1px solid ${TOKENS.primary}`, cursor: "pointer" }}>
                      <Plus size={14} /> Add Branch
                    </button>
                  </div>
                  {companyBranches.length === 0 ? (
                    <p style={{ color: TOKENS.textTertiary, textAlign: "center", padding: 20 }}>No branches configured</p>
                  ) : (
                    <table style={{ width: "100%", borderCollapse: "collapse" }}>
                      <thead>
                        <tr>
                          <th style={{ padding: "8px 12px", textAlign: "left", fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase", borderBottom: `2px solid ${TOKENS.border}` }}>Name</th>
                          <th style={{ padding: "8px 12px", textAlign: "left", fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase", borderBottom: `2px solid ${TOKENS.border}` }}>City</th>
                          <th style={{ padding: "8px 12px", textAlign: "left", fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase", borderBottom: `2px solid ${TOKENS.border}` }}>Address</th>
                          <th style={{ padding: "8px 12px", textAlign: "center", fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase", borderBottom: `2px solid ${TOKENS.border}`, width: 80 }}>Status</th>
                        </tr>
                      </thead>
                      <tbody>
                        {companyBranches.map((b) => (
                          <tr key={b.id}>
                            <td style={{ padding: "8px 12px", fontSize: "0.85rem", color: TOKENS.text, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>{b.name}</td>
                            <td style={{ padding: "8px 12px", fontSize: "0.85rem", color: TOKENS.textSecondary, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>{b.city}</td>
                            <td style={{ padding: "8px 12px", fontSize: "0.85rem", color: TOKENS.textSecondary, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>{b.address || "—"}</td>
                            <td style={{ padding: "8px 12px", textAlign: "center", borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>
                              <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: b.is_active ? TOKENS.successLight : TOKENS.errorLight, color: b.is_active ? TOKENS.success : TOKENS.error }}>{b.is_active ? "Active" : "Inactive"}</span>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  )}
                </div>
              )}

              {detailTab === "business" && (
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 16 }}>
                  {[
                    { label: "VAT Registered", value: selectedCompany.vat_registered, icon: Landmark },
                    { label: "WHT Agent", value: selectedCompany.wht_agent, icon: Briefcase },
                    { label: "E-Invoicing Enabled", value: selectedCompany.einvoicing_enabled, icon: Globe },
                  ].map((item) => (
                    <div key={item.label} style={{ padding: 16, borderRadius: TOKENS.radiusMd, border: `1px solid ${item.value ? TOKENS.success : TOKENS.border}`, background: item.value ? TOKENS.successLight : TOKENS.bg }}>
                      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
                        <item.icon size={18} style={{ color: item.value ? TOKENS.success : TOKENS.textTertiary }} />
                        <span style={{ fontSize: "0.85rem", fontWeight: 600, color: item.value ? TOKENS.success : TOKENS.textSecondary }}>{item.label}</span>
                      </div>
                      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                        {item.value ? <Check size={16} style={{ color: TOKENS.success }} /> : <X size={16} style={{ color: TOKENS.textTertiary }} />}
                        <span style={{ fontSize: "0.85rem", color: item.value ? TOKENS.success : TOKENS.textTertiary }}>{item.value ? "Enabled" : "Disabled"}</span>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        ) : (
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(320px, 1fr))", gap: 16 }}>
            {companies.map((company) => (
              <div
                key={company.id}
                onClick={() => setSelectedCompany(company)}
                onMouseEnter={(e) => { e.currentTarget.style.borderColor = TOKENS.primary; e.currentTarget.style.boxShadow = `0 4px 16px rgba(27,67,50,0.15)`; }}
                onMouseLeave={(e) => { e.currentTarget.style.borderColor = TOKENS.border; e.currentTarget.style.boxShadow = TOKENS.shadowSm; }}
                style={{
                  ...cardStyle,
                  borderLeft: company.id === activeCompanyId ? `3px solid ${TOKENS.accent}` : `3px solid transparent`,
                }}
              >
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: 12 }}>
                  <div>
                    <h3 style={{ fontSize: "0.95rem", fontWeight: 600, color: TOKENS.text, marginBottom: 2 }}>{company.name}</h3>
                    <span style={{ fontSize: "0.75rem", color: TOKENS.textTertiary, fontFamily: TOKENS.fontUi }}>RC: {company.rc_number}</span>
                  </div>
                  <span style={{ fontSize: "0.7rem", padding: "2px 8px", borderRadius: 4, background: company.id === activeCompanyId ? TOKENS.accentLight : company.is_active ? TOKENS.successLight : TOKENS.errorLight, color: company.id === activeCompanyId ? TOKENS.accent : company.is_active ? TOKENS.success : TOKENS.error, fontWeight: 600 }}>
                    {company.id === activeCompanyId ? "ACTIVE" : company.is_active ? "Active" : "Inactive"}
                  </span>
                </div>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 4, fontSize: "0.8rem", color: TOKENS.textSecondary }}>
                    <Briefcase size={12} /> {company.industry}
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: 4, fontSize: "0.8rem", color: TOKENS.textSecondary }}>
                    <Globe size={12} /> {company.base_currency}
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: 4, fontSize: "0.8rem", color: TOKENS.textSecondary }}>
                    <Calendar size={12} /> FY End: {MONTHS[(company.fiscal_year_end || 12) - 1]}
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: 4, fontSize: "0.8rem", color: TOKENS.textSecondary }}>
                    <MapPin size={12} /> {company.tax_authority}
                  </div>
                </div>
              </div>
            ))}
            {companies.length === 0 && (
              <div style={{ gridColumn: "1 / -1", textAlign: "center", padding: 60, color: TOKENS.textTertiary }}>
                <Building2 size={40} style={{ marginBottom: 12 }} />
                <p>No companies yet. Add your first company to get started.</p>
              </div>
            )}
          </div>
        )}

        {showAddModal && (
          <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }}>
            <div style={{ background: TOKENS.surface, borderRadius: TOKENS.radiusMd, padding: 24, width: 560, maxWidth: "90vw", maxHeight: "85vh", overflow: "auto", boxShadow: "0 8px 24px rgba(0,0,0,0.2)" }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
                <h2 style={{ fontSize: "1.1rem", fontWeight: 600, color: TOKENS.text, fontFamily: TOKENS.fontHeading }}>Add Company</h2>
                <button onClick={() => setShowAddModal(false)} style={{ color: TOKENS.textTertiary, cursor: "pointer" }}><X size={20} /></button>
              </div>
              <form onSubmit={handleAddCompany}>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
                  <div style={{ gridColumn: "1 / -1" }}>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>Company Name *</label>
                    <input type="text" value={addForm.name} onChange={(e) => setAddForm({ ...addForm, name: e.target.value })} required style={inputStyle} placeholder="e.g. Atharu Nigeria Ltd" />
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>RC Number *</label>
                    <input type="text" value={addForm.rc_number} onChange={(e) => setAddForm({ ...addForm, rc_number: e.target.value })} required style={inputStyle} placeholder="e.g. RC 1234567" />
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>TIN</label>
                    <input type="text" value={addForm.tin} onChange={(e) => setAddForm({ ...addForm, tin: e.target.value })} style={inputStyle} placeholder="e.g. 12345678-0001" />
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>Industry *</label>
                    <select value={addForm.industry} onChange={(e) => setAddForm({ ...addForm, industry: e.target.value })} required style={selectStyle}>
                      <option value="">Select Industry</option>
                      {INDUSTRIES.map((ind) => <option key={ind} value={ind}>{ind}</option>)}
                    </select>
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>Base Currency</label>
                    <select value={addForm.base_currency} onChange={(e) => setAddForm({ ...addForm, base_currency: e.target.value })} style={selectStyle}>
                      {CURRENCIES.map((c) => <option key={c} value={c}>{c}</option>)}
                    </select>
                  </div>
                  <div style={{ gridColumn: "1 / -1" }}>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>Address</label>
                    <input type="text" value={addForm.address} onChange={(e) => setAddForm({ ...addForm, address: e.target.value })} style={inputStyle} placeholder="Company address" />
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>Fiscal Year End Month</label>
                    <select value={addForm.fiscal_year_end} onChange={(e) => setAddForm({ ...addForm, fiscal_year_end: Number(e.target.value) })} style={selectStyle}>
                      {MONTHS.map((m, i) => <option key={i} value={i + 1}>{m}</option>)}
                    </select>
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "0.75rem", fontWeight: 600, color: TOKENS.textTertiary, textTransform: "uppercase" }}>Tax Authority</label>
                    <select value={addForm.tax_authority} onChange={(e) => setAddForm({ ...addForm, tax_authority: e.target.value })} style={selectStyle}>
                      {TAX_AUTHORITIES.map((ta) => <option key={ta} value={ta}>{ta}</option>)}
                    </select>
                  </div>
                  <div style={{ gridColumn: "1 / -1", display: "flex", gap: 16, marginTop: 4 }}>
                    <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: "0.85rem", color: TOKENS.textSecondary, cursor: "pointer" }}>
                      <input type="checkbox" checked={addForm.vat_registered} onChange={(e) => setAddForm({ ...addForm, vat_registered: e.target.checked })} style={{ accentColor: TOKENS.primary }} /> VAT Registered
                    </label>
                    <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: "0.85rem", color: TOKENS.textSecondary, cursor: "pointer" }}>
                      <input type="checkbox" checked={addForm.wht_agent} onChange={(e) => setAddForm({ ...addForm, wht_agent: e.target.checked })} style={{ accentColor: TOKENS.primary }} /> WHT Agent
                    </label>
                    <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: "0.85rem", color: TOKENS.textSecondary, cursor: "pointer" }}>
                      <input type="checkbox" checked={addForm.einvoicing_enabled} onChange={(e) => setAddForm({ ...addForm, einvoicing_enabled: e.target.checked })} style={{ accentColor: TOKENS.primary }} /> E-Invoicing Enabled
                    </label>
                  </div>
                </div>
                <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 20, paddingTop: 16, borderTop: `1px solid ${TOKENS.borderSubtle}` }}>
                  <button type="button" onClick={() => setShowAddModal(false)} style={{ padding: "8px 16px", borderRadius: TOKENS.radiusSm, fontSize: "0.85rem", background: TOKENS.bg, border: `1px solid ${TOKENS.border}`, color: TOKENS.textSecondary, cursor: "pointer" }}>Cancel</button>
                  <button type="submit" disabled={saving} style={{ padding: "8px 16px", borderRadius: TOKENS.radiusSm, fontSize: "0.85rem", fontWeight: 600, background: TOKENS.primary, color: "#FFFFFF", border: "none", cursor: "pointer", display: "flex", alignItems: "center", gap: 6 }}>
                    {saving ? <Loader2 size={14} style={{ animation: "spin 0.8s linear infinite" }} /> : <Check size={14} />} Create Company
                  </button>
                </div>
              </form>
            </div>
          </div>
        )}

        <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
      </div>
    </WorkspaceShell>
  );
}
