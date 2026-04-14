// Author: Quadri Atharu
"use client";

import { useEffect, useState, useCallback } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet, apiPost, apiPatch } from "@/lib/api";
import { Users, Plus, Phone, Mail, Calendar, CheckCircle, X, ChevronLeft, ChevronRight, TrendingUp, MessageSquare, Target } from "lucide-react";

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

interface Contact {
  id: string;
  name: string;
  organization: string;
  email: string;
  phone: string;
  contact_type: string;
  last_activity: string;
}

interface Deal {
  id: string;
  name: string;
  contact_name: string;
  contact_id: string;
  value: number;
  probability: number;
  stage: string;
  expected_close_date: string;
  assigned_to: string;
}

interface Activity {
  id: string;
  activity_type: string;
  subject: string;
  contact_name: string;
  deal_name: string;
  status: string;
  due_date: string;
  notes: string;
  created_at: string;
}

const PIPELINE_STAGES = [
  { key: "lead", label: "Lead", color: "#6B7280" },
  { key: "qualified", label: "Qualified", color: "#3B82F6" },
  { key: "proposal", label: "Proposal", color: "#D4AF37" },
  { key: "negotiation", label: "Negotiation", color: "#F59E0B" },
  { key: "won", label: "Won", color: "#16A34A" },
  { key: "lost", label: "Lost", color: "#DC2626" },
];

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

export default function CrmPage() {
  const token = getToken();
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [deals, setDeals] = useState<Deal[]>([]);
  const [activities, setActivities] = useState<Activity[]>([]);
  const [tab, setTab] = useState<"contacts" | "pipeline" | "activities">("contacts");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddContact, setShowAddContact] = useState(false);
  const [showAddDeal, setShowAddDeal] = useState(false);
  const [showLogActivity, setShowLogActivity] = useState(false);
  const [movingDeal, setMovingDeal] = useState<string | null>(null);

  const [newContact, setNewContact] = useState({ name: "", organization: "", email: "", phone: "", contact_type: "Customer" });
  const [newDeal, setNewDeal] = useState({ name: "", contact_id: "", value: "", expected_close_date: "", stage: "lead" });
  const [newActivity, setNewActivity] = useState({ activity_type: "call", subject: "", notes: "", contact_id: "", deal_id: "" });

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [cRes, dRes, aRes] = await Promise.all([
        apiGet("/crm/contacts", token),
        apiGet("/crm/deals", token),
        apiGet("/crm/activities", token),
      ]);
      if (cRes.ok) setContacts((await cRes.json()).data || []);
      if (dRes.ok) setDeals((await dRes.json()).data || []);
      if (aRes.ok) setActivities((await aRes.json()).data || []);
    } catch {
      setError("Failed to load CRM data");
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadData(); }, [loadData]);

  const handleAddContact = async () => {
    try {
      const res = await apiPost("/crm/contacts", newContact, token);
      if (res.ok) { setShowAddContact(false); setNewContact({ name: "", organization: "", email: "", phone: "", contact_type: "Customer" }); loadData(); }
    } catch { setError("Failed to add contact"); }
  };

  const handleAddDeal = async () => {
    try {
      const res = await apiPost("/crm/deals", { ...newDeal, value: Number(newDeal.value), probability: 20 }, token);
      if (res.ok) { setShowAddDeal(false); setNewDeal({ name: "", contact_id: "", value: "", expected_close_date: "", stage: "lead" }); loadData(); }
    } catch { setError("Failed to add deal"); }
  };

  const handleLogActivity = async () => {
    try {
      const res = await apiPost("/crm/activities", newActivity, token);
      if (res.ok) { setShowLogActivity(false); setNewActivity({ activity_type: "call", subject: "", notes: "", contact_id: "", deal_id: "" }); loadData(); }
    } catch { setError("Failed to log activity"); }
  };

  const handleMoveDeal = async (dealId: string, newStage: string) => {
    setMovingDeal(dealId);
    try {
      const res = await apiPatch(`/crm/deals/${dealId}/stage`, { stage: newStage }, token);
      if (res.ok) loadData();
    } catch { setError("Failed to move deal"); }
    setMovingDeal(null);
  };

  const contactColumns: Column<Contact>[] = [
    { key: "name", label: "Name", sortable: true },
    { key: "organization", label: "Company", width: "150px" },
    { key: "email", label: "Email", width: "180px" },
    { key: "phone", label: "Phone", width: "120px" },
    {
      key: "contact_type", label: "Type", width: "90px",
      render: (v: string) => {
        const colors: Record<string, { bg: string; color: string }> = {
          Customer: { bg: "rgba(22,163,74,0.12)", color: T.success },
          Supplier: { bg: "rgba(59,130,246,0.12)", color: "#3B82F6" },
          Partner: { bg: "rgba(212,175,55,0.12)", color: T.accent },
        };
        const c = colors[v] || colors.Customer;
        return <span style={{ fontSize: "0.75rem", padding: "2px 10px", borderRadius: 12, fontWeight: 600, background: c.bg, color: c.color }}>{v}</span>;
      },
    },
    { key: "last_activity", label: "Last Activity", width: "120px" },
  ];

  const activityColumns: Column<Activity>[] = [
    {
      key: "activity_type", label: "Type", width: "80px",
      render: (v: string) => {
        const icons: Record<string, typeof Phone> = { call: Phone, email: Mail, meeting: Calendar, note: MessageSquare };
        const Icon = icons[v] || CheckCircle;
        return <span style={{ display: "flex", alignItems: "center", gap: 4 }}><Icon size={12} style={{ color: T.primary }} />{v}</span>;
      },
    },
    { key: "subject", label: "Subject", sortable: true },
    { key: "contact_name", label: "Contact", width: "130px" },
    {
      key: "status", label: "Status", width: "90px",
      render: (v: string) => {
        const c: Record<string, { bg: string; color: string }> = {
          completed: { bg: "rgba(22,163,74,0.12)", color: T.success },
          pending: { bg: "rgba(245,158,11,0.12)", color: "#F59E0B" },
          cancelled: { bg: "rgba(107,114,128,0.12)", color: T.muted },
        };
        const s = c[v] || c.pending;
        return <span style={{ fontSize: "0.75rem", padding: "2px 10px", borderRadius: 12, fontWeight: 600, background: s.bg, color: s.color }}>{v}</span>;
      },
    },
    { key: "due_date", label: "Due", width: "110px" },
  ];

  const stageIndex = (key: string) => PIPELINE_STAGES.findIndex(s => s.key === key);
  const weightedPipeline = PIPELINE_STAGES.map(stage => {
    const stageDeals = deals.filter(d => d.stage === stage.key);
    const total = stageDeals.reduce((s, d) => s + d.value, 0);
    const weighted = stageDeals.reduce((s, d) => s + (d.value * d.probability / 100), 0);
    return { ...stage, deals: stageDeals, total, weighted, count: stageDeals.length };
  });
  const totalWeighted = weightedPipeline.reduce((s, w) => s + w.weighted, 0);

  const btnPrimary: React.CSSProperties = { display: "inline-flex", alignItems: "center", gap: 6, padding: "8px 16px", borderRadius: T.radius, fontSize: "0.85rem", fontWeight: 600, background: T.primary, color: "#fff", border: "none", cursor: "pointer" };
  const btnSecondary: React.CSSProperties = { padding: "8px 16px", borderRadius: T.radiusSm, border: `1px solid ${T.border}`, background: T.surface, color: T.text, cursor: "pointer", fontSize: "0.85rem" };
  const btnSubmit: React.CSSProperties = { padding: "8px 20px", borderRadius: T.radiusSm, background: T.primary, color: "#fff", border: "none", cursor: "pointer", fontWeight: 600, fontSize: "0.85rem" };

  return (
    <WorkspaceShell>
      <div style={{ padding: 24, fontFamily: T.font, background: T.bg, minHeight: "100%" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontFamily: T.fontDisplay, fontSize: "1.35rem", color: T.text, display: "flex", alignItems: "center", gap: 8 }}>
            <Users size={22} style={{ color: T.primary }} /> CRM
          </h1>
          <div style={{ display: "flex", gap: 8 }}>
            <button onClick={() => setShowAddContact(true)} style={btnPrimary}><Plus size={16} /> Contact</button>
            <button onClick={() => setShowAddDeal(true)} style={{ ...btnPrimary, background: T.accent, color: T.text }}><Target size={16} /> Deal</button>
            <button onClick={() => setShowLogActivity(true)} style={btnSecondary}><Calendar size={16} /> Log Activity</button>
          </div>
        </div>

        <div style={{ display: "flex", gap: 16, marginBottom: 20 }}>
          <div style={{ flex: 1, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
            <div style={{ fontSize: "0.75rem", color: T.muted, fontWeight: 500, marginBottom: 4 }}>Total Contacts</div>
            <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.primary }}>{contacts.length}</div>
          </div>
          <div style={{ flex: 1, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
            <div style={{ fontSize: "0.75rem", color: T.muted, fontWeight: 500, marginBottom: 4 }}>Active Deals</div>
            <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.accent }}>{deals.filter(d => !["won", "lost"].includes(d.stage)).length}</div>
          </div>
          <div style={{ flex: 1, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
            <div style={{ fontSize: "0.75rem", color: T.muted, fontWeight: 500, marginBottom: 4, display: "flex", alignItems: "center", gap: 4 }}><TrendingUp size={12} /> Weighted Pipeline</div>
            <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.primary }}>{fmt(totalWeighted)}</div>
          </div>
          <div style={{ flex: 1, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
            <div style={{ fontSize: "0.75rem", color: T.muted, fontWeight: 500, marginBottom: 4 }}>Won This Month</div>
            <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.success }}>{fmt(deals.filter(d => d.stage === "won").reduce((s, d) => s + d.value, 0))}</div>
          </div>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: `2px solid ${T.border}` }}>
          {(["contacts", "pipeline", "activities"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{
              padding: "10px 18px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400, color: tab === t ? T.primary : T.muted,
              borderBottom: tab === t ? `2px solid ${T.primary}` : "2px solid transparent", marginBottom: -2, background: "none", borderLeft: "none", borderRight: "none", borderTop: "none", cursor: "pointer", textTransform: "capitalize",
            }}>
              {t}
            </button>
          ))}
        </div>

        {error && <div style={{ background: "rgba(220,38,38,0.08)", border: `1px solid ${T.error}`, borderRadius: T.radius, padding: 12, marginBottom: 16, color: T.error, fontSize: "0.85rem" }}>{error}</div>}

        {loading ? (
          <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
        ) : tab === "contacts" ? (
          <div style={{ background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 20, boxShadow: "0 1px 3px rgba(0,0,0,0.06)" }}>
            <DataTable columns={contactColumns} data={contacts} pageSize={15} emptyMessage="No contacts found" />
          </div>
        ) : tab === "pipeline" ? (
          <div>
            <div style={{ display: "flex", gap: 12, overflowX: "auto", paddingBottom: 8 }}>
              {weightedPipeline.map((stage) => (
                <div key={stage.key} style={{ minWidth: 220, flex: 1, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
                  <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
                    <span style={{ fontSize: "0.8rem", fontWeight: 600, color: T.text }}>{stage.label}</span>
                    <span style={{ fontSize: "0.7rem", padding: "1px 6px", borderRadius: 12, background: `${stage.color}18`, color: stage.color, fontWeight: 600 }}>{stage.count}</span>
                  </div>
                  <div style={{ fontSize: "0.8rem", color: T.muted, marginBottom: 4 }}>Total: <strong style={{ color: T.text }}>{fmt(stage.total)}</strong></div>
                  <div style={{ fontSize: "0.75rem", color: T.muted, marginBottom: 12 }}>Weighted: <strong style={{ color: T.accent }}>{fmt(stage.weighted)}</strong></div>
                  {stage.deals.map((deal) => (
                    <div key={deal.id} style={{ background: T.bg, border: `1px solid ${T.border}`, borderRadius: T.radiusSm, padding: 10, marginBottom: 6 }}>
                      <div style={{ fontSize: "0.8rem", fontWeight: 500, color: T.text }}>{deal.name}</div>
                      <div style={{ fontSize: "0.75rem", color: T.muted }}>{deal.contact_name}</div>
                      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginTop: 6 }}>
                        <span style={{ fontSize: "0.8rem", fontWeight: 600, color: T.primary }}>{fmt(deal.value)}</span>
                        <span style={{ fontSize: "0.7rem", color: T.muted }}>{deal.probability}%</span>
                      </div>
                      <div style={{ display: "flex", justifyContent: "space-between", marginTop: 6 }}>
                        <button
                          disabled={stageIndex(deal.stage) === 0}
                          onClick={() => { const prev = PIPELINE_STAGES[stageIndex(deal.stage) - 1]; if (prev) handleMoveDeal(deal.id, prev.key); }}
                          style={{ padding: "2px 6px", fontSize: "0.7rem", border: `1px solid ${T.border}`, borderRadius: 4, background: T.surface, cursor: stageIndex(deal.stage) === 0 ? "default" : "pointer", opacity: stageIndex(deal.stage) === 0 ? 0.4 : 1, color: T.muted }}
                        ><ChevronLeft size={10} /></button>
                        <button
                          disabled={stageIndex(deal.stage) === PIPELINE_STAGES.length - 1}
                          onClick={() => { const next = PIPELINE_STAGES[stageIndex(deal.stage) + 1]; if (next) handleMoveDeal(deal.id, next.key); }}
                          style={{ padding: "2px 6px", fontSize: "0.7rem", border: `1px solid ${T.border}`, borderRadius: 4, background: T.surface, cursor: stageIndex(deal.stage) === PIPELINE_STAGES.length - 1 ? "default" : "pointer", opacity: stageIndex(deal.stage) === PIPELINE_STAGES.length - 1 ? 0.4 : 1, color: T.muted }}
                        ><ChevronRight size={10} /></button>
                      </div>
                    </div>
                  ))}
                  {stage.deals.length === 0 && <div style={{ fontSize: "0.75rem", color: T.muted, textAlign: "center", padding: 12 }}>No deals</div>}
                </div>
              ))}
            </div>
            <div style={{ marginTop: 16, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
              <h4 style={{ fontSize: "0.9rem", fontWeight: 600, color: T.text, marginBottom: 12, display: "flex", alignItems: "center", gap: 6 }}><TrendingUp size={16} style={{ color: T.accent }} /> Revenue Forecast by Stage</h4>
              <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                {weightedPipeline.map(stage => (
                  <div key={stage.key} style={{ display: "flex", alignItems: "center", gap: 12 }}>
                    <span style={{ fontSize: "0.8rem", width: 100, color: T.muted }}>{stage.label}</span>
                    <div style={{ flex: 1, height: 20, background: T.bg, borderRadius: 4, overflow: "hidden" }}>
                      <div style={{ height: "100%", width: `${totalWeighted ? Math.min(100, (stage.weighted / totalWeighted) * 100) : 0}%`, background: stage.color, borderRadius: 4, transition: "width 0.3s" }} />
                    </div>
                    <span style={{ fontSize: "0.8rem", fontWeight: 600, color: T.text, width: 100, textAlign: "right" }}>{fmt(stage.weighted)}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        ) : (
          <div style={{ background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 20, boxShadow: "0 1px 3px rgba(0,0,0,0.06)" }}>
            <DataTable columns={activityColumns} data={activities} pageSize={15} emptyMessage="No activities found" />
          </div>
        )}

        <Modal open={showAddContact} onClose={() => setShowAddContact(false)} title="Add Contact">
          <Field label="Name"><input style={inputStyle} value={newContact.name} onChange={(e) => setNewContact({ ...newContact, name: e.target.value })} /></Field>
          <Field label="Company"><input style={inputStyle} value={newContact.organization} onChange={(e) => setNewContact({ ...newContact, organization: e.target.value })} /></Field>
          <div style={{ display: "flex", gap: 12 }}>
            <div style={{ flex: 1 }}><Field label="Email"><input type="email" style={inputStyle} value={newContact.email} onChange={(e) => setNewContact({ ...newContact, email: e.target.value })} /></Field></div>
            <div style={{ flex: 1 }}><Field label="Phone"><input style={inputStyle} value={newContact.phone} onChange={(e) => setNewContact({ ...newContact, phone: e.target.value })} /></Field></div>
          </div>
          <Field label="Type">
            <select style={selectStyle} value={newContact.contact_type} onChange={(e) => setNewContact({ ...newContact, contact_type: e.target.value })}>
              <option value="Customer">Customer</option>
              <option value="Supplier">Supplier</option>
              <option value="Partner">Partner</option>
            </select>
          </Field>
          <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 20 }}>
            <button onClick={() => setShowAddContact(false)} style={btnSecondary}>Cancel</button>
            <button onClick={handleAddContact} style={btnSubmit}>Add Contact</button>
          </div>
        </Modal>

        <Modal open={showAddDeal} onClose={() => setShowAddDeal(false)} title="Add Deal">
          <Field label="Deal Name"><input style={inputStyle} value={newDeal.name} onChange={(e) => setNewDeal({ ...newDeal, name: e.target.value })} /></Field>
          <Field label="Contact">
            <select style={selectStyle} value={newDeal.contact_id} onChange={(e) => setNewDeal({ ...newDeal, contact_id: e.target.value })}>
              <option value="">Select contact...</option>
              {contacts.map(c => <option key={c.id} value={c.id}>{c.name}</option>)}
            </select>
          </Field>
          <div style={{ display: "flex", gap: 12 }}>
            <div style={{ flex: 1 }}><Field label="Value (₦)"><input type="number" style={inputStyle} value={newDeal.value} onChange={(e) => setNewDeal({ ...newDeal, value: e.target.value })} /></Field></div>
            <div style={{ flex: 1 }}><Field label="Expected Close"><input type="date" style={inputStyle} value={newDeal.expected_close_date} onChange={(e) => setNewDeal({ ...newDeal, expected_close_date: e.target.value })} /></Field></div>
          </div>
          <Field label="Stage">
            <select style={selectStyle} value={newDeal.stage} onChange={(e) => setNewDeal({ ...newDeal, stage: e.target.value })}>
              {PIPELINE_STAGES.map(s => <option key={s.key} value={s.key}>{s.label}</option>)}
            </select>
          </Field>
          <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 20 }}>
            <button onClick={() => setShowAddDeal(false)} style={btnSecondary}>Cancel</button>
            <button onClick={handleAddDeal} style={btnSubmit}>Add Deal</button>
          </div>
        </Modal>

        <Modal open={showLogActivity} onClose={() => setShowLogActivity(false)} title="Log Activity">
          <Field label="Type">
            <select style={selectStyle} value={newActivity.activity_type} onChange={(e) => setNewActivity({ ...newActivity, activity_type: e.target.value })}>
              <option value="call">Call</option>
              <option value="email">Email</option>
              <option value="meeting">Meeting</option>
              <option value="note">Note</option>
            </select>
          </Field>
          <Field label="Subject"><input style={inputStyle} value={newActivity.subject} onChange={(e) => setNewActivity({ ...newActivity, subject: e.target.value })} /></Field>
          <Field label="Notes"><textarea style={{ ...inputStyle, minHeight: 80, resize: "vertical" }} value={newActivity.notes} onChange={(e) => setNewActivity({ ...newActivity, notes: e.target.value })} /></Field>
          <Field label="Contact">
            <select style={selectStyle} value={newActivity.contact_id} onChange={(e) => setNewActivity({ ...newActivity, contact_id: e.target.value })}>
              <option value="">None</option>
              {contacts.map(c => <option key={c.id} value={c.id}>{c.name}</option>)}
            </select>
          </Field>
          <Field label="Deal">
            <select style={selectStyle} value={newActivity.deal_id} onChange={(e) => setNewActivity({ ...newActivity, deal_id: e.target.value })}>
              <option value="">None</option>
              {deals.map(d => <option key={d.id} value={d.id}>{d.name}</option>)}
            </select>
          </Field>
          <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 20 }}>
            <button onClick={() => setShowLogActivity(false)} style={btnSecondary}>Cancel</button>
            <button onClick={handleLogActivity} style={btnSubmit}>Log Activity</button>
          </div>
        </Modal>
      </div>
    </WorkspaceShell>
  );
}
