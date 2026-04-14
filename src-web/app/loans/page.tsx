// Author: Quadri Atharu
"use client";

import { useEffect, useState, useCallback } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet, apiPost } from "@/lib/api";
import { Landmark, Plus, X, History, Calendar } from "lucide-react";

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

const LOAN_TYPES = ["Amortizing", "Interest-Only", "Bullet"] as const;
const PAYMENT_FREQUENCIES = ["Monthly", "Quarterly"] as const;

interface Loan {
  id: string;
  loan_code: string;
  lender_name: string;
  principal_amount: number;
  interest_rate: number;
  tenure_months: number;
  outstanding_principal: number;
  monthly_payment: number;
  start_date: string;
  next_due_date: string;
  maturity_date: string;
  loan_type: string;
  payment_frequency: string;
  status: string;
}

interface AmortizationEntry {
  period_number: number;
  payment_date: string;
  opening_balance: number;
  principal_payment: number;
  interest_payment: number;
  total_payment: number;
  closing_balance: number;
  is_paid: boolean;
}

interface PaymentRecord {
  id: string;
  loan_id: string;
  payment_date: string;
  amount: number;
  principal_portion: number;
  interest_portion: number;
  reference: string;
}

const LOAN_STATUS_MAP: Record<string, { bg: string; color: string }> = {
  active: { bg: "rgba(22,163,74,0.12)", color: T.success },
  disbursed: { bg: "rgba(59,130,246,0.12)", color: "#3B82F6" },
  completed: { bg: "rgba(134,142,150,0.12)", color: T.muted },
  defaulted: { bg: "rgba(220,38,38,0.12)", color: T.error },
};

function StatusPill({ status }: { status: string }) {
  const c = LOAN_STATUS_MAP[status] || LOAN_STATUS_MAP.active;
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

const emptyLoanForm = {
  lender_name: "", principal_amount: "", interest_rate: "", tenure_months: "",
  start_date: "", loan_type: LOAN_TYPES[0], payment_frequency: PAYMENT_FREQUENCIES[0],
};

export default function LoansPage() {
  const token = getToken();
  const [loans, setLoans] = useState<Loan[]>([]);
  const [amortization, setAmortization] = useState<AmortizationEntry[]>([]);
  const [payments, setPayments] = useState<PaymentRecord[]>([]);
  const [tab, setTab] = useState<"active" | "schedule" | "payments">("active");
  const [showNew, setShowNew] = useState(false);
  const [showPayment, setShowPayment] = useState(false);
  const [selectedLoan, setSelectedLoan] = useState<Loan | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [loanForm, setLoanForm] = useState({ ...emptyLoanForm });
  const [paymentForm, setPaymentForm] = useState({ amount: "", payment_date: "" });

  const loadLoans = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await apiGet("/loans", token);
      if (res.ok) setLoans((await res.json()).data || []);
      else setError("Failed to load loans");
    } catch {
      setError("Network error");
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadLoans(); }, [loadLoans]);

  const loadSchedule = useCallback(async () => {
    if (!selectedLoan) return;
    try {
      const res = await apiGet(`/loans/${selectedLoan.id}/schedule`, token);
      if (res.ok) setAmortization((await res.json()).data || []);
      else setAmortization([]);
    } catch {
      setAmortization([]);
    }
  }, [selectedLoan, token]);

  const loadPayments = useCallback(async () => {
    if (!selectedLoan) return;
    try {
      const res = await apiGet(`/loans/${selectedLoan.id}/payments`, token);
      if (res.ok) setPayments((await res.json()).data || []);
      else setPayments([]);
    } catch {
      setPayments([]);
    }
  }, [selectedLoan, token]);

  useEffect(() => {
    if (selectedLoan && tab === "schedule") loadSchedule();
  }, [selectedLoan, tab, loadSchedule]);

  useEffect(() => {
    if (selectedLoan && tab === "payments") loadPayments();
  }, [selectedLoan, tab, loadPayments]);

  function computePaymentSplit(amount: number) {
    if (!selectedLoan) return { principal: 0, interest: 0 };
    const outstanding = selectedLoan.outstanding_principal || 0;
    const rate = (selectedLoan.interest_rate || 0) / 100 / 12;
    const interestPortion = outstanding * rate;
    const principalPortion = Math.max(0, amount - interestPortion);
    return { principal: principalPortion, interest: interestPortion };
  }

  async function handleCreateLoan() {
    setSubmitting(true);
    try {
      const res = await apiPost("/loans", {
        ...loanForm,
        principal_amount: Number(loanForm.principal_amount) || 0,
        interest_rate: Number(loanForm.interest_rate) || 0,
        tenure_months: Number(loanForm.tenure_months) || 1,
      }, token);
      if (res.ok) {
        setShowNew(false);
        setLoanForm({ ...emptyLoanForm });
        loadLoans();
      } else {
        setError("Failed to create loan");
      }
    } catch {
      setError("Network error");
    } finally {
      setSubmitting(false);
    }
  }

  async function handleRecordPayment() {
    if (!selectedLoan) return;
    setSubmitting(true);
    try {
      const amount = Number(paymentForm.amount) || 0;
      const split = computePaymentSplit(amount);
      const res = await apiPost(`/loans/${selectedLoan.id}/repay`, {
        amount,
        payment_date: paymentForm.payment_date,
        principal_portion: split.principal,
        interest_portion: split.interest,
      }, token);
      if (res.ok) {
        setShowPayment(false);
        setPaymentForm({ amount: "", payment_date: "" });
        loadLoans();
        if (tab === "payments") loadPayments();
        if (tab === "schedule") loadSchedule();
      } else {
        setError("Payment failed");
      }
    } catch {
      setError("Network error");
    } finally {
      setSubmitting(false);
    }
  }

  const loanColumns: Column<Loan>[] = [
    { key: "loan_code", label: "Loan #", sortable: true, width: "110px" },
    { key: "lender_name", label: "Lender", sortable: true },
    { key: "principal_amount", label: "Principal", align: "right", width: "130px", render: (v: number) => naira(v) },
    { key: "interest_rate", label: "Rate", align: "right", width: "80px", render: (v: number) => `${v}%` },
    { key: "outstanding_principal", label: "Outstanding", align: "right", width: "130px", render: (v: number) => naira(v) },
    { key: "monthly_payment", label: "Monthly Pmt", align: "right", width: "120px", render: (v: number) => naira(v) },
    { key: "next_due_date", label: "Next Due", width: "110px" },
    { key: "maturity_date", label: "Maturity", width: "110px" },
    { key: "status", label: "Status", width: "100px", render: (v: string) => <StatusPill status={v} /> },
  ];

  const amortColumns: Column<AmortizationEntry>[] = [
    { key: "period_number", label: "Period", width: "70px" },
    { key: "payment_date", label: "Date", width: "110px" },
    { key: "opening_balance", label: "Opening Balance", align: "right", width: "130px", render: (v: number) => naira(v) },
    { key: "principal_payment", label: "Principal", align: "right", width: "120px", render: (v: number) => naira(v) },
    { key: "interest_payment", label: "Interest", align: "right", width: "120px", render: (v: number) => naira(v) },
    { key: "total_payment", label: "Total Payment", align: "right", width: "130px", render: (v: number) => naira(v) },
    { key: "closing_balance", label: "Closing Balance", align: "right", width: "130px", render: (v: number) => naira(v) },
  ];

  const paymentColumns: Column<PaymentRecord>[] = [
    { key: "payment_date", label: "Date", sortable: true, width: "110px" },
    { key: "amount", label: "Amount", align: "right", width: "120px", render: (v: number) => naira(v) },
    { key: "principal_portion", label: "Principal", align: "right", width: "120px", render: (v: number) => naira(v) },
    { key: "interest_portion", label: "Interest", align: "right", width: "120px", render: (v: number) => naira(v) },
    { key: "reference", label: "Reference", width: "140px" },
  ];

  const paymentSplit = paymentForm.amount ? computePaymentSplit(Number(paymentForm.amount) || 0) : null;

  return (
    <WorkspaceShell>
      <div style={{ padding: 24, background: T.bg, minHeight: "100vh" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: T.text, fontFamily: "'DM Serif', serif" }}>
            <Landmark size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> Loans
          </h1>
          <div style={{ display: "flex", gap: 8 }}>
            {selectedLoan && tab === "payments" && (
              <button onClick={() => setShowPayment(true)} style={{ ...btnGhost, borderColor: T.accent, color: T.accent }}>
                <Plus size={16} /> Record Payment
              </button>
            )}
            <button onClick={() => setShowNew(true)} style={btnPrimary}>
              <Plus size={16} /> New Loan
            </button>
          </div>
        </div>

        {error && (
          <div style={{ padding: "10px 16px", marginBottom: 16, borderRadius: T.radiusSm, background: "rgba(220,38,38,0.08)", color: T.error, fontSize: "0.85rem" }}>{error}</div>
        )}

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: `2px solid ${T.border}` }}>
          {(["active", "schedule", "payments"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{
              padding: "10px 20px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400,
              color: tab === t ? T.primary : T.muted,
              borderBottom: tab === t ? `2px solid ${T.primary}` : "2px solid transparent",
              marginBottom: -2, background: "none", borderLeft: "none", borderRight: "none",
              borderTop: "none", cursor: "pointer", fontFamily: "Inter, sans-serif",
              display: "inline-flex", alignItems: "center", gap: 6,
            }}>
              {t === "active" ? "Active Loans" : t === "schedule" ? "Amortization Schedule" : "Payment History"}
            </button>
          ))}
        </div>

        {selectedLoan && tab !== "active" && (
          <div style={{ marginBottom: 16, padding: "12px 16px", background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <div>
              <span style={{ fontSize: "0.9rem", fontWeight: 600, color: T.text }}>{selectedLoan.lender_name}</span>
              <span style={{ fontSize: "0.8rem", color: T.muted, marginLeft: 12 }}>{selectedLoan.loan_code}</span>
              <span style={{ fontSize: "0.8rem", color: T.primary, marginLeft: 12, fontWeight: 600 }}>Outstanding: {naira(selectedLoan.outstanding_principal)}</span>
            </div>
            <button onClick={() => { setSelectedLoan(null); setTab("active"); }} style={{ ...btnGhost, fontSize: "0.8rem", padding: "6px 14px" }}>Back to Loans</button>
          </div>
        )}

        <div style={{ background: T.surface, border: `1px solid ${T.border}`, borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.06)" }}>
          {loading ? (
            <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
          ) : tab === "active" ? (
            <DataTable
              columns={loanColumns}
              data={loans}
              pageSize={15}
              emptyMessage="No active loans"
              onRowClick={(row) => { setSelectedLoan(row); setTab("schedule"); }}
              actions={[
                { label: "Schedule", icon: <Calendar size={14} />, onClick: (row) => { setSelectedLoan(row); setTab("schedule"); } },
                { label: "Payments", icon: <History size={14} />, onClick: (row) => { setSelectedLoan(row); setTab("payments"); } },
              ]}
            />
          ) : tab === "schedule" ? (
            selectedLoan ? (
              <DataTable columns={amortColumns} data={amortization} pageSize={12} emptyMessage="No amortization data" />
            ) : (
              <div style={{ textAlign: "center", padding: 40, color: T.muted, fontSize: "0.9rem" }}>Select a loan to view amortization schedule</div>
            )
          ) : (
            selectedLoan ? (
              <DataTable columns={paymentColumns} data={payments} pageSize={12} emptyMessage="No payment records" />
            ) : (
              <div style={{ textAlign: "center", padding: 40, color: T.muted, fontSize: "0.9rem" }}>Select a loan to view payment history</div>
            )
          )}
        </div>

        {showNew && (
          <Overlay onClose={() => setShowNew(false)}>
            <ModalHeader title="New Loan" onClose={() => setShowNew(false)} />
            <div style={{ padding: 24, display: "flex", flexDirection: "column", gap: 16 }}>
              <div>
                <label style={labelStyle}>Lender Name</label>
                <input style={inputStyle} value={loanForm.lender_name} onChange={(e) => setLoanForm({ ...loanForm, lender_name: e.target.value })} placeholder="e.g. First Bank Nigeria" />
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
                <div>
                  <label style={labelStyle}>Principal Amount (₦)</label>
                  <input type="number" style={inputStyle} value={loanForm.principal_amount} onChange={(e) => setLoanForm({ ...loanForm, principal_amount: e.target.value })} placeholder="0" />
                </div>
                <div>
                  <label style={labelStyle}>Interest Rate (%)</label>
                  <input type="number" step="0.01" style={inputStyle} value={loanForm.interest_rate} onChange={(e) => setLoanForm({ ...loanForm, interest_rate: e.target.value })} placeholder="12.5" />
                </div>
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
                <div>
                  <label style={labelStyle}>Term (Months)</label>
                  <input type="number" style={inputStyle} value={loanForm.tenure_months} onChange={(e) => setLoanForm({ ...loanForm, tenure_months: e.target.value })} placeholder="36" />
                </div>
                <div>
                  <label style={labelStyle}>Start Date</label>
                  <input type="date" style={inputStyle} value={loanForm.start_date} onChange={(e) => setLoanForm({ ...loanForm, start_date: e.target.value })} />
                </div>
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
                <div>
                  <label style={labelStyle}>Loan Type</label>
                  <select style={inputStyle} value={loanForm.loan_type} onChange={(e) => setLoanForm({ ...loanForm, loan_type: e.target.value })}>
                    {LOAN_TYPES.map((t) => <option key={t} value={t}>{t}</option>)}
                  </select>
                </div>
                <div>
                  <label style={labelStyle}>Payment Frequency</label>
                  <select style={inputStyle} value={loanForm.payment_frequency} onChange={(e) => setLoanForm({ ...loanForm, payment_frequency: e.target.value })}>
                    {PAYMENT_FREQUENCIES.map((f) => <option key={f} value={f}>{f}</option>)}
                  </select>
                </div>
              </div>
              <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 8 }}>
                <button onClick={() => setShowNew(false)} style={btnGhost}>Cancel</button>
                <button onClick={handleCreateLoan} disabled={submitting || !loanForm.lender_name} style={{ ...btnPrimary, opacity: submitting || !loanForm.lender_name ? 0.6 : 1 }}>
                  {submitting ? "Creating..." : "Create Loan"}
                </button>
              </div>
            </div>
          </Overlay>
        )}

        {showPayment && selectedLoan && (
          <Overlay onClose={() => setShowPayment(false)}>
            <ModalHeader title={`Record Payment — ${selectedLoan.lender_name}`} onClose={() => setShowPayment(false)} />
            <div style={{ padding: 24, display: "flex", flexDirection: "column", gap: 16 }}>
              <div style={{ padding: 12, borderRadius: T.radiusSm, background: T.bg, fontSize: "0.85rem", color: T.text }}>
                <strong>Outstanding:</strong> <span style={{ color: T.primary, fontWeight: 600 }}>{naira(selectedLoan.outstanding_principal)}</span>
                <span style={{ marginLeft: 16 }}><strong>Rate:</strong> {selectedLoan.interest_rate}%</span>
              </div>
              <div>
                <label style={labelStyle}>Payment Amount (₦)</label>
                <input type="number" style={inputStyle} value={paymentForm.amount} onChange={(e) => setPaymentForm({ ...paymentForm, amount: e.target.value })} placeholder="0" />
              </div>
              <div>
                <label style={labelStyle}>Payment Date</label>
                <input type="date" style={inputStyle} value={paymentForm.payment_date} onChange={(e) => setPaymentForm({ ...paymentForm, payment_date: e.target.value })} />
              </div>
              {paymentSplit && Number(paymentForm.amount) > 0 && (
                <div style={{ padding: 12, borderRadius: T.radiusSm, background: T.bg, fontSize: "0.85rem", color: T.text }}>
                  <div><strong>Principal Portion:</strong> <span style={{ color: T.primary }}>{naira(paymentSplit.principal)}</span></div>
                  <div style={{ marginTop: 4 }}><strong>Interest Portion:</strong> <span style={{ color: T.accent }}>{naira(paymentSplit.interest)}</span></div>
                </div>
              )}
              <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 8 }}>
                <button onClick={() => setShowPayment(false)} style={btnGhost}>Cancel</button>
                <button onClick={handleRecordPayment} disabled={submitting || !paymentForm.amount} style={{ ...btnPrimary, opacity: submitting || !paymentForm.amount ? 0.6 : 1 }}>
                  {submitting ? "Recording..." : "Record Payment"}
                </button>
              </div>
            </div>
          </Overlay>
        )}
      </div>
    </WorkspaceShell>
  );
}
