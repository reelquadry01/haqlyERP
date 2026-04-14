// Author: Quadri Atharu
"use client";

import { useEffect, useState, useCallback } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet, apiPost, apiPatch } from "@/lib/api";
import { Building2, Plus, PlayCircle, Trash2, X, Calculator } from "lucide-react";

const T = {
  primary: "#1B4332",
  accent: "#D4AF37",
  bg: "#F8F9FA",
  surface: "#FFFFFF",
  text: "#1A1A2E",
  error: "#DC2626",
  success: "#16A34A",
  border: "#E9ECEF",
  muted: "#868E96",
  radius: 8,
  radiusSm: 6,
} as const;

const CATEGORIES = ["Land", "Building", "Machinery", "Vehicle", "Furniture", "IT Equipment"] as const;
const DEP_METHODS = ["Straight Line", "Reducing Balance"] as const;

interface FixedAsset {
  id: string;
  asset_code: string;
  name: string;
  category: string;
  acquisition_date: string;
  acquisition_cost: number;
  residual_value: number;
  useful_life_years: number;
  useful_life_months: number;
  accumulated_depreciation: number;
  net_book_value: number;
  depreciation_method: string;
  location: string;
  department: string;
  status: string;
}

interface DepreciationEntry {
  period: string;
  cost: number;
  opening_accum_dep: number;
  current_dep: number;
  closing_accum_dep: number;
  nbv: number;
}

const STATUS_MAP: Record<string, { bg: string; color: string }> = {
  active: { bg: "rgba(22,163,74,0.12)", color: T.success },
  disposed: { bg: "rgba(220,38,38,0.12)", color: T.error },
  revalued: { bg: "rgba(59,130,246,0.12)", color: "#3B82F6" },
  under_construction: { bg: "rgba(245,158,11,0.12)", color: "#F59E0B" },
};

function StatusPill({ status }: { status: string }) {
  const c = STATUS_MAP[status] || STATUS_MAP.active;
  return (
    <span style={{ fontSize: "0.75rem", padding: "2px 10px", borderRadius: T.radiusSm, background: c.bg, color: c.color, fontWeight: 600 }}>
      {status.replace(/_/g, " ")}
    </span>
  );
}

function naira(v: number) {
  return `₦${(v || 0).toLocaleString("en-NG")}`;
}

function Overlay({ children, onClose }: { children: React.ReactNode; onClose: () => void }) {
  return (
    <div style={{ position: "fixed", inset: 0, zIndex: 1000, display: "flex", alignItems: "center", justifyContent: "center", background: "rgba(0,0,0,0.45)" }} onClick={onClose}>
      <div style={{ background: T.surface, borderRadius: T.radius, width: 520, maxHeight: "90vh", overflowY: "auto", boxShadow: "0 16px 48px rgba(0,0,0,0.18)" }} onClick={(e) => e.stopPropagation()}>
        {children}
      </div>
    </div>
  );
}

function ModalHeader({ title, onClose }: { title: string; onClose: () => void }) {
  return (
    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "16px 24px", borderBottom: `1px solid ${T.border}` }}>
      <h3 style={{ fontSize: "1rem", fontWeight: 600, color: T.text, fontFamily: "'DM Serif', serif" }}>{title}</h3>
      <button onClick={onClose} style={{ background: "none", border: "none", cursor: "pointer", color: T.muted }}><X size={18} /></button>
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  width: "100%", padding: "8px 12px", fontSize: "0.85rem", borderRadius: T.radiusSm,
  border: `1px solid ${T.border}`, color: T.text, fontFamily: "Inter, sans-serif", boxSizing: "border-box",
};

const labelStyle: React.CSSProperties = { fontSize: "0.8rem", fontWeight: 500, color: T.muted, marginBottom: 4, display: "block" };

const btnPrimary: React.CSSProperties = {
  display: "inline-flex", alignItems: "center", gap: 6, padding: "8px 20px", fontSize: "0.85rem",
  fontWeight: 600, borderRadius: T.radiusSm, border: "none", cursor: "pointer",
  background: T.primary, color: "#fff", fontFamily: "Inter, sans-serif",
};

const btnGhost: React.CSSProperties = {
  display: "inline-flex", alignItems: "center", gap: 6, padding: "8px 16px", fontSize: "0.85rem",
  fontWeight: 500, borderRadius: T.radiusSm, border: `1px solid ${T.border}`, cursor: "pointer",
  background: T.surface, color: T.text, fontFamily: "Inter, sans-serif",
};

const btnDanger: React.CSSProperties = {
  ...btnPrimary, background: T.error,
};

function computeSchedule(asset: FixedAsset): DepreciationEntry[] {
  const entries: DepreciationEntry[] = [];
  const cost = asset.acquisition_cost || 0;
  const residual = asset.residual_value || 0;
  const lifeMonths = asset.useful_life_months || (asset.useful_life_years || 1) * 12;
  let accumDep = 0;
  const startDate = new Date(asset.acquisition_date);
  for (let m = 0; m < lifeMonths; m++) {
    const openAccum = accumDep;
    let currentDep: number;
    if (asset.depreciation_method === "Reducing Balance") {
      const nbvOpen = cost - openAccum;
      const rate = residual > 0 ? (1 - Math.pow(residual / cost, 1 / lifeMonths)) : (2 / lifeMonths);
      currentDep = Math.max(0, nbvOpen * rate);
    } else {
      currentDep = Math.max(0, (cost - residual) / lifeMonths);
    }
    const closeAccum = openAccum + currentDep;
    const nbv = cost - closeAccum;
    const periodDate = new Date(startDate.getFullYear(), startDate.getMonth() + m, 1);
    entries.push({
      period: periodDate.toLocaleDateString("en-NG", { year: "numeric", month: "short" }),
      cost, opening_accum_dep: openAccum, current_dep: currentDep,
      closing_accum_dep: closeAccum, nbv: Math.max(0, nbv),
    });
    accumDep = closeAccum;
  }
  return entries;
}

const emptyAddForm = {
  name: "", category: CATEGORIES[0], acquisition_date: "", acquisition_cost: "",
  residual_value: "", useful_life_years: "", depreciation_method: DEP_METHODS[0],
  location: "", department: "",
};

export default function FixedAssetsPage() {
  const token = getToken();
  const [assets, setAssets] = useState<FixedAsset[]>([]);
  const [depreciationData, setDepreciationData] = useState<DepreciationEntry[]>([]);
  const [tab, setTab] = useState<"register" | "schedule">("register");
  const [showAdd, setShowAdd] = useState(false);
  const [showDisposal, setShowDisposal] = useState(false);
  const [selectedAsset, setSelectedAsset] = useState<FixedAsset | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [addForm, setAddForm] = useState({ ...emptyAddForm });
  const [disposalForm, setDisposalForm] = useState({ sale_proceeds: "", disposal_date: "" });

  const loadAssets = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await apiGet("/fixed-assets/assets", token);
      if (res.ok) setAssets((await res.json()).data || []);
      else setError("Failed to load assets");
    } catch {
      setError("Network error");
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadAssets(); }, [loadAssets]);

  useEffect(() => {
    async function loadSchedule() {
      if (!selectedAsset) return;
      try {
        const res = await apiGet(`/fixed-assets/assets/${selectedAsset.id}/depreciation`, token);
        if (res.ok) setDepreciationData((await res.json()).data || []);
        else setDepreciationData(computeSchedule(selectedAsset));
      } catch {
        setDepreciationData(computeSchedule(selectedAsset));
      }
    }
    if (tab === "schedule") loadSchedule();
  }, [selectedAsset, tab, token]);

  async function handleAddAsset() {
    setSubmitting(true);
    try {
      const res = await apiPost("/fixed-assets/assets", {
        ...addForm,
        acquisition_cost: Number(addForm.acquisition_cost) || 0,
        residual_value: Number(addForm.residual_value) || 0,
        useful_life_years: Number(addForm.useful_life_years) || 1,
      }, token);
      if (res.ok) {
        setShowAdd(false);
        setAddForm({ ...emptyAddForm });
        loadAssets();
      } else {
        setError("Failed to create asset");
      }
    } catch {
      setError("Network error");
    } finally {
      setSubmitting(false);
    }
  }

  async function handleRunDepreciation() {
    setSubmitting(true);
    try {
      const res = await apiPost("/fixed-assets/depreciation/run", { period: new Date().toISOString().slice(0, 7) }, token);
      if (res.ok) loadAssets();
      else setError("Depreciation run failed");
    } catch {
      setError("Network error");
    } finally {
      setSubmitting(false);
    }
  }

  async function handleDisposal() {
    if (!selectedAsset) return;
    setSubmitting(true);
    try {
      const proceeds = Number(disposalForm.sale_proceeds) || 0;
      const nbv = selectedAsset.net_book_value || 0;
      const gainLoss = proceeds - nbv;
      const res = await apiPatch(`/fixed-assets/assets/${selectedAsset.id}`, {
        status: "disposed",
        disposal_date: disposalForm.disposal_date,
        sale_proceeds: proceeds,
        gain_loss_on_disposal: gainLoss,
      }, token);
      if (res.ok) {
        setShowDisposal(false);
        setDisposalForm({ sale_proceeds: "", disposal_date: "" });
        setSelectedAsset(null);
        loadAssets();
      } else {
        setError("Disposal failed");
      }
    } catch {
      setError("Network error");
    } finally {
      setSubmitting(false);
    }
  }

  const assetColumns: Column<FixedAsset>[] = [
    { key: "asset_code", label: "Asset #", sortable: true, width: "110px" },
    { key: "name", label: "Name", sortable: true },
    { key: "category", label: "Category", width: "120px" },
    { key: "acquisition_cost", label: "Cost", align: "right", width: "130px", render: (v: number) => naira(v) },
    { key: "accumulated_depreciation", label: "Accum. Dep.", align: "right", width: "130px", render: (v: number) => naira(v) },
    { key: "net_book_value", label: "NBV", align: "right", width: "130px", render: (v: number) => naira(v) },
    { key: "location", label: "Location", width: "110px" },
    { key: "status", label: "Status", width: "130px", render: (v: string) => <StatusPill status={v} /> },
  ];

  const depColumns: Column<DepreciationEntry>[] = [
    { key: "period", label: "Period", width: "110px" },
    { key: "cost", label: "Cost", align: "right", width: "120px", render: (v: number) => naira(v) },
    { key: "opening_accum_dep", label: "Opening Accum Dep", align: "right", width: "140px", render: (v: number) => naira(v) },
    { key: "current_dep", label: "Current Dep", align: "right", width: "120px", render: (v: number) => naira(v) },
    { key: "closing_accum_dep", label: "Closing Accum Dep", align: "right", width: "140px", render: (v: number) => naira(v) },
    { key: "nbv", label: "NBV", align: "right", width: "120px", render: (v: number) => naira(v) },
  ];

  const disposalGainLoss = selectedAsset && disposalForm.sale_proceeds
    ? (Number(disposalForm.sale_proceeds) || 0) - (selectedAsset.net_book_value || 0)
    : null;

  return (
    <WorkspaceShell>
      <div style={{ padding: 24, background: T.bg, minHeight: "100vh" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: T.text, fontFamily: "'DM Serif', serif" }}>
            <Building2 size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> Fixed Assets
          </h1>
          <div style={{ display: "flex", gap: 8 }}>
            <button onClick={handleRunDepreciation} disabled={submitting} style={{ ...btnGhost, borderColor: T.accent, color: T.accent }}>
              <PlayCircle size={16} /> Run Depreciation
            </button>
            <button onClick={() => setShowAdd(true)} style={btnPrimary}>
              <Plus size={16} /> Add Asset
            </button>
          </div>
        </div>

        {error && (
          <div style={{ padding: "10px 16px", marginBottom: 16, borderRadius: T.radiusSm, background: "rgba(220,38,38,0.08)", color: T.error, fontSize: "0.85rem" }}>{error}</div>
        )}

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: `2px solid ${T.border}` }}>
          {(["register", "schedule"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{
              padding: "10px 20px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400,
              color: tab === t ? T.primary : T.muted,
              borderBottom: tab === t ? `2px solid ${T.primary}` : "2px solid transparent",
              marginBottom: -2, background: "none", borderLeft: "none", borderRight: "none",
              borderTop: "none", cursor: "pointer", fontFamily: "Inter, sans-serif",
            }}>
              {t === "register" ? "Asset Register" : "Depreciation Schedule"}
            </button>
          ))}
        </div>

        <div style={{ background: T.surface, border: `1px solid ${T.border}`, borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.06)" }}>
          {loading ? (
            <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
          ) : tab === "register" ? (
            <DataTable
              columns={assetColumns}
              data={assets}
              pageSize={15}
              emptyMessage="No fixed assets registered"
              actions={[
                { label: "View Schedule", icon: <Calculator size={14} />, onClick: (row) => { setSelectedAsset(row); setTab("schedule"); } },
                { label: "Dispose", icon: <Trash2 size={14} />, onClick: (row) => { setSelectedAsset(row); setShowDisposal(true); }, variant: "danger" },
              ]}
            />
          ) : selectedAsset ? (
            <>
              <div style={{ marginBottom: 16, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <div>
                  <span style={{ fontSize: "0.9rem", fontWeight: 600, color: T.text }}>{selectedAsset.name}</span>
                  <span style={{ fontSize: "0.8rem", color: T.muted, marginLeft: 12 }}>{selectedAsset.asset_code}</span>
                </div>
                <button onClick={() => setTab("register")} style={{ ...btnGhost, fontSize: "0.8rem", padding: "6px 14px" }}>Back to Register</button>
              </div>
              <DataTable columns={depColumns} data={depreciationData} pageSize={12} emptyMessage="No depreciation data" />
            </>
          ) : (
            <div style={{ textAlign: "center", padding: 40, color: T.muted, fontSize: "0.9rem" }}>
              Select an asset from the register to view its depreciation schedule
            </div>
          )}
        </div>

        {showAdd && (
          <Overlay onClose={() => setShowAdd(false)}>
            <ModalHeader title="Register New Asset" onClose={() => setShowAdd(false)} />
            <div style={{ padding: 24, display: "flex", flexDirection: "column", gap: 16 }}>
              <div>
                <label style={labelStyle}>Asset Name</label>
                <input style={inputStyle} value={addForm.name} onChange={(e) => setAddForm({ ...addForm, name: e.target.value })} placeholder="e.g. Delivery Van" />
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
                <div>
                  <label style={labelStyle}>Category</label>
                  <select style={inputStyle} value={addForm.category} onChange={(e) => setAddForm({ ...addForm, category: e.target.value })}>
                    {CATEGORIES.map((c) => <option key={c} value={c}>{c}</option>)}
                  </select>
                </div>
                <div>
                  <label style={labelStyle}>Depreciation Method</label>
                  <select style={inputStyle} value={addForm.depreciation_method} onChange={(e) => setAddForm({ ...addForm, depreciation_method: e.target.value })}>
                    {DEP_METHODS.map((m) => <option key={m} value={m}>{m}</option>)}
                  </select>
                </div>
              </div>
              <div>
                <label style={labelStyle}>Purchase Date</label>
                <input type="date" style={inputStyle} value={addForm.acquisition_date} onChange={(e) => setAddForm({ ...addForm, acquisition_date: e.target.value })} />
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 12 }}>
                <div>
                  <label style={labelStyle}>Cost (₦)</label>
                  <input type="number" style={inputStyle} value={addForm.acquisition_cost} onChange={(e) => setAddForm({ ...addForm, acquisition_cost: e.target.value })} placeholder="0" />
                </div>
                <div>
                  <label style={labelStyle}>Residual Value (₦)</label>
                  <input type="number" style={inputStyle} value={addForm.residual_value} onChange={(e) => setAddForm({ ...addForm, residual_value: e.target.value })} placeholder="0" />
                </div>
                <div>
                  <label style={labelStyle}>Useful Life (Years)</label>
                  <input type="number" style={inputStyle} value={addForm.useful_life_years} onChange={(e) => setAddForm({ ...addForm, useful_life_years: e.target.value })} placeholder="5" />
                </div>
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
                <div>
                  <label style={labelStyle}>Location</label>
                  <input style={inputStyle} value={addForm.location} onChange={(e) => setAddForm({ ...addForm, location: e.target.value })} placeholder="Warehouse A" />
                </div>
                <div>
                  <label style={labelStyle}>Department</label>
                  <input style={inputStyle} value={addForm.department} onChange={(e) => setAddForm({ ...addForm, department: e.target.value })} placeholder="Operations" />
                </div>
              </div>
              <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 8 }}>
                <button onClick={() => setShowAdd(false)} style={btnGhost}>Cancel</button>
                <button onClick={handleAddAsset} disabled={submitting || !addForm.name} style={{ ...btnPrimary, opacity: submitting || !addForm.name ? 0.6 : 1 }}>
                  {submitting ? "Saving..." : "Register Asset"}
                </button>
              </div>
            </div>
          </Overlay>
        )}

        {showDisposal && selectedAsset && (
          <Overlay onClose={() => { setShowDisposal(false); setDisposalForm({ sale_proceeds: "", disposal_date: "" }); }}>
            <ModalHeader title={`Dispose ${selectedAsset.name}`} onClose={() => setShowDisposal(false)} />
            <div style={{ padding: 24, display: "flex", flexDirection: "column", gap: 16 }}>
              <div style={{ padding: 12, borderRadius: T.radiusSm, background: T.bg, fontSize: "0.85rem", color: T.text }}>
                <strong>Current NBV:</strong> <span style={{ color: T.primary, fontWeight: 600 }}>{naira(selectedAsset.net_book_value)}</span>
              </div>
              <div>
                <label style={labelStyle}>Sale Proceeds (₦)</label>
                <input type="number" style={inputStyle} value={disposalForm.sale_proceeds} onChange={(e) => setDisposalForm({ ...disposalForm, sale_proceeds: e.target.value })} placeholder="0" />
              </div>
              <div>
                <label style={labelStyle}>Disposal Date</label>
                <input type="date" style={inputStyle} value={disposalForm.disposal_date} onChange={(e) => setDisposalForm({ ...disposalForm, disposal_date: e.target.value })} />
              </div>
              {disposalGainLoss !== null && (
                <div style={{ padding: 12, borderRadius: T.radiusSm, background: disposalGainLoss >= 0 ? "rgba(22,163,74,0.08)" : "rgba(220,38,38,0.08)", fontSize: "0.85rem" }}>
                  <strong>{disposalGainLoss >= 0 ? "Gain" : "Loss"} on Disposal:</strong>{" "}
                  <span style={{ color: disposalGainLoss >= 0 ? T.success : T.error, fontWeight: 600 }}>
                    {naira(Math.abs(disposalGainLoss))}
                  </span>
                </div>
              )}
              <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 8 }}>
                <button onClick={() => setShowDisposal(false)} style={btnGhost}>Cancel</button>
                <button onClick={handleDisposal} disabled={submitting || !disposalForm.sale_proceeds} style={{ ...btnDanger, opacity: submitting || !disposalForm.sale_proceeds ? 0.6 : 1 }}>
                  {submitting ? "Processing..." : "Confirm Disposal"}
                </button>
              </div>
            </div>
          </Overlay>
        )}
      </div>
    </WorkspaceShell>
  );
}
