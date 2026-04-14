// Author: Quadri Atharu

"use client";

import { useEffect, useState, useCallback } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { getToken, getCompanyContext } from "@/lib/session";
import { apiGet, apiPost, apiPatch, apiDelete } from "@/lib/api";
import {
  BookOpen,
  Plus,
  ChevronRight,
  ChevronDown,
  FolderOpen,
  Search,
  Upload,
  Edit3,
  Trash2,
  X,
  Check,
  RefreshCw,
  AlertTriangle,
} from "lucide-react";

interface Account {
  id: string;
  code: string;
  name: string;
  account_type: string;
  subtype: string;
  parent_id: string | null;
  is_active: boolean;
  is_bank_account: boolean;
  is_tax_account: boolean;
  opening_balance: number;
  balance: number;
  children?: Account[];
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

const ACCOUNT_TYPES = [
  "Asset",
  "Liability",
  "Equity",
  "Revenue",
  "Expense",
];

const ACCOUNT_SUBTYPES: Record<string, string[]> = {
  Asset: ["Current Asset", "Fixed Asset", "Other Asset"],
  Liability: ["Current Liability", "Long-term Liability", "Other Liability"],
  Equity: ["Owner's Equity", "Retained Earnings", "Other Equity"],
  Revenue: ["Operating Revenue", "Other Revenue"],
  Expense: ["Operating Expense", "Other Expense", "Tax Expense"],
};

const TYPE_COLORS: Record<string, { bg: string; color: string }> = {
  Asset: { bg: "rgba(59,130,246,0.12)", color: "#2563EB" },
  Liability: { bg: "rgba(239,68,68,0.12)", color: "#DC2626" },
  Equity: { bg: "rgba(139,92,246,0.12)", color: "#7C3AED" },
  Revenue: { bg: "rgba(22,163,74,0.12)", color: SUCCESS },
  Expense: { bg: "rgba(212,175,55,0.15)", color: ACCENT },
};

function formatNaira(v: number) {
  return `₦${(v || 0).toLocaleString("en-NG", { minimumFractionDigits: 0, maximumFractionDigits: 0 })}`;
}

interface NewAccountForm {
  code: string;
  name: string;
  account_type: string;
  subtype: string;
  parent_id: string;
  opening_balance: number;
  is_bank_account: boolean;
  is_tax_account: boolean;
}

const emptyForm: NewAccountForm = {
  code: "",
  name: "",
  account_type: "Asset",
  subtype: "Current Asset",
  parent_id: "",
  opening_balance: 0,
  is_bank_account: false,
  is_tax_account: false,
};

export default function AccountingPage() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [allAccountsFlat, setAllAccountsFlat] = useState<Account[]>([]);
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());
  const [searchQuery, setSearchQuery] = useState("");
  const [filterType, setFilterType] = useState("all");
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showImportModal, setShowImportModal] = useState(false);
  const [showDeactivateConfirm, setShowDeactivateConfirm] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [form, setForm] = useState<NewAccountForm>({ ...emptyForm });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const loadAccounts = useCallback(async () => {
    setLoading(true);
    setError(null);
    const token = getToken();
    if (!token) {
      window.location.replace("/login");
      return;
    }
    try {
      const companyId = getCompanyContext() || "";
      const res = await apiGet(`/accounting/accounts?companyId=${companyId}`, token);
      if (res.ok) {
        const d = await res.json();
        const data = d.data || d.accounts || d || [];
        setAccounts(Array.isArray(data) ? data : []);
        setAllAccountsFlat(flattenAll(Array.isArray(data) ? data : []));
      } else {
        setError("Failed to load accounts.");
      }
    } catch {
      setError("Network error. Please try again.");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadAccounts();
  }, [loadAccounts]);

  function flattenAll(accs: Account[]): Account[] {
    const result: Account[] = [];
    for (const a of accs) {
      result.push(a);
      if (a.children?.length) result.push(...flattenAll(a.children));
    }
    return result;
  }

  function toggleExpand(id: string) {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function expandAll() {
    const ids = new Set<string>();
    function collect(accs: Account[]) {
      for (const a of accs) {
        if (a.children?.length) {
          ids.add(a.id);
          collect(a.children);
        }
      }
    }
    collect(accounts);
    setExpandedIds(ids);
  }

  function collapseAll() {
    setExpandedIds(new Set());
  }

  function filterAccounts(accs: Account[], query: string, type: string): Account[] {
    return accs
      .filter((a) => {
        const matchQuery = !query || a.code.toLowerCase().includes(query.toLowerCase()) || a.name.toLowerCase().includes(query.toLowerCase());
        const matchType = type === "all" || a.account_type === type;
        return matchQuery && matchType;
      })
      .map((a) => ({
        ...a,
        children: a.children?.length ? filterAccounts(a.children, query, type) : undefined,
      }))
      .filter((a) => {
        if (!query && type === "all") return true;
        const selfMatch = !query || a.code.toLowerCase().includes(query.toLowerCase()) || a.name.toLowerCase().includes(query.toLowerCase());
        const childMatch = a.children?.length ? a.children.length > 0 : false;
        return selfMatch || childMatch;
      });
  }

  const filteredAccounts = filterAccounts(accounts, searchQuery, filterType);

  function renderTree(accs: Account[], level: number): React.ReactNode[] {
    const nodes: React.ReactNode[] = [];
    for (const acc of accs) {
      const hasChildren = acc.children && acc.children.length > 0;
      const isExpanded = expandedIds.has(acc.id);
      const isEditing = editingId === acc.id;
      const tc = TYPE_COLORS[acc.account_type] || TYPE_COLORS.Asset;
      nodes.push(
        <div
          key={acc.id}
          style={{
            display: "flex",
            alignItems: "center",
            padding: "10px 12px",
            paddingLeft: 12 + level * 28,
            borderBottom: "1px solid #E9ECEF",
            fontSize: "0.85rem",
            cursor: hasChildren ? "pointer" : "default",
            background: !acc.is_active ? "#F8F9FA" : "transparent",
            opacity: acc.is_active ? 1 : 0.55,
          }}
          onClick={() => hasChildren && toggleExpand(acc.id)}
        >
          {hasChildren ? (
            isExpanded ? (
              <ChevronDown size={14} style={{ marginRight: 6, color: TEXT_TER, flexShrink: 0 }} />
            ) : (
              <ChevronRight size={14} style={{ marginRight: 6, color: TEXT_TER, flexShrink: 0 }} />
            )
          ) : (
            <span style={{ width: 20, marginRight: 6, flexShrink: 0 }} />
          )}
          <span style={{ fontFamily: '"JetBrains Mono", monospace', color: PRIMARY, fontWeight: 600, marginRight: 12, width: 60, flexShrink: 0 }}>
            {acc.code}
          </span>
          {isEditing ? (
            <input
              type="text"
              value={editName}
              onChange={(e) => setEditName(e.target.value)}
              onClick={(e) => e.stopPropagation()}
              onKeyDown={async (e) => {
                if (e.key === "Enter") {
                  await handleEditSave(acc.id);
                } else if (e.key === "Escape") {
                  setEditingId(null);
                }
              }}
              style={{ flex: 1, fontSize: "0.85rem", padding: "2px 6px", border: `1px solid ${PRIMARY}`, borderRadius: 4, background: SURFACE, color: TEXT }}
              autoFocus
            />
          ) : (
            <span style={{ flex: 1, color: TEXT }}>{acc.name}</span>
          )}
          <span style={{ fontSize: "0.72rem", padding: "2px 8px", borderRadius: 4, background: tc.bg, color: tc.color, marginRight: 8, flexShrink: 0 }}>
            {acc.account_type}
          </span>
          {acc.subtype && (
            <span style={{ fontSize: "0.68rem", padding: "2px 6px", borderRadius: 4, background: "rgba(134,142,150,0.1)", color: TEXT_TER, marginRight: 8, flexShrink: 0 }}>
              {acc.subtype}
            </span>
          )}
          <span style={{ fontSize: "0.82rem", color: TEXT_SEC, fontFamily: '"JetBrains Mono", monospace', width: 100, textAlign: "right", marginRight: 12, flexShrink: 0 }}>
            {formatNaira(acc.balance)}
          </span>
          <div style={{ display: "flex", gap: 4, flexShrink: 0 }} onClick={(e) => e.stopPropagation()}>
            {isEditing ? (
              <>
                <button onClick={() => handleEditSave(acc.id)} style={{ padding: "2px 6px", borderRadius: 4, background: `${SUCCESS}15`, color: SUCCESS, border: "none", cursor: "pointer" }}>
                  <Check size={14} />
                </button>
                <button onClick={() => setEditingId(null)} style={{ padding: "2px 6px", borderRadius: 4, background: `${ERROR}15`, color: ERROR, border: "none", cursor: "pointer" }}>
                  <X size={14} />
                </button>
              </>
            ) : (
              <>
                <button
                  onClick={() => { setEditingId(acc.id); setEditName(acc.name); }}
                  style={{ padding: "2px 6px", borderRadius: 4, background: "transparent", border: "none", cursor: "pointer", color: TEXT_TER }}
                  onMouseEnter={(e) => (e.currentTarget.style.color = PRIMARY)}
                  onMouseLeave={(e) => (e.currentTarget.style.color = TEXT_TER)}
                >
                  <Edit3 size={14} />
                </button>
                {acc.is_active && (
                  <button
                    onClick={() => setShowDeactivateConfirm(acc.id)}
                    style={{ padding: "2px 6px", borderRadius: 4, background: "transparent", border: "none", cursor: "pointer", color: TEXT_TER }}
                    onMouseEnter={(e) => (e.currentTarget.style.color = ERROR)}
                    onMouseLeave={(e) => (e.currentTarget.style.color = TEXT_TER)}
                  >
                    <Trash2 size={14} />
                  </button>
                )}
              </>
            )}
          </div>
        </div>
      );
      if (hasChildren && isExpanded) {
        nodes.push(...renderTree(acc.children!, level + 1));
      }
    }
    return nodes;
  }

  async function handleEditSave(id: string) {
    if (!editName.trim()) return;
    const token = getToken();
    try {
      const res = await apiPatch(`/accounting/accounts/${id}`, { name: editName.trim() }, token);
      if (res.ok) {
        setEditingId(null);
        loadAccounts();
      }
    } catch {
      // handle error silently
    }
  }

  async function handleCreate() {
    if (!form.code.trim() || !form.name.trim()) return;
    setSubmitting(true);
    const token = getToken();
    try {
      const body = {
        ...form,
        parent_id: form.parent_id || null,
        opening_balance: form.opening_balance || 0,
      };
      const res = await apiPost("/accounting/accounts", body, token);
      if (res.ok) {
        setShowCreateModal(false);
        setForm({ ...emptyForm });
        loadAccounts();
      }
    } catch {
      // handle error silently
    } finally {
      setSubmitting(false);
    }
  }

  async function handleDeactivate(id: string) {
    const token = getToken();
    try {
      const res = await apiDelete(`/accounting/accounts/${id}`, token);
      if (res.ok) {
        setShowDeactivateConfirm(null);
        loadAccounts();
      }
    } catch {
      // handle error silently
    }
  }

  async function handleImport(file: File) {
    const token = getToken();
    const text = await file.text();
    let accountsData: any[];
    try {
      if (file.name.endsWith(".json")) {
        accountsData = JSON.parse(text);
      } else {
        const lines = text.split("\n").filter((l) => l.trim());
        const headers = lines[0].split(",");
        accountsData = lines.slice(1).map((line) => {
          const vals = line.split(",");
          const obj: any = {};
          headers.forEach((h, i) => {
            obj[h.trim()] = vals[i]?.trim() || "";
          });
          return obj;
        });
      }
      const res = await apiPost("/accounting/accounts/import", { accounts: accountsData }, token);
      if (res.ok) {
        setShowImportModal(false);
        loadAccounts();
      }
    } catch {
      // handle error silently
    }
  }

  if (error && !loading) {
    return (
      <WorkspaceShell>
        <div style={{ padding: 24, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", minHeight: "60vh" }}>
          <AlertTriangle size={48} style={{ color: ERROR, marginBottom: 16 }} />
          <h2 style={{ fontSize: "1.1rem", fontWeight: 600, color: TEXT, marginBottom: 8 }}>Failed to Load Accounts</h2>
          <p style={{ fontSize: "0.85rem", color: TEXT_SEC, marginBottom: 20 }}>{error}</p>
          <button onClick={loadAccounts} style={{ display: "flex", alignItems: "center", gap: 8, padding: "10px 20px", borderRadius: 6, background: PRIMARY, color: "#FFFFFF", fontSize: "0.85rem", fontWeight: 500, border: "none", cursor: "pointer" }}>
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
            <BookOpen size={22} style={{ color: PRIMARY }} />
            Chart of Accounts
          </h1>
          <div style={{ display: "flex", gap: 8 }}>
            <button onClick={() => setShowImportModal(true)} style={{ display: "flex", alignItems: "center", gap: 6, padding: "8px 14px", borderRadius: 6, border: `1px solid ${BORDER}`, background: SURFACE, color: TEXT_SEC, fontSize: "0.82rem", cursor: "pointer" }}>
              <Upload size={14} /> Import COA
            </button>
            <button onClick={() => setShowCreateModal(true)} style={{ display: "flex", alignItems: "center", gap: 6, padding: "8px 16px", borderRadius: 6, background: PRIMARY, color: "#FFFFFF", fontSize: "0.82rem", fontWeight: 500, border: "none", cursor: "pointer" }}>
              <Plus size={14} /> Add Account
            </button>
          </div>
        </div>

        {/* Search & Filter Bar */}
        <div style={{ display: "flex", gap: 12, marginBottom: 16, flexWrap: "wrap", alignItems: "center" }}>
          <div style={{ position: "relative", flex: 1, minWidth: 200 }}>
            <Search size={14} style={{ position: "absolute", left: 10, top: "50%", transform: "translateY(-50%)", color: TEXT_TER }} />
            <input
              type="text"
              placeholder="Search by code or name..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              style={{ width: "100%", paddingLeft: 32, padding: "8px 12px", borderRadius: 6, border: `1px solid ${BORDER}`, background: SURFACE, fontSize: "0.85rem", color: TEXT }}
            />
          </div>
          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value)}
            style={{ padding: "8px 12px", borderRadius: 6, border: `1px solid ${BORDER}`, background: SURFACE, fontSize: "0.85rem", color: TEXT_SEC }}
          >
            <option value="all">All Types</option>
            {ACCOUNT_TYPES.map((t) => (
              <option key={t} value={t}>{t}</option>
            ))}
          </select>
          <button onClick={expandAll} style={{ padding: "6px 10px", borderRadius: 4, background: SURFACE, border: `1px solid ${BORDER}`, fontSize: "0.75rem", color: TEXT_SEC, cursor: "pointer" }}>Expand All</button>
          <button onClick={collapseAll} style={{ padding: "6px 10px", borderRadius: 4, background: SURFACE, border: `1px solid ${BORDER}`, fontSize: "0.75rem", color: TEXT_SEC, cursor: "pointer" }}>Collapse All</button>
        </div>

        {/* Account Tree */}
        <div style={{ background: SURFACE, border: `1px solid ${BORDER}`, borderRadius: 8, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)", maxHeight: "calc(100vh - 220px)", overflowY: "auto" }}>
          <div style={{ display: "flex", padding: "8px 12px", borderBottom: `1px solid ${BORDER}`, fontSize: "0.75rem", fontWeight: 600, color: TEXT_TER, textTransform: "uppercase", letterSpacing: 0.05 }}>
            <span style={{ width: 60 + 28, paddingLeft: 20 }}>Code</span>
            <span style={{ flex: 1 }}>Account Name</span>
            <span style={{ width: 80 }}>Type</span>
            <span style={{ width: 100, textAlign: "right" }}>Balance</span>
            <span style={{ width: 70, textAlign: "center" }}>Actions</span>
          </div>
          {loading ? (
            <div style={{ padding: 32, textAlign: "center" }}>
              <div className="splash-loader" style={{ margin: "0 auto 16px" }} />
              <p style={{ color: TEXT_TER, fontSize: "0.85rem" }}>Loading accounts...</p>
            </div>
          ) : filteredAccounts.length === 0 ? (
            <div style={{ textAlign: "center", padding: 40, color: TEXT_TER }}>
              <FolderOpen size={40} style={{ marginBottom: 12, opacity: 0.4 }} />
              <p style={{ fontSize: "0.95rem", fontWeight: 500 }}>No accounts found</p>
              <p style={{ fontSize: "0.82rem", marginTop: 4 }}>Create your first account or import a Chart of Accounts template</p>
            </div>
          ) : (
            renderTree(filteredAccounts, 0)
          )}
        </div>

        {/* Create Account Modal */}
        {showCreateModal && (
          <div style={{ position: "fixed", inset: 0, background: "rgba(26,26,46,0.5)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }} onClick={(e) => e.target === e.currentTarget && setShowCreateModal(false)}>
            <div style={{ background: SURFACE, borderRadius: 12, boxShadow: "0 12px 40px rgba(0,0,0,0.2)", width: "90%", maxWidth: 560, maxHeight: "85vh", overflow: "auto" }}>
              <div style={{ padding: "16px 24px", borderBottom: `1px solid ${BORDER}`, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <h2 style={{ fontSize: "1.1rem", fontWeight: 600, color: TEXT, fontFamily: '"DM Serif Display", serif' }}>New Account</h2>
                <button onClick={() => setShowCreateModal(false)} style={{ padding: 4, borderRadius: 4, border: "none", cursor: "pointer", color: TEXT_TER }}><X size={18} /></button>
              </div>
              <div style={{ padding: 24, display: "flex", flexDirection: "column", gap: 16 }}>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
                  <div>
                    <label style={{ display: "block", fontSize: "0.78rem", fontWeight: 500, color: TEXT_SEC, marginBottom: 4 }}>Account Code</label>
                    <input type="text" value={form.code} onChange={(e) => setForm({ ...form, code: e.target.value })} placeholder="e.g. 1000" style={{ width: "100%", padding: "8px 12px", borderRadius: 6, border: `1px solid ${BORDER}`, background: BG, color: TEXT, fontSize: "0.85rem" }} />
                  </div>
                  <div>
                    <label style={{ display: "block", fontSize: "0.78rem", fontWeight: 500, color: TEXT_SEC, marginBottom: 4 }}>Account Name</label>
                    <input type="text" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder="e.g. Cash on Hand" style={{ width: "100%", padding: "8px 12px", borderRadius: 6, border: `1px solid ${BORDER}`, background: BG, color: TEXT, fontSize: "0.85rem" }} />
                  </div>
                </div>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
                  <div>
                    <label style={{ display: "block", fontSize: "0.78rem", fontWeight: 500, color: TEXT_SEC, marginBottom: 4 }}>Account Type</label>
                    <select value={form.account_type} onChange={(e) => setForm({ ...form, account_type: e.target.value, subtype: ACCOUNT_SUBTYPES[e.target.value]?.[0] || "" })} style={{ width: "100%", padding: "8px 12px", borderRadius: 6, border: `1px solid ${BORDER}`, background: BG, color: TEXT, fontSize: "0.85rem" }}>
                      {ACCOUNT_TYPES.map((t) => (<option key={t} value={t}>{t}</option>))}
                    </select>
                  </div>
                  <div>
                    <label style={{ display: "block", fontSize: "0.78rem", fontWeight: 500, color: TEXT_SEC, marginBottom: 4 }}>Subtype</label>
                    <select value={form.subtype} onChange={(e) => setForm({ ...form, subtype: e.target.value })} style={{ width: "100%", padding: "8px 12px", borderRadius: 6, border: `1px solid ${BORDER}`, background: BG, color: TEXT, fontSize: "0.85rem" }}>
                      {(ACCOUNT_SUBTYPES[form.account_type] || []).map((s) => (<option key={s} value={s}>{s}</option>))}
                    </select>
                  </div>
                </div>
                <div>
                  <label style={{ display: "block", fontSize: "0.78rem", fontWeight: 500, color: TEXT_SEC, marginBottom: 4 }}>Parent Account</label>
                  <select value={form.parent_id} onChange={(e) => setForm({ ...form, parent_id: e.target.value })} style={{ width: "100%", padding: "8px 12px", borderRadius: 6, border: `1px solid ${BORDER}`, background: BG, color: TEXT, fontSize: "0.85rem" }}>
                    <option value="">None (Top Level)</option>
                    {allAccountsFlat.filter((a) => a.is_active).map((a) => (<option key={a.id} value={a.id}>{a.code} — {a.name}</option>))}
                  </select>
                </div>
                <div>
                  <label style={{ display: "block", fontSize: "0.78rem", fontWeight: 500, color: TEXT_SEC, marginBottom: 4 }}>Opening Balance</label>
                  <input type="number" value={form.opening_balance || ""} onChange={(e) => setForm({ ...form, opening_balance: parseFloat(e.target.value) || 0 })} placeholder="0" style={{ width: "100%", padding: "8px 12px", borderRadius: 6, border: `1px solid ${BORDER}`, background: BG, color: TEXT, fontSize: "0.85rem" }} />
                </div>
                <div style={{ display: "flex", gap: 16 }}>
                  <label style={{ display: "flex", alignItems: "center", gap: 8, fontSize: "0.85rem", color: TEXT, cursor: "pointer" }}>
                    <input type="checkbox" checked={form.is_bank_account} onChange={(e) => setForm({ ...form, is_bank_account: e.target.checked })} style={{ accentColor: PRIMARY }} />
                    Bank Account
                  </label>
                  <label style={{ display: "flex", alignItems: "center", gap: 8, fontSize: "0.85rem", color: TEXT, cursor: "pointer" }}>
                    <input type="checkbox" checked={form.is_tax_account} onChange={(e) => setForm({ ...form, is_tax_account: e.target.checked })} style={{ accentColor: PRIMARY }} />
                    Tax Account
                  </label>
                </div>
              </div>
              <div style={{ padding: "16px 24px", borderTop: `1px solid ${BORDER}`, display: "flex", justifyContent: "flex-end", gap: 8 }}>
                <button onClick={() => setShowCreateModal(false)} style={{ padding: "8px 16px", borderRadius: 6, border: `1px solid ${BORDER}`, background: SURFACE, color: TEXT_SEC, fontSize: "0.85rem", cursor: "pointer" }}>Cancel</button>
                <button onClick={handleCreate} disabled={!form.code.trim() || !form.name.trim() || submitting} style={{ padding: "8px 16px", borderRadius: 6, background: PRIMARY, color: "#FFFFFF", fontSize: "0.85rem", fontWeight: 500, border: "none", cursor: submitting ? "not-allowed" : "pointer", opacity: submitting ? 0.7 : 1 }}>
                  {submitting ? "Creating..." : "Create Account"}
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Deactivate Confirmation */}
        {showDeactivateConfirm && (
          <div style={{ position: "fixed", inset: 0, background: "rgba(26,26,46,0.5)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }}>
            <div style={{ background: SURFACE, borderRadius: 8, padding: 24, maxWidth: 400, boxShadow: "0 12px 40px rgba(0,0,0,0.2)" }}>
              <h3 style={{ fontSize: "1rem", fontWeight: 600, color: TEXT, marginBottom: 8 }}>Deactivate Account?</h3>
              <p style={{ fontSize: "0.85rem", color: TEXT_SEC, marginBottom: 20 }}>This account will be marked as inactive. Existing transactions will not be affected.</p>
              <div style={{ display: "flex", justifyContent: "flex-end", gap: 8 }}>
                <button onClick={() => setShowDeactivateConfirm(null)} style={{ padding: "8px 16px", borderRadius: 6, border: `1px solid ${BORDER}`, background: SURFACE, color: TEXT_SEC, cursor: "pointer" }}>Cancel</button>
                <button onClick={() => handleDeactivate(showDeactivateConfirm)} style={{ padding: "8px 16px", borderRadius: 6, background: ERROR, color: "#FFFFFF", fontWeight: 500, border: "none", cursor: "pointer" }}>Deactivate</button>
              </div>
            </div>
          </div>
        )}

        {/* Import Modal */}
        {showImportModal && (
          <div style={{ position: "fixed", inset: 0, background: "rgba(26,26,46,0.5)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }} onClick={(e) => e.target === e.currentTarget && setShowImportModal(false)}>
            <div style={{ background: SURFACE, borderRadius: 12, padding: 24, maxWidth: 480, width: "90%", boxShadow: "0 12px 40px rgba(0,0,0,0.2)" }}>
              <h3 style={{ fontSize: "1rem", fontWeight: 600, color: TEXT, marginBottom: 12, fontFamily: '"DM Serif Display", serif' }}>Import Chart of Accounts</h3>
              <p style={{ fontSize: "0.82rem", color: TEXT_SEC, marginBottom: 16 }}>Upload a JSON or CSV file with account data. CSV format: code, name, account_type, subtype, opening_balance</p>
              <input
                type="file"
                accept=".json,.csv"
                onChange={(e) => {
                  const file = e.target.files?.[0];
                  if (file) handleImport(file);
                }}
                style={{ width: "100%", padding: "12px", border: `1px dashed ${BORDER}`, borderRadius: 6, background: BG, color: TEXT, fontSize: "0.85rem" }}
              />
              <div style={{ display: "flex", justifyContent: "flex-end", marginTop: 16, gap: 8 }}>
                <button onClick={() => setShowImportModal(false)} style={{ padding: "8px 16px", borderRadius: 6, border: `1px solid ${BORDER}`, background: SURFACE, color: TEXT_SEC, cursor: "pointer" }}>Cancel</button>
              </div>
            </div>
          </div>
        )}
      </div>
    </WorkspaceShell>
  );
}
