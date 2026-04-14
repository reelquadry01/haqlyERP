// Author: Quadri Atharu

"use client";

import { useEffect, useState, useCallback } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { getToken, getCompanyContext } from "@/lib/session";
import { apiGet, apiPost, apiPatch } from "@/lib/api";
import {
  FileText,
  Plus,
  X,
  Check,
  AlertTriangle,
  RefreshCw,
  ChevronDown,
  ChevronRight,
  Search,
  RotateCcw,
  Send,
  Save,
  Eye,
} from "lucide-react";

interface JournalEntry {
  id: string;
  reference: string;
  entry_date: string;
  narration: string;
  reference_no: string;
  total_debit: number;
  total_credit: number;
  status: "draft" | "validated" | "approved" | "posted" | "reversed";
  created_by: string;
  created_at: string;
  approved_by: string | null;
  approved_at: string | null;
  posted_at: string | null;
  line_items: JournalLineItem[];
}

interface JournalLineItem {
  id: string;
  account_code: string;
  account_name: string;
  debit: number;
  credit: number;
  description: string;
}

interface AccountOption {
  code: string;
  name: string;
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

const STATUS_CONFIG: Record<string, { bg: string; color: string; label: string }> = {
  draft: { bg: "rgba(134,142,150,0.12)", color: TEXT_TER, label: "Draft" },
  validated: { bg: "rgba(59,130,246,0.12)", color: "#2563EB", label: "Validated" },
  approved: { bg: "rgba(22,163,74,0.12)", color: SUCCESS, label: "Approved" },
  posted: { bg: "rgba(212,175,55,0.15)", color: ACCENT, label: "Posted" },
  reversed: { bg: "rgba(220,38,38,0.12)", color: ERROR, label: "Reversed" },
};

function formatNaira(v: number) {
  return `₦${(v || 0).toLocaleString("en-NG", { minimumFractionDigits: 0, maximumFractionDigits: 0 })}`;
}

const TABS = ["all", "draft", "validated", "approved", "posted", "reversed"] as const;
type TabType = (typeof TABS)[number];

interface NewLineItem {
  account_code: string;
  account_name: string;
  debit: number;
  credit: number;
  description: string;
}

const emptyLineItem: NewLineItem = {
  account_code: "",
  account_name: "",
  debit: 0,
  credit: 0,
  description: "",
};

export default function JournalEntriesPage() {
  const [entries, setEntries] = useState<JournalEntry[]>([]);
  const [accounts, setAccounts] = useState<AccountOption[]>([]);
  const [activeTab, setActiveTab] = useState<TabType>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [showCreate, setShowCreate] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  const [accountSearch, setAccountSearch] = useState("");
  const [activeAccountField, setActiveAccountField] = useState<number | null>(null);

  const [newEntry, setNewEntry] = useState({
    entry_date: new Date().toISOString().split("T")[0],
    narration: "",
    reference_no: "",
    line_items: [{ ...emptyLineItem }, { ...emptyLineItem }] as NewLineItem[],
  });

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
      const [jeRes, accRes] = await Promise.all([
        apiGet(`/journal-entries?companyId=${companyId}`, token),
        apiGet(`/accounting/accounts?companyId=${companyId}&active=true`, token),
      ]);
      if (jeRes.ok) {
        const d = await jeRes.json();
        setEntries(d.data || d.entries || []);
      }
      if (accRes.ok) {
        const d = await accRes.json();
        const accData = d.data || d.accounts || d || [];
        setAccounts(
          (Array.isArray(accData) ? accData : []).map((a: any) => ({
            code: a.code,
            name: a.name,
          }))
        );
      }
      if (!jeRes.ok) setError("Failed to load journal entries.");
    } catch {
      setError("Network error. Please try again.");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const filteredEntries = entries.filter((e) => {
    const matchTab = activeTab === "all" || e.status === activeTab;
    const matchSearch =
      !searchQuery ||
      e.reference.toLowerCase().includes(searchQuery.toLowerCase()) ||
      e.narration.toLowerCase().includes(searchQuery.toLowerCase());
    return matchTab && matchSearch;
  });

  const totalDebit = newEntry.line_items.reduce((s, l) => s + (l.debit || 0), 0);
  const totalCredit = newEntry.line_items.reduce((s, l) => s + (l.credit || 0), 0);
  const isBalanced = totalDebit === totalCredit && totalDebit > 0;
  const difference = Math.abs(totalDebit - totalCredit);

  function addLineItem() {
    setNewEntry({ ...newEntry, line_items: [...newEntry.line_items, { ...emptyLineItem }] });
  }

  function removeLineItem(index: number) {
    if (newEntry.line_items.length <= 2) return;
    setNewEntry({ ...newEntry, line_items: newEntry.line_items.filter((_, i) => i !== index) });
  }

  function updateLineItem(index: number, field: keyof NewLineItem, value: string | number) {
    const items = [...newEntry.line_items];
    items[index] = { ...items[index], [field]: value };
    setNewEntry({ ...newEntry, line_items: items });
  }

  function selectAccount(index: number, code: string) {
    const acc = accounts.find((a) => a.code === code);
    if (acc) {
      const items = [...newEntry.line_items];
      items[index] = { ...items[index], account_code: acc.code, account_name: acc.name };
      setNewEntry({ ...newEntry, line_items: items });
    }
    setActiveAccountField(null);
    setAccountSearch("");
  }

  function getFilteredAccounts() {
    if (!accountSearch) return accounts.slice(0, 20);
    return accounts.filter(
      (a) =>
        a.code.toLowerCase().includes(accountSearch.toLowerCase()) ||
        a.name.toLowerCase().includes(accountSearch.toLowerCase())
    ).slice(0, 20);
  }

  async function handleSubmit(status: "draft" | "validated") {
    if (!isBalanced && status === "validated") return;
    setSubmitting(true);
    const token = getToken();
    try {
      const body = {
        entry_date: newEntry.entry_date,
        narration: newEntry.narration,
        reference_no: newEntry.reference_no,
        status,
        line_items: newEntry.line_items.map((l) => ({
          account_code: l.account_code,
          debit: l.debit || 0,
          credit: l.credit || 0,
          description: l.description,
        })),
      };
      const res = await apiPost("/journal-entries", body, token);
      if (res.ok) {
        setShowCreate(false);
        setNewEntry({
          entry_date: new Date().toISOString().split("T")[0],
          narration: "",
          reference_no: "",
          line_items: [{ ...emptyLineItem }, { ...emptyLineItem }],
        });
        loadData();
      }
    } catch {
      // handle error silently
    } finally {
      setSubmitting(false);
    }
  }

  async function handleStatusChange(id: string, newStatus: string) {
    setActionLoading(id);
    const token = getToken();
    try {
      const res = await apiPatch(`/journal-entries/${id}`, { status: newStatus }, token);
      if (res.ok) {
        loadData();
        setExpandedId(null);
      }
    } catch {
      // handle error silently
    } finally {
      setActionLoading(null);
    }
  }

  async function handleReverse(id: string) {
    setActionLoading(id);
    const token = getToken();
    try {
      const res = await apiPost(`/journal-entries/${id}/reverse`, {}, token);
      if (res.ok) {
        loadData();
        setExpandedId(null);
      }
    } catch {
      // handle error silently
    } finally {
      setActionLoading(null);
    }
  }

  const tabCounts = entries.reduce<Record<string, number>>((acc, e) => {
    acc[e.status] = (acc[e.status] || 0) + 1;
    return acc;
  }, {});

  if (error && !loading) {
    return (
      <WorkspaceShell>
        <div style={{ padding: 24, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", minHeight: "60vh" }}>
          <AlertTriangle size={48} style={{ color: ERROR, marginBottom: 16 }} />
          <h2 style={{ fontSize: "1.1rem", fontWeight: 600, color: TEXT, marginBottom: 8 }}>Failed to Load Journal Entries</h2>
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
            <FileText size={22} style={{ color: PRIMARY }} />
            Journal Entries
          </h1>
          <button onClick={() => setShowCreate(true)} style={{ display: "flex", alignItems: "center", gap: 6, padding: "8px 16px", borderRadius: 6, background: PRIMARY, color: "#FFFFFF", fontSize: "0.82rem", fontWeight: 500, border: "none", cursor: "pointer" }}>
            <Plus size={14} /> New Entry
          </button>
        </div>

        {/* Tabs */}
        <div style={{ display: "flex", gap: 2, marginBottom: 16, borderBottom: `2px solid ${BORDER}`, overflowX: "auto" }}>
          {TABS.map((tab) => {
            const isActive = activeTab === tab;
            const count = tab === "all" ? entries.length : tabCounts[tab] || 0;
            return (
              <button
                key={tab}
                onClick={() => setActiveTab(tab)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 6,
                  padding: "10px 14px",
                  fontSize: "0.82rem",
                  fontWeight: isActive ? 600 : 400,
                  color: isActive ? PRIMARY : TEXT_TER,
                  borderBottom: isActive ? `2px solid ${PRIMARY}` : "2px solid transparent",
                  marginBottom: -2,
                  whiteSpace: "nowrap",
                  textTransform: "capitalize",
                  cursor: "pointer",
                  background: "transparent",
                  borderLeft: "none",
                  borderRight: "none",
                  borderTop: "none",
                }}
              >
                {tab}
                {count > 0 && (
                  <span style={{ fontSize: "0.68rem", padding: "1px 6px", borderRadius: 10, background: isActive ? "rgba(27,67,50,0.12)" : "rgba(134,142,150,0.1)", color: isActive ? PRIMARY : TEXT_TER }}>
                    {count}
                  </span>
                )}
              </button>
            );
          })}
        </div>

        {/* Search */}
        <div style={{ marginBottom: 16 }}>
          <div style={{ position: "relative", maxWidth: 320 }}>
            <Search size={14} style={{ position: "absolute", left: 10, top: "50%", transform: "translateY(-50%)", color: TEXT_TER }} />
            <input
              type="text"
              placeholder="Search by reference or narration..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              style={{ width: "100%", paddingLeft: 32, padding: "8px 12px", borderRadius: 6, border: `1px solid ${BORDER}`, background: SURFACE, fontSize: "0.85rem", color: TEXT }}
            />
          </div>
        </div>

        {/* Entry List */}
        <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, overflow: "hidden", boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
          {loading ? (
            <div style={{ padding: 40, textAlign: "center" }}>
              <div className="splash-loader" style={{ margin: "0 auto 16px" }} />
              <p style={{ color: TEXT_TER, fontSize: "0.85rem" }}>Loading entries...</p>
            </div>
          ) : filteredEntries.length === 0 ? (
            <div style={{ textAlign: "center", padding: 40, color: TEXT_TER }}>
              <FileText size={40} style={{ marginBottom: 12, opacity: 0.4 }} />
              <p style={{ fontSize: "0.95rem", fontWeight: 500 }}>No journal entries found</p>
              <p style={{ fontSize: "0.82rem", marginTop: 4 }}>Create your first journal entry to get started</p>
            </div>
          ) : (
            filteredEntries.map((entry) => {
              const sc = STATUS_CONFIG[entry.status] || STATUS_CONFIG.draft;
              const isExpanded = expandedId === entry.id;
              return (
                <div key={entry.id} style={{ borderBottom: `1px solid #E9ECEF` }}>
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      padding: "12px 16px",
                      cursor: "pointer",
                      background: isExpanded ? BG : "transparent",
                    }}
                    onClick={() => setExpandedId(isExpanded ? null : entry.id)}
                  >
                    {isExpanded ? <ChevronDown size={14} style={{ marginRight: 8, color: TEXT_TER }} /> : <ChevronRight size={14} style={{ marginRight: 8, color: TEXT_TER }} />}
                    <span style={{ fontFamily: '"JetBrains Mono", monospace', color: PRIMARY, fontWeight: 600, width: 120, fontSize: "0.85rem" }}>
                      {entry.reference}
                    </span>
                    <span style={{ width: 100, fontSize: "0.85rem", color: TEXT_SEC }}>{entry.entry_date}</span>
                    <span style={{ flex: 1, fontSize: "0.85rem", color: TEXT, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                      {entry.narration}
                    </span>
                    <span style={{ width: 110, textAlign: "right", fontSize: "0.85rem", color: TEXT_SEC, fontFamily: '"JetBrains Mono", monospace' }}>
                      {formatNaira(entry.total_debit)}
                    </span>
                    <span style={{ width: 110, textAlign: "right", fontSize: "0.85rem", color: TEXT_SEC, fontFamily: '"JetBrains Mono", monospace', marginLeft: 12 }}>
                      {formatNaira(entry.total_credit)}
                    </span>
                    <span style={{ marginLeft: 12, fontSize: "0.75rem", padding: "3px 10px", borderRadius: 4, background: sc.bg, color: sc.color, fontWeight: 500 }}>
                      {sc.label}
                    </span>
                  </div>

                  {/* Expanded Detail */}
                  {isExpanded && (
                    <div style={{ padding: "0 16px 16px 38px", background: BG }}>
                      <table style={{ width: "100%", borderCollapse: "collapse", marginBottom: 12, background: SURFACE, borderRadius: 6, overflow: "hidden" }}>
                        <thead>
                          <tr>
                            {["Account", "Debit", "Credit", "Description"].map((h) => (
                              <th key={h} style={{ padding: "8px 12px", textAlign: h === "Debit" || h === "Credit" ? "right" : "left", fontSize: "0.75rem", fontWeight: 600, color: TEXT_TER, textTransform: "uppercase", borderBottom: `1px solid ${BORDER}`, background: BG }}>
                                {h}
                              </th>
                            ))}
                          </tr>
                        </thead>
                        <tbody>
                          {entry.line_items.map((line) => (
                            <tr key={line.id}>
                              <td style={{ padding: "8px 12px", fontSize: "0.82rem", borderBottom: "1px solid #E9ECEF" }}>
                                <span style={{ fontFamily: '"JetBrains Mono", monospace', color: PRIMARY, fontWeight: 500, marginRight: 8 }}>{line.account_code}</span>
                                <span style={{ color: TEXT }}>{line.account_name}</span>
                              </td>
                              <td style={{ padding: "8px 12px", fontSize: "0.82rem", textAlign: "right", fontFamily: '"JetBrains Mono", monospace', color: TEXT_SEC, borderBottom: "1px solid #E9ECEF" }}>
                                {line.debit ? formatNaira(line.debit) : ""}
                              </td>
                              <td style={{ padding: "8px 12px", fontSize: "0.82rem", textAlign: "right", fontFamily: '"JetBrains Mono", monospace', color: TEXT_SEC, borderBottom: "1px solid #E9ECEF" }}>
                                {line.credit ? formatNaira(line.credit) : ""}
                              </td>
                              <td style={{ padding: "8px 12px", fontSize: "0.82rem", color: TEXT_TER, borderBottom: "1px solid #E9ECEF" }}>{line.description}</td>
                            </tr>
                          ))}
                        </tbody>
                        <tfoot>
                          <tr style={{ fontWeight: 600 }}>
                            <td style={{ padding: "8px 12px", fontSize: "0.82rem", color: TEXT }}>Totals</td>
                            <td style={{ padding: "8px 12px", fontSize: "0.82rem", textAlign: "right", fontFamily: '"JetBrains Mono", monospace', color: TEXT }}>{formatNaira(entry.total_debit)}</td>
                            <td style={{ padding: "8px 12px", fontSize: "0.82rem", textAlign: "right", fontFamily: '"JetBrains Mono", monospace', color: TEXT }}>{formatNaira(entry.total_credit)}</td>
                            <td />
                          </tr>
                        </tfoot>
                      </table>

                      {/* Audit Trail */}
                      <div style={{ display: "flex", gap: 16, fontSize: "0.75rem", color: TEXT_TER, marginBottom: 12, flexWrap: "wrap" }}>
                        <span>Created: {entry.created_at} by {entry.created_by}</span>
                        {entry.approved_by && <span>Approved: {entry.approved_at} by {entry.approved_by}</span>}
                        {entry.posted_at && <span>Posted: {entry.posted_at}</span>}
                      </div>

                      {/* Action Buttons */}
                      <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                        {entry.status === "draft" && (
                          <button
                            onClick={() => handleStatusChange(entry.id, "validated")}
                            disabled={actionLoading === entry.id}
                            style={{ display: "flex", alignItems: "center", gap: 6, padding: "6px 14px", borderRadius: 6, background: "#2563EB", color: "#FFFFFF", fontSize: "0.78rem", fontWeight: 500, border: "none", cursor: "pointer" }}
                          >
                            <Check size={14} /> Validate
                          </button>
                        )}
                        {entry.status === "validated" && (
                          <button
                            onClick={() => handleStatusChange(entry.id, "approved")}
                            disabled={actionLoading === entry.id}
                            style={{ display: "flex", alignItems: "center", gap: 6, padding: "6px 14px", borderRadius: 6, background: SUCCESS, color: "#FFFFFF", fontSize: "0.78rem", fontWeight: 500, border: "none", cursor: "pointer" }}
                          >
                            <Send size={14} /> Approve
                          </button>
                        )}
                        {entry.status === "approved" && (
                          <button
                            onClick={() => handleStatusChange(entry.id, "posted")}
                            disabled={actionLoading === entry.id}
                            style={{ display: "flex", alignItems: "center", gap: 6, padding: "6px 14px", borderRadius: 6, background: ACCENT, color: TEXT, fontSize: "0.78rem", fontWeight: 500, border: "none", cursor: "pointer" }}
                          >
                            <Eye size={14} /> Post
                          </button>
                        )}
                        {entry.status === "posted" && (
                          <button
                            onClick={() => handleReverse(entry.id)}
                            disabled={actionLoading === entry.id}
                            style={{ display: "flex", alignItems: "center", gap: 6, padding: "6px 14px", borderRadius: 6, background: ERROR, color: "#FFFFFF", fontSize: "0.78rem", fontWeight: 500, border: "none", cursor: "pointer" }}
                          >
                            <RotateCcw size={14} /> Reverse
                          </button>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              );
            })
          )}
        </div>

        {/* Create Journal Entry Modal */}
        {showCreate && (
          <div style={{ position: "fixed", inset: 0, background: "rgba(26,26,46,0.5)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }} onClick={(e) => e.target === e.currentTarget && setShowCreate(false)}>
            <div style={{ background: SURFACE, borderRadius: 12, boxShadow: "0 12px 40px rgba(0,0,0,0.2)", width: "95%", maxWidth: 960, maxHeight: "90vh", overflow: "auto" }}>
              <div style={{ padding: "16px 24px", borderBottom: `1px solid ${BORDER}`, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <h2 style={{ fontSize: "1.1rem", fontWeight: 600, color: TEXT, fontFamily: '"DM Serif Display", serif' }}>New Journal Entry</h2>
                <button onClick={() => setShowCreate(false)} style={{ padding: 4, borderRadius: 4, border: "none", cursor: "pointer", color: TEXT_TER }}><X size={18} /></button>
              </div>

              <div style={{ padding: 24 }}>
                <div style={{ display: "flex", gap: 12, marginBottom: 20, flexWrap: "wrap" }}>
                  <div style={{ flex: "0 0 160px" }}>
                    <label style={{ display: "block", fontSize: "0.78rem", fontWeight: 500, color: TEXT_SEC, marginBottom: 4 }}>Date</label>
                    <input type="date" value={newEntry.entry_date} onChange={(e) => setNewEntry({ ...newEntry, entry_date: e.target.value })} style={{ width: "100%", padding: "8px 10px", borderRadius: 6, border: `1px solid ${BORDER}`, background: BG, color: TEXT, fontSize: "0.85rem" }} />
                  </div>
                  <div style={{ flex: 1, minWidth: 200 }}>
                    <label style={{ display: "block", fontSize: "0.78rem", fontWeight: 500, color: TEXT_SEC, marginBottom: 4 }}>Narration</label>
                    <input type="text" value={newEntry.narration} onChange={(e) => setNewEntry({ ...newEntry, narration: e.target.value })} placeholder="Describe this journal entry" style={{ width: "100%", padding: "8px 10px", borderRadius: 6, border: `1px solid ${BORDER}`, background: BG, color: TEXT, fontSize: "0.85rem" }} />
                  </div>
                  <div style={{ flex: "0 0 160px" }}>
                    <label style={{ display: "block", fontSize: "0.78rem", fontWeight: 500, color: TEXT_SEC, marginBottom: 4 }}>Reference</label>
                    <input type="text" value={newEntry.reference_no} onChange={(e) => setNewEntry({ ...newEntry, reference_no: e.target.value })} placeholder="Optional" style={{ width: "100%", padding: "8px 10px", borderRadius: 6, border: `1px solid ${BORDER}`, background: BG, color: TEXT, fontSize: "0.85rem" }} />
                  </div>
                </div>

                <h3 style={{ fontSize: "0.9rem", fontWeight: 600, marginBottom: 10, color: TEXT }}>Line Items</h3>
                <div style={{ overflowX: "auto" }}>
                  <table style={{ width: "100%", borderCollapse: "collapse", marginBottom: 8 }}>
                    <thead>
                      <tr>
                        {["Account", "Debit", "Credit", "Description", ""].map((h, i) => (
                          <th key={h} style={{ padding: "8px 10px", textAlign: i === 1 || i === 2 ? "right" : "left", fontSize: "0.75rem", fontWeight: 600, color: TEXT_TER, textTransform: "uppercase", borderBottom: `1px solid ${BORDER}` }}>
                            {h}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {newEntry.line_items.map((item, i) => (
                        <tr key={i}>
                          <td style={{ padding: "6px 8px", position: "relative" }}>
                            <input
                              type="text"
                              value={activeAccountField === i ? accountSearch : item.account_code || ""}
                              onFocus={() => { setActiveAccountField(i); setAccountSearch(""); }}
                              onChange={(e) => {
                                setAccountSearch(e.target.value);
                                updateLineItem(i, "account_code", e.target.value);
                              }}
                              placeholder="Search account..."
                              style={{ width: "100%", padding: "6px 8px", borderRadius: 4, border: `1px solid ${BORDER}`, background: BG, color: TEXT, fontSize: "0.82rem" }}
                            />
                            {activeAccountField === i && (
                              <div style={{ position: "absolute", top: "100%", left: 0, right: 0, background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 4, boxShadow: "0 4px 12px rgba(0,0,0,0.1)", zIndex: 50, maxHeight: 200, overflowY: "auto" }}>
                                {getFilteredAccounts().map((a) => (
                                  <div
                                    key={a.code}
                                    onClick={() => selectAccount(i, a.code)}
                                    style={{ padding: "6px 10px", fontSize: "0.82rem", cursor: "pointer", display: "flex", gap: 8 }}
                                    onMouseEnter={(e) => (e.currentTarget.style.background = BG)}
                                    onMouseLeave={(e) => (e.currentTarget.style.background = SURFACE)}
                                  >
                                    <span style={{ fontFamily: '"JetBrains Mono", monospace', color: PRIMARY, fontWeight: 500 }}>{a.code}</span>
                                    <span style={{ color: TEXT }}>{a.name}</span>
                                  </div>
                                ))}
                              </div>
                            )}
                          </td>
                          <td style={{ padding: "6px 8px" }}>
                            <input type="number" value={item.debit || ""} onChange={(e) => updateLineItem(i, "debit", parseFloat(e.target.value) || 0)} placeholder="0" style={{ width: 110, padding: "6px 8px", borderRadius: 4, border: `1px solid ${BORDER}`, background: BG, color: TEXT, textAlign: "right", fontSize: "0.82rem" }} />
                          </td>
                          <td style={{ padding: "6px 8px" }}>
                            <input type="number" value={item.credit || ""} onChange={(e) => updateLineItem(i, "credit", parseFloat(e.target.value) || 0)} placeholder="0" style={{ width: 110, padding: "6px 8px", borderRadius: 4, border: `1px solid ${BORDER}`, background: BG, color: TEXT, textAlign: "right", fontSize: "0.82rem" }} />
                          </td>
                          <td style={{ padding: "6px 8px" }}>
                            <input type="text" value={item.description} onChange={(e) => updateLineItem(i, "description", e.target.value)} placeholder="Description" style={{ width: "100%", padding: "6px 8px", borderRadius: 4, border: `1px solid ${BORDER}`, background: BG, color: TEXT, fontSize: "0.82rem" }} />
                          </td>
                          <td style={{ padding: "6px 4px" }}>
                            {newEntry.line_items.length > 2 && (
                              <button onClick={() => removeLineItem(i)} style={{ padding: "2px 6px", borderRadius: 4, border: "none", cursor: "pointer", color: ERROR, background: "transparent" }}>
                                <X size={14} />
                              </button>
                            )}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                    <tfoot>
                      <tr style={{ fontWeight: 600 }}>
                        <td style={{ padding: "8px 10px", fontSize: "0.82rem", color: TEXT }}>Totals</td>
                        <td style={{ padding: "8px 10px", textAlign: "right", fontSize: "0.82rem", color: TEXT, fontFamily: '"JetBrains Mono", monospace' }}>{formatNaira(totalDebit)}</td>
                        <td style={{ padding: "8px 10px", textAlign: "right", fontSize: "0.82rem", color: TEXT, fontFamily: '"JetBrains Mono", monospace' }}>{formatNaira(totalCredit)}</td>
                        <td colSpan={2} />
                      </tr>
                    </tfoot>
                  </table>
                </div>

                <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 16, flexWrap: "wrap" }}>
                  <button onClick={addLineItem} style={{ display: "flex", alignItems: "center", gap: 4, padding: "6px 12px", borderRadius: 6, border: `1px solid ${BORDER}`, background: SURFACE, color: TEXT_SEC, fontSize: "0.78rem", cursor: "pointer" }}>
                    <Plus size={14} /> Add Line
                  </button>
                  <span
                    style={{
                      fontSize: "0.78rem",
                      padding: "3px 10px",
                      borderRadius: 4,
                      background: isBalanced ? "rgba(22,163,74,0.12)" : totalDebit || totalCredit ? "rgba(220,38,38,0.12)" : "rgba(134,142,150,0.08)",
                      color: isBalanced ? SUCCESS : totalDebit || totalCredit ? ERROR : TEXT_TER,
                      fontWeight: 500,
                    }}
                  >
                    {isBalanced ? "Entry Balanced" : totalDebit || totalCredit ? `Difference: ${formatNaira(difference)}` : "Enter amounts to check balance"}
                  </span>
                </div>
              </div>

              <div style={{ padding: "16px 24px", borderTop: `1px solid ${BORDER}`, display: "flex", justifyContent: "flex-end", gap: 8, flexWrap: "wrap" }}>
                <button onClick={() => setShowCreate(false)} style={{ padding: "8px 16px", borderRadius: 6, border: `1px solid ${BORDER}`, background: SURFACE, color: TEXT_SEC, fontSize: "0.85rem", cursor: "pointer" }}>Cancel</button>
                <button onClick={() => handleSubmit("draft")} disabled={submitting} style={{ display: "flex", alignItems: "center", gap: 6, padding: "8px 16px", borderRadius: 6, border: `1px solid ${BORDER}`, background: SURFACE, color: TEXT_SEC, fontSize: "0.85rem", cursor: submitting ? "not-allowed" : "pointer" }}>
                  <Save size={14} /> Save Draft
                </button>
                <button onClick={() => handleSubmit("validated")} disabled={!isBalanced || submitting} style={{ display: "flex", alignItems: "center", gap: 6, padding: "8px 16px", borderRadius: 6, background: PRIMARY, color: "#FFFFFF", fontSize: "0.85rem", fontWeight: 500, border: "none", cursor: !isBalanced || submitting ? "not-allowed" : "pointer", opacity: !isBalanced ? 0.5 : 1 }}>
                  <Check size={14} /> Validate &amp; Submit
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </WorkspaceShell>
  );
}
