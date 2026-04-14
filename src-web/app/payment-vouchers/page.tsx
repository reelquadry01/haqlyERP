// Author: Quadri Atharu
"use client";

import { useEffect, useState, useCallback } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet, apiPost, apiPatch } from "@/lib/api";
import { FileText, Plus, X, CheckCircle, Printer, CreditCard, Banknote } from "lucide-react";

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

const PAYMENT_METHODS = ["Bank Transfer", "Cheque", "Cash"] as const;

interface PayeeOption {
  id: string;
  name: string;
  type: "vendor" | "employee";
}

interface BankAccount {
  id: string;
  account_name: string;
  account_number: string;
  bank_name: string;
}

interface PaymentVoucher {
  id: string;
  voucher_number: string;
  payee_name: string;
  amount: number;
  payment_method: string;
  bank_account: string;
  reference: string;
  narration: string;
  status: "draft" | "pending" | "approved" | "paid" | "cancelled";
  payment_date: string;
  approved_by: string | null;
  created_at: string;
}

const PV_STATUS_MAP: Record<string, { bg: string; color: string }> = {
  draft: { bg: "rgba(134,142,150,0.12)", color: T.muted },
  pending: { bg: "rgba(245,158,11,0.12)", color: "#F59E0B" },
  approved: { bg: "rgba(22,163,74,0.12)", color: T.success },
  paid: { bg: "rgba(212,175,55,0.12)", color: T.accent },
  cancelled: { bg: "rgba(220,38,38,0.12)", color: T.error },
};

function StatusPill({ status }: { status: string }) {
  const c = PV_STATUS_MAP[status] || PV_STATUS_MAP.draft;
  return (
    <span style={{ fontSize: "0.75rem", padding: "2px 10px", borderRadius: T.radiusSm, background: c.bg, color: c.color, fontWeight: 600 }}>
      {status.charAt(0).toUpperCase() + status.slice(1)}
    </span>
  );
}

function naira(v: number) {
  return `₦${(v || 0).toLocaleString("en-NG")}`;
}

function Overlay({ children, onClose }: { children: React.ReactNode; onClose: () => void }) {
  return (
    <div style={{ position: "fixed", inset: 0, zIndex: 1000, display: "flex", alignItems: "center", justifyContent: "center", background: "rgba(0,0,0,0.45)" }} onClick={onClose}>
      <div style={{ background: T.surface, borderRadius: T.radius, width: 560, maxHeight: "90vh", overflowY: "auto", boxShadow: "0 16px 48px rgba(0,0,0,0.18)" }} onClick={(e) => e.stopPropagation()}>
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

const btnAccent: React.CSSProperties = {
  ...btnPrimary, background: T.accent,
};

const btnDanger: React.CSSProperties = {
  ...btnPrimary, background: T.error,
};

const emptyVoucherForm = {
  payee_id: "", payee_name: "", amount: "", payment_method: PAYMENT_METHODS[0],
  bank_account_id: "", reference: "", narration: "", supporting_docs: "",
};

export default function PaymentVouchersPage() {
  const token = getToken();
  const [vouchers, setVouchers] = useState<PaymentVoucher[]>([]);
  const [payees, setPayees] = useState<PayeeOption[]>([]);
  const [bankAccounts, setBankAccounts] = useState<BankAccount[]>([]);
  const [tab, setTab] = useState<"all" | "draft" | "pending" | "approved" | "paid">("all");
  const [showCreate, setShowCreate] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [selectedVoucher, setSelectedVoucher] = useState<PaymentVoucher | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [voucherForm, setVoucherForm] = useState({ ...emptyVoucherForm });

  const loadVouchers = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await apiGet("/payment-vouchers", token);
      if (res.ok) setVouchers((await res.json()).data || []);
      else setError("Failed to load vouchers");
    } catch {
      setError("Network error");
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadVouchers(); }, [loadVouchers]);

  useEffect(() => {
    async function loadMeta() {
      try {
        const res = await apiGet("/payment-vouchers/metadata/options", token);
        if (res.ok) {
          const data = await res.json();
          setPayees(data.payees || []);
          setBankAccounts(data.bank_accounts || []);
        }
      } catch {
        // offline
      }
    }
    if (showCreate) loadMeta();
  }, [showCreate, token]);

  const filteredVouchers = tab === "all"
    ? vouchers
    : tab === "pending"
    ? vouchers.filter((v) => v.status === "pending")
    : vouchers.filter((v) => v.status === tab);

  async function handleCreateVoucher() {
    setSubmitting(true);
    try {
      const res = await apiPost("/payment-vouchers", {
        payee_id: voucherForm.payee_id,
        payee_name: voucherForm.payee_name,
        amount: Number(voucherForm.amount) || 0,
        payment_method: voucherForm.payment_method,
        bank_account_id: voucherForm.bank_account_id,
        reference: voucherForm.reference,
        narration: voucherForm.narration,
      }, token);
      if (res.ok) {
        setShowCreate(false);
        setVoucherForm({ ...emptyVoucherForm });
        loadVouchers();
      } else {
        setError("Failed to create voucher");
      }
    } catch {
      setError("Network error");
    } finally {
      setSubmitting(false);
    }
  }

  async function handleApprove(voucher: PaymentVoucher) {
    setSubmitting(true);
    try {
      const res = await apiPost(`/payment-vouchers/${voucher.id}/approve`, {}, token);
      if (res.ok) loadVouchers();
      else setError("Approval failed");
    } catch {
      setError("Network error");
    } finally {
      setSubmitting(false);
    }
  }

  async function handleMarkPaid(voucher: PaymentVoucher) {
    setSubmitting(true);
    try {
      const res = await apiPost(`/payment-vouchers/${voucher.id}/mark-paid`, {}, token);
      if (res.ok) {
        loadVouchers();
        setShowConfirm(false);
      } else {
        setError("Mark paid failed");
      }
    } catch {
      setError("Network error");
    } finally {
      setSubmitting(false);
    }
  }

  async function handleCancel(voucher: PaymentVoucher) {
    setSubmitting(true);
    try {
      const res = await apiPost(`/payment-vouchers/${voucher.id}/cancel`, {}, token);
      if (res.ok) loadVouchers();
      else setError("Cancel failed");
    } catch {
      setError("Network error");
    } finally {
      setSubmitting(false);
    }
  }

  async function handleSubmit(voucher: PaymentVoucher) {
    setSubmitting(true);
    try {
      const res = await apiPost(`/payment-vouchers/${voucher.id}/submit`, {}, token);
      if (res.ok) loadVouchers();
      else setError("Submit failed");
    } catch {
      setError("Network error");
    } finally {
      setSubmitting(false);
    }
  }

  function handlePrint(voucher: PaymentVoucher) {
    const content = `
      <html><head><title>PV ${voucher.voucher_number}</title>
      <style>body{font-family:Inter,sans-serif;padding:40px;color:#1A1A2E}h1{font-size:1.25rem}table{width:100%;border-collapse:collapse;margin-top:20px}td,th{padding:8px 12px;border:1px solid #E9ECEF;text-align:left;font-size:0.85rem}th{background:#F8F9FA;font-weight:600}.right{text-align:right}</style></head>
      <body><h1>Payment Voucher — ${voucher.voucher_number}</h1>
      <table><tr><th>Payee</th><td>${voucher.payee_name}</td><th>Amount</th><td class="right">${naira(voucher.amount)}</td></tr>
      <tr><th>Payment Method</th><td>${voucher.payment_method}</td><th>Date</th><td>${voucher.payment_date}</td></tr>
      <tr><th>Status</th><td>${voucher.status}</td><th>Reference</th><td>${voucher.reference || "—"}</td></tr>
      <tr><th>Narration</th><td colspan="3">${voucher.narration || "—"}</td></tr></table>
      <div style="margin-top:40px;display:flex;justify-content:space-between"><div>Prepared By: ________________</div><div>Approved By: ________________</div></div>
      </body></html>`;
    const win = window.open("", "_blank");
    if (win) { win.document.write(content); win.document.close(); win.print(); }
  }

  const columns: Column<PaymentVoucher>[] = [
    { key: "voucher_number", label: "PV #", sortable: true, width: "120px" },
    { key: "payee_name", label: "Payee", sortable: true },
    { key: "amount", label: "Amount", align: "right", sortable: true, width: "130px", render: (v: number) => naira(v) },
    { key: "payment_method", label: "Method", width: "120px" },
    { key: "status", label: "Status", width: "100px", render: (v: string) => <StatusPill status={v} /> },
    { key: "payment_date", label: "Date", sortable: true, width: "110px" },
    { key: "approved_by", label: "Approved By", width: "120px", render: (v: string | null) => v || "—" },
  ];

  const selectedBankAccount = bankAccounts.find((b) => b.id === selectedVoucher?.bank_account);

  return (
    <WorkspaceShell>
      <div style={{ padding: 24, background: T.bg, minHeight: "100vh" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: T.text, fontFamily: "'DM Serif', serif" }}>
            <FileText size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> Payment Vouchers
          </h1>
          <button onClick={() => setShowCreate(true)} style={btnPrimary}>
            <Plus size={16} /> Create Voucher
          </button>
        </div>

        {error && (
          <div style={{ padding: "10px 16px", marginBottom: 16, borderRadius: T.radiusSm, background: "rgba(220,38,38,0.08)", color: T.error, fontSize: "0.85rem" }}>{error}</div>
        )}

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: `2px solid ${T.border}` }}>
          {(["all", "draft", "pending", "approved", "paid"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{
              padding: "10px 16px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400,
              color: tab === t ? T.primary : T.muted,
              borderBottom: tab === t ? `2px solid ${T.primary}` : "2px solid transparent",
              marginBottom: -2, background: "none", borderLeft: "none", borderRight: "none",
              borderTop: "none", cursor: "pointer", fontFamily: "Inter, sans-serif",
              textTransform: "capitalize" as const,
            }}>
              {t}
            </button>
          ))}
        </div>

        <div style={{ background: T.surface, border: `1px solid ${T.border}`, borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.06)" }}>
          {loading ? (
            <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
          ) : (
            <DataTable
              columns={columns}
              data={filteredVouchers}
              pageSize={15}
              emptyMessage="No payment vouchers"
              actions={[
                { label: "Submit", icon: <CheckCircle size={14} />, onClick: (row) => handleSubmit(row), variant: "default" as const },
                { label: "Approve", icon: <CheckCircle size={14} />, onClick: (row) => handleApprove(row) },
                { label: "Pay", icon: <CreditCard size={14} />, onClick: (row) => { setSelectedVoucher(row); setShowConfirm(true); } },
                { label: "Print", icon: <Printer size={14} />, onClick: (row) => handlePrint(row) },
                { label: "Cancel", icon: <X size={14} />, onClick: (row) => handleCancel(row), variant: "danger" as const },
              ]}
            />
          )}
        </div>

        {showCreate && (
          <Overlay onClose={() => setShowCreate(false)}>
            <ModalHeader title="Create Payment Voucher" onClose={() => setShowCreate(false)} />
            <div style={{ padding: 24, display: "flex", flexDirection: "column", gap: 16 }}>
              <div>
                <label style={labelStyle}>Payee</label>
                <select style={inputStyle} value={voucherForm.payee_id} onChange={(e) => {
                  const p = payees.find((p) => p.id === e.target.value);
                  setVoucherForm({ ...voucherForm, payee_id: e.target.value, payee_name: p?.name || "" });
                }}>
                  <option value="">Select vendor/employee</option>
                  {payees.map((p) => <option key={p.id} value={p.id}>{p.name} ({p.type})</option>)}
                </select>
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
                <div>
                  <label style={labelStyle}>Amount (₦)</label>
                  <input type="number" style={inputStyle} value={voucherForm.amount} onChange={(e) => setVoucherForm({ ...voucherForm, amount: e.target.value })} placeholder="0" />
                </div>
                <div>
                  <label style={labelStyle}>Payment Method</label>
                  <select style={inputStyle} value={voucherForm.payment_method} onChange={(e) => setVoucherForm({ ...voucherForm, payment_method: e.target.value })}>
                    {PAYMENT_METHODS.map((m) => <option key={m} value={m}>{m}</option>)}
                  </select>
                </div>
              </div>
              <div>
                <label style={labelStyle}>Bank Account</label>
                <select style={inputStyle} value={voucherForm.bank_account_id} onChange={(e) => setVoucherForm({ ...voucherForm, bank_account_id: e.target.value })}>
                  <option value="">Select bank account</option>
                  {bankAccounts.map((b) => <option key={b.id} value={b.id}>{b.account_name} — {b.bank_name} ({b.account_number})</option>)}
                </select>
              </div>
              <div>
                <label style={labelStyle}>Reference</label>
                <input style={inputStyle} value={voucherForm.reference} onChange={(e) => setVoucherForm({ ...voucherForm, reference: e.target.value })} placeholder="INV-2024-001" />
              </div>
              <div>
                <label style={labelStyle}>Narration</label>
                <textarea style={{ ...inputStyle, minHeight: 60, resize: "vertical" }} value={voucherForm.narration} onChange={(e) => setVoucherForm({ ...voucherForm, narration: e.target.value })} placeholder="Payment for..." />
              </div>
              <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 8 }}>
                <button onClick={() => setShowCreate(false)} style={btnGhost}>Cancel</button>
                <button onClick={handleCreateVoucher} disabled={submitting || !voucherForm.payee_id || !voucherForm.amount} style={{ ...btnPrimary, opacity: submitting || !voucherForm.payee_id || !voucherForm.amount ? 0.6 : 1 }}>
                  {submitting ? "Creating..." : "Create Voucher"}
                </button>
              </div>
            </div>
          </Overlay>
        )}

        {showConfirm && selectedVoucher && (
          <Overlay onClose={() => setShowConfirm(false)}>
            <ModalHeader title={`Confirm Payment — ${selectedVoucher.voucher_number}`} onClose={() => setShowConfirm(false)} />
            <div style={{ padding: 24, display: "flex", flexDirection: "column", gap: 16 }}>
              <div style={{ padding: 16, borderRadius: T.radiusSm, background: T.bg, fontSize: "0.85rem", color: T.text }}>
                <div style={{ marginBottom: 8 }}><strong>Payee:</strong> {selectedVoucher.payee_name}</div>
                <div style={{ marginBottom: 8 }}><strong>Amount:</strong> <span style={{ color: T.primary, fontWeight: 600, fontSize: "1rem" }}>{naira(selectedVoucher.amount)}</span></div>
                <div style={{ marginBottom: 8 }}><strong>Method:</strong> {selectedVoucher.payment_method}</div>
                {selectedVoucher.payment_method === "Bank Transfer" && selectedBankAccount && (
                  <div style={{ padding: "10px 12px", borderRadius: T.radiusSm, background: T.surface, border: `1px solid ${T.border}`, marginTop: 8 }}>
                    <div style={{ fontWeight: 600, marginBottom: 4, display: "flex", alignItems: "center", gap: 6 }}>
                      <Banknote size={14} /> Bank Details for Transfer
                    </div>
                    <div>Bank: {selectedBankAccount.bank_name}</div>
                    <div>Account: {selectedBankAccount.account_name}</div>
                    <div>Account #: {selectedBankAccount.account_number}</div>
                  </div>
                )}
                {selectedVoucher.reference && (
                  <div style={{ marginTop: 8 }}><strong>Reference:</strong> {selectedVoucher.reference}</div>
                )}
              </div>
              <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 8 }}>
                <button onClick={() => handlePrint(selectedVoucher)} style={{ ...btnGhost, display: "inline-flex", alignItems: "center", gap: 6 }}>
                  <Printer size={14} /> Print Voucher
                </button>
                <button onClick={() => setShowConfirm(false)} style={btnGhost}>Cancel</button>
                <button onClick={() => handleMarkPaid(selectedVoucher)} disabled={submitting} style={{ ...btnAccent, opacity: submitting ? 0.6 : 1 }}>
                  {submitting ? "Processing..." : "Confirm Paid"}
                </button>
              </div>
            </div>
          </Overlay>
        )}
      </div>
    </WorkspaceShell>
  );
}
