// Author: Quadri Atharu
"use client";

import { useEffect, useState, useCallback } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { getToken } from "@/lib/session";
import { apiGet, apiPost, apiPatch } from "@/lib/api";
import {
  ShoppingCart, Plus, X, ChevronDown, ChevronUp, Send, CreditCard,
  Ban, FileText, RefreshCw, AlertTriangle, DollarSign, Clock, TrendingUp
} from "lucide-react";

interface LineItem {
  product_name: string;
  quantity: number;
  unit_price: number;
  vat_rate: number;
  line_total: number;
}

interface Invoice {
  id: string;
  invoice_number: string;
  customer_id: string;
  customer_name: string;
  invoice_date: string;
  due_date: string;
  subtotal: number;
  vat_total: number;
  total_amount: number;
  status: string;
  line_items: LineItem[];
}

interface Customer {
  id: string;
  name: string;
  code: string;
  email: string;
  outstanding_balance: number;
}

interface AgingRow {
  customer_name: string;
  current: number;
  days_30: number;
  days_60: number;
  days_90_plus: number;
  total: number;
}

interface NewLineItem {
  product_name: string;
  quantity: number;
  unit_price: number;
}

const STATUS_STYLES: Record<string, { bg: string; color: string; textDecoration?: string }> = {
  draft: { bg: "rgba(107,114,128,0.12)", color: "#6B7280" },
  sent: { bg: "rgba(59,130,246,0.12)", color: "#2563EB" },
  paid: { bg: "rgba(22,163,74,0.12)", color: "#16A34A" },
  overdue: { bg: "rgba(220,38,38,0.12)", color: "#DC2626" },
  cancelled: { bg: "rgba(107,114,128,0.12)", color: "#6B7280", textDecoration: "line-through" },
};

const VAT_RATE = 0.075;

function fmt(v: number): string {
  return `₦${(v || 0).toLocaleString("en-NG", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

const INPUT_STYLE: React.CSSProperties = {
  width: "100%", padding: "8px 12px", fontSize: "0.85rem",
  border: "1px solid #D1D5DB", borderRadius: 6, background: "#FFFFFF", color: "#1A1A2E", outline: "none",
};

export default function SalesPage() {
  const token = getToken();
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [aging, setAging] = useState<AgingRow[]>([]);
  const [tab, setTab] = useState<"overview" | "invoices" | "aging">("overview");
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [showCreateInvoice, setShowCreateInvoice] = useState(false);
  const [showRecordPayment, setShowRecordPayment] = useState(false);
  const [selectedInvoice, setSelectedInvoice] = useState<Invoice | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [newInvoice, setNewInvoice] = useState({
    customer_id: "",
    invoice_date: new Date().toISOString().slice(0, 10),
    due_date: "",
    line_items: [{ product_name: "", quantity: 1, unit_price: 0 }] as NewLineItem[],
  });
  const [paymentAmount, setPaymentAmount] = useState("");
  const [paymentDate, setPaymentDate] = useState(new Date().toISOString().slice(0, 10));
  const [paymentMethod, setPaymentMethod] = useState("bank_transfer");
  const [actionMenuId, setActionMenuId] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [invRes, custRes, agingRes] = await Promise.all([
        apiGet("/sales/invoices", token),
        apiGet("/sales/customers", token),
        apiGet("/sales/invoices/aging", token),
      ]);
      if (!invRes.ok) throw new Error("Failed to load invoices");
      if (invRes.ok) setInvoices((await invRes.json()).data || []);
      if (custRes.ok) setCustomers((await custRes.json()).data || []);
      if (agingRes.ok) setAging((await agingRes.json()).data || []);
    } catch (e: any) {
      setError(e.message || "Failed to load data");
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadData(); }, [loadData]);

  async function handleSendInvoice(inv: Invoice) {
    try {
      const res = await apiPatch(`/sales/invoices/${inv.id}`, { status: "sent" }, token);
      if (res.ok) loadData();
    } catch {}
  }

  async function handleCancelInvoice(inv: Invoice) {
    try {
      const res = await apiPatch(`/sales/invoices/${inv.id}`, { status: "cancelled" }, token);
      if (res.ok) loadData();
    } catch {}
  }

  async function handleRecordPayment() {
    if (!selectedInvoice || !paymentAmount) return;
    try {
      const res = await apiPost("/sales/receipts", {
        invoice_id: selectedInvoice.id,
        amount: parseFloat(paymentAmount),
        payment_date: paymentDate,
        payment_method: paymentMethod,
      }, token);
      if (res.ok) {
        setShowRecordPayment(false);
        setSelectedInvoice(null);
        setPaymentAmount("");
        loadData();
      }
    } catch {}
  }

  async function handleCreateCreditNote(inv: Invoice) {
    try {
      const res = await apiPost("/sales/invoices", {
        type: "credit_note",
        reference_invoice_id: inv.id,
        customer_id: inv.customer_id,
        line_items: inv.line_items.map(li => ({ ...li, quantity: -li.quantity })),
      }, token);
      if (res.ok) loadData();
    } catch {}
  }

  function calcTotals(items: NewLineItem[]) {
    const subtotal = items.reduce((s, li) => s + li.quantity * li.unit_price, 0);
    const vat = subtotal * VAT_RATE;
    return { subtotal, vat, total: subtotal + vat };
  }

  async function handleCreateInvoice() {
    const t = calcTotals(newInvoice.line_items);
    try {
      const res = await apiPost("/sales/invoices", {
        customer_id: newInvoice.customer_id,
        invoice_date: newInvoice.invoice_date,
        due_date: newInvoice.due_date,
        line_items: newInvoice.line_items.map(li => ({
          product_name: li.product_name,
          quantity: li.quantity,
          unit_price: li.unit_price,
          vat_rate: VAT_RATE,
          line_total: li.quantity * li.unit_price * (1 + VAT_RATE),
        })),
        subtotal: t.subtotal,
        vat_total: t.vat,
        total_amount: t.total,
      }, token);
      if (res.ok) {
        setShowCreateInvoice(false);
        setNewInvoice({
          customer_id: "",
          invoice_date: new Date().toISOString().slice(0, 10),
          due_date: "",
          line_items: [{ product_name: "", quantity: 1, unit_price: 0 }],
        });
        loadData();
      }
    } catch {}
  }

  const monthKey = new Date().toISOString().slice(0, 7);
  const revenueThisMonth = invoices
    .filter(i => i.status === "paid" && i.invoice_date.startsWith(monthKey))
    .reduce((s, i) => s + i.total_amount, 0);
  const outstandingInvoices = invoices
    .filter(i => i.status === "sent")
    .reduce((s, i) => s + i.total_amount, 0);
  const overduePayments = invoices
    .filter(i => i.status === "overdue")
    .reduce((s, i) => s + i.total_amount, 0);
  const newTotals = calcTotals(newInvoice.line_items);
  const agingTotals = aging.reduce(
    (acc, r) => ({
      current: acc.current + r.current,
      days_30: acc.days_30 + r.days_30,
      days_60: acc.days_60 + r.days_60,
      days_90_plus: acc.days_90_plus + r.days_90_plus,
      total: acc.total + r.total,
    }),
    { current: 0, days_30: 0, days_60: 0, days_90_plus: 0, total: 0 }
  );

  if (error && !loading) {
    return (
      <WorkspaceShell>
        <div style={{ padding: 24, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", minHeight: "60vh" }}>
          <AlertTriangle size={40} style={{ color: "#DC2626", marginBottom: 16 }} />
          <p style={{ fontSize: "1rem", color: "#1A1A2E", marginBottom: 8 }}>{error}</p>
          <button onClick={loadData} style={{ display: "flex", alignItems: "center", gap: 6, background: "#1B4332", color: "#FFFFFF", padding: "8px 16px", borderRadius: 6, fontSize: "0.85rem" }}>
            <RefreshCw size={14} /> Retry
          </button>
        </div>
      </WorkspaceShell>
    );
  }

  return (
    <WorkspaceShell>
      <div style={{ padding: 24, background: "#F8F9FA", minHeight: "100%" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20, flexWrap: "wrap", gap: 12 }}>
          <h1 style={{ fontSize: "1.5rem", fontFamily: "'DM Serif Display', serif", color: "#1A1A2E", display: "flex", alignItems: "center", gap: 8 }}>
            <ShoppingCart size={22} style={{ color: "#1B4332" }} /> Sales
          </h1>
          <button onClick={() => setShowCreateInvoice(true)} style={{ display: "flex", alignItems: "center", gap: 6, background: "#1B4332", color: "#FFFFFF", padding: "8px 16px", borderRadius: 6, fontSize: "0.85rem", fontWeight: 500 }}>
            <Plus size={16} /> New Invoice
          </button>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: "2px solid #E9ECEF" }}>
          {(["overview", "invoices", "aging"] as const).map(t => (
            <button key={t} onClick={() => setTab(t)} style={{
              padding: "10px 16px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400,
              color: tab === t ? "#1B4332" : "#6B7280",
              borderBottom: tab === t ? "2px solid #1B4332" : "2px solid transparent", marginBottom: -2, background: "none",
            }}>
              {t === "aging" ? "Aging Report" : t.charAt(0).toUpperCase() + t.slice(1)}
            </button>
          ))}
        </div>

        {loading ? (
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", gap: 16 }}>
            {[1, 2, 3].map(i => (
              <div key={i} style={{ background: "#FFFFFF", borderRadius: 8, padding: 20, border: "1px solid #E9ECEF" }}>
                <div style={{ width: "60%", height: 12, background: "#E9ECEF", borderRadius: 4, marginBottom: 12 }} />
                <div style={{ width: "40%", height: 24, background: "#E9ECEF", borderRadius: 4, marginBottom: 8 }} />
                <div style={{ width: "30%", height: 12, background: "#E9ECEF", borderRadius: 4 }} />
              </div>
            ))}
          </div>
        ) : tab === "overview" ? (
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))", gap: 16, marginBottom: 24 }}>
            <div style={{ background: "#FFFFFF", borderRadius: 8, padding: 20, border: "1px solid #E9ECEF" }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: 12 }}>
                <span style={{ fontSize: "0.8rem", color: "#6B7280", fontWeight: 500 }}>Revenue This Month</span>
                <div style={{ width: 32, height: 32, borderRadius: 8, background: "rgba(27,67,50,0.12)", display: "flex", alignItems: "center", justifyContent: "center" }}>
                  <TrendingUp size={16} style={{ color: "#1B4332" }} />
                </div>
              </div>
              <div style={{ fontSize: "1.5rem", fontWeight: 700, color: "#1A1A2E", marginBottom: 4 }}>{fmt(revenueThisMonth)}</div>
              <span style={{ fontSize: "0.8rem", color: "#16A34A" }}>Paid invoices</span>
            </div>
            <div style={{ background: "#FFFFFF", borderRadius: 8, padding: 20, border: "1px solid #E9ECEF" }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: 12 }}>
                <span style={{ fontSize: "0.8rem", color: "#6B7280", fontWeight: 500 }}>Outstanding Invoices</span>
                <div style={{ width: 32, height: 32, borderRadius: 8, background: "rgba(212,175,55,0.12)", display: "flex", alignItems: "center", justifyContent: "center" }}>
                  <DollarSign size={16} style={{ color: "#D4AF37" }} />
                </div>
              </div>
              <div style={{ fontSize: "1.5rem", fontWeight: 700, color: "#1A1A2E", marginBottom: 4 }}>{fmt(outstandingInvoices)}</div>
              <span style={{ fontSize: "0.8rem", color: "#D4AF37" }}>Sent invoices</span>
            </div>
            <div style={{ background: "#FFFFFF", borderRadius: 8, padding: 20, border: "1px solid #E9ECEF" }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: 12 }}>
                <span style={{ fontSize: "0.8rem", color: "#6B7280", fontWeight: 500 }}>Overdue Payments</span>
                <div style={{ width: 32, height: 32, borderRadius: 8, background: "rgba(220,38,38,0.12)", display: "flex", alignItems: "center", justifyContent: "center" }}>
                  <Clock size={16} style={{ color: "#DC2626" }} />
                </div>
              </div>
              <div style={{ fontSize: "1.5rem", fontWeight: 700, color: "#1A1A2E", marginBottom: 4 }}>{fmt(overduePayments)}</div>
              <span style={{ fontSize: "0.8rem", color: "#DC2626" }}>Past due date</span>
            </div>
          </div>
        ) : tab === "invoices" ? (
          <div style={{ background: "#FFFFFF", borderRadius: 8, border: "1px solid #E9ECEF", overflow: "hidden" }}>
            <div style={{ overflowX: "auto" }}>
              <table style={{ width: "100%", borderCollapse: "collapse" }}>
                <thead>
                  <tr>
                    {["Invoice #", "Customer", "Amount", "Status", "Date", "Due Date", "Actions"].map(h => (
                      <th key={h} style={{ padding: "12px 14px", textAlign: h === "Amount" ? "right" : h === "Actions" ? "center" : "left", fontWeight: 600, fontSize: "0.8rem", color: "#6B7280", textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: "2px solid #E9ECEF", background: "#F8F9FA" }}>
                        {h}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {invoices.length === 0 ? (
                    <tr>
                      <td colSpan={7} style={{ textAlign: "center", padding: 32, color: "#6B7280" }}>No invoices found</td>
                    </tr>
                  ) : invoices.map(inv => {
                    const isExpanded = expandedId === inv.id;
                    const ss = STATUS_STYLES[inv.status] || STATUS_STYLES.draft;
                    return (
                      <React.Fragment key={inv.id}>
                        <tr
                          onClick={() => setExpandedId(isExpanded ? null : inv.id)}
                          style={{ cursor: "pointer", transition: "background 0.1s" }}
                          onMouseEnter={e => (e.currentTarget.style.background = "#F8F9FA")}
                          onMouseLeave={e => (e.currentTarget.style.background = "transparent")}
                        >
                          <td style={{ padding: "12px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", display: "flex", alignItems: "center", gap: 6 }}>
                            {isExpanded ? <ChevronUp size={14} style={{ color: "#1B4332" }} /> : <ChevronDown size={14} style={{ color: "#6B7280" }} />}
                            {inv.invoice_number}
                          </td>
                          <td style={{ padding: "12px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", color: "#1A1A2E" }}>{inv.customer_name}</td>
                          <td style={{ padding: "12px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", textAlign: "right", fontWeight: 600, color: "#1A1A2E" }}>{fmt(inv.total_amount)}</td>
                          <td style={{ padding: "12px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF" }}>
                            <span style={{ fontSize: "0.75rem", fontWeight: 500, padding: "2px 8px", borderRadius: 4, background: ss.bg, color: ss.color, textDecoration: ss.textDecoration || "none" }}>
                              {inv.status.charAt(0).toUpperCase() + inv.status.slice(1)}
                            </span>
                          </td>
                          <td style={{ padding: "12px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", color: "#6B7280" }}>{inv.invoice_date}</td>
                          <td style={{ padding: "12px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", color: "#6B7280" }}>{inv.due_date}</td>
                          <td style={{ padding: "12px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", textAlign: "center" }}>
                            <div style={{ position: "relative", display: "inline-block" }}>
                              <button onClick={e => { e.stopPropagation(); setActionMenuId(actionMenuId === inv.id ? null : inv.id); }} style={{ padding: "4px 8px", borderRadius: 4, background: "#F8F9FA", border: "1px solid #E9ECEF", fontSize: "0.8rem", color: "#1A1A2E" }}>
                                Actions
                              </button>
                              {actionMenuId === inv.id && (
                                <div style={{ position: "absolute", right: 0, top: "100%", marginTop: 4, background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 8, minWidth: 180, boxShadow: "0 8px 24px rgba(0,0,0,0.12)", zIndex: 50 }}>
                                  {inv.status === "draft" && (
                                    <button onClick={e => { e.stopPropagation(); handleSendInvoice(inv); setActionMenuId(null); }} style={{ display: "flex", alignItems: "center", gap: 8, width: "100%", padding: "8px 12px", fontSize: "0.85rem", color: "#2563EB", background: "none" }}>
                                      <Send size={14} /> Send Invoice
                                    </button>
                                  )}
                                  {(inv.status === "sent" || inv.status === "overdue") && (
                                    <button onClick={e => { e.stopPropagation(); setSelectedInvoice(inv); setShowRecordPayment(true); setActionMenuId(null); }} style={{ display: "flex", alignItems: "center", gap: 8, width: "100%", padding: "8px 12px", fontSize: "0.85rem", color: "#16A34A", background: "none" }}>
                                      <CreditCard size={14} /> Record Payment
                                    </button>
                                  )}
                                  {(inv.status === "sent" || inv.status === "overdue") && (
                                    <button onClick={e => { e.stopPropagation(); handleCreateCreditNote(inv); setActionMenuId(null); }} style={{ display: "flex", alignItems: "center", gap: 8, width: "100%", padding: "8px 12px", fontSize: "0.85rem", color: "#D4AF37", background: "none" }}>
                                      <FileText size={14} /> Create Credit Note
                                    </button>
                                  )}
                                  {inv.status !== "paid" && inv.status !== "cancelled" && (
                                    <button onClick={e => { e.stopPropagation(); handleCancelInvoice(inv); setActionMenuId(null); }} style={{ display: "flex", alignItems: "center", gap: 8, width: "100%", padding: "8px 12px", fontSize: "0.85rem", color: "#DC2626", background: "none" }}>
                                      <Ban size={14} /> Cancel Invoice
                                    </button>
                                  )}
                                </div>
                              )}
                            </div>
                          </td>
                        </tr>
                        {isExpanded && inv.line_items && inv.line_items.length > 0 && (
                          <tr>
                            <td colSpan={7} style={{ padding: "4px 14px 14px 44px", background: "#F8F9FA", borderBottom: "1px solid #E9ECEF" }}>
                              <table style={{ width: "100%", fontSize: "0.85rem", borderCollapse: "collapse" }}>
                                <thead>
                                  <tr>
                                    {["Product", "Qty", "Unit Price", "VAT (7.5%)", "Line Total"].map(h => (
                                      <th key={h} style={{ padding: "6px 10px", textAlign: h === "Qty" || h === "Unit Price" || h === "VAT (7.5%)" || h === "Line Total" ? "right" : "left", fontWeight: 600, fontSize: "0.75rem", color: "#6B7280", borderBottom: "1px solid #E9ECEF" }}>
                                        {h}
                                      </th>
                                    ))}
                                  </tr>
                                </thead>
                                <tbody>
                                  {inv.line_items.map((li, j) => (
                                    <tr key={j}>
                                      <td style={{ padding: "6px 10px", color: "#1A1A2E" }}>{li.product_name}</td>
                                      <td style={{ padding: "6px 10px", textAlign: "right", color: "#1A1A2E" }}>{li.quantity}</td>
                                      <td style={{ padding: "6px 10px", textAlign: "right", color: "#1A1A2E" }}>{fmt(li.unit_price)}</td>
                                      <td style={{ padding: "6px 10px", textAlign: "right", color: "#6B7280" }}>{fmt(li.unit_price * li.vat_rate * li.quantity)}</td>
                                      <td style={{ padding: "6px 10px", textAlign: "right", fontWeight: 600, color: "#1A1A2E" }}>{fmt(li.line_total)}</td>
                                    </tr>
                                  ))}
                                </tbody>
                                <tfoot>
                                  <tr style={{ borderTop: "2px solid #E9ECEF" }}>
                                    <td colSpan={3} />
                                    <td style={{ padding: "6px 10px", textAlign: "right", fontSize: "0.8rem", color: "#6B7280" }}>Subtotal: {fmt(inv.subtotal)}</td>
                                    <td style={{ padding: "6px 10px", textAlign: "right", fontSize: "0.8rem", color: "#6B7280" }}>VAT: {fmt(inv.vat_total)}</td>
                                  </tr>
                                  <tr>
                                    <td colSpan={4} />
                                    <td style={{ padding: "6px 10px", textAlign: "right", fontWeight: 700, color: "#1B4332" }}>{fmt(inv.total_amount)}</td>
                                  </tr>
                                </tfoot>
                              </table>
                            </td>
                          </tr>
                        )}
                      </React.Fragment>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>
        ) : (
          <div style={{ background: "#FFFFFF", borderRadius: 8, border: "1px solid #E9ECEF", overflow: "hidden" }}>
            <div style={{ padding: 16, borderBottom: "1px solid #E9ECEF" }}>
              <h3 style={{ fontSize: "1rem", fontWeight: 600, color: "#1A1A2E", marginBottom: 4 }}>Customer Receivables Aging</h3>
              <p style={{ fontSize: "0.8rem", color: "#6B7280" }}>Outstanding receivables grouped by aging period</p>
            </div>
            <div style={{ overflowX: "auto" }}>
              <table style={{ width: "100%", borderCollapse: "collapse" }}>
                <thead>
                  <tr>
                    {["Customer", "Current", "1-30 Days", "31-60 Days", "90+ Days", "Total"].map(h => (
                      <th key={h} style={{ padding: "10px 14px", textAlign: h === "Customer" ? "left" : "right", fontWeight: 600, fontSize: "0.8rem", color: "#6B7280", textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: "2px solid #E9ECEF", background: "#F8F9FA" }}>
                        {h}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {aging.length === 0 ? (
                    <tr><td colSpan={6} style={{ textAlign: "center", padding: 32, color: "#6B7280" }}>No outstanding receivables</td></tr>
                  ) : aging.map((r, i) => (
                    <tr key={i} style={{ transition: "background 0.1s" }} onMouseEnter={e => (e.currentTarget.style.background = "#F8F9FA")} onMouseLeave={e => (e.currentTarget.style.background = "transparent")}>
                      <td style={{ padding: "10px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", color: "#1A1A2E", fontWeight: 500 }}>{r.customer_name}</td>
                      <td style={{ padding: "10px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", textAlign: "right" }}>{fmt(r.current)}</td>
                      <td style={{ padding: "10px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", textAlign: "right", color: "#D4AF37" }}>{fmt(r.days_30)}</td>
                      <td style={{ padding: "10px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", textAlign: "right", color: "#F59E0B" }}>{fmt(r.days_60)}</td>
                      <td style={{ padding: "10px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", textAlign: "right", color: "#DC2626" }}>{fmt(r.days_90_plus)}</td>
                      <td style={{ padding: "10px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", textAlign: "right", fontWeight: 700, color: "#1A1A2E" }}>{fmt(r.total)}</td>
                    </tr>
                  ))}
                </tbody>
                {aging.length > 0 && (
                  <tfoot>
                    <tr style={{ background: "#F8F9FA" }}>
                      <td style={{ padding: "10px 14px", fontSize: "0.875rem", fontWeight: 700, color: "#1A1A2E" }}>Total</td>
                      <td style={{ padding: "10px 14px", fontSize: "0.875rem", textAlign: "right", fontWeight: 700 }}>{fmt(agingTotals.current)}</td>
                      <td style={{ padding: "10px 14px", fontSize: "0.875rem", textAlign: "right", fontWeight: 700, color: "#D4AF37" }}>{fmt(agingTotals.days_30)}</td>
                      <td style={{ padding: "10px 14px", fontSize: "0.875rem", textAlign: "right", fontWeight: 700, color: "#F59E0B" }}>{fmt(agingTotals.days_60)}</td>
                      <td style={{ padding: "10px 14px", fontSize: "0.875rem", textAlign: "right", fontWeight: 700, color: "#DC2626" }}>{fmt(agingTotals.days_90_plus)}</td>
                      <td style={{ padding: "10px 14px", fontSize: "0.875rem", textAlign: "right", fontWeight: 700, color: "#1B4332" }}>{fmt(agingTotals.total)}</td>
                    </tr>
                  </tfoot>
                )}
              </table>
            </div>
          </div>
        )}

        {showCreateInvoice && (
          <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }} onClick={() => setShowCreateInvoice(false)}>
            <div style={{ background: "#FFFFFF", borderRadius: 12, maxWidth: 720, width: "90%", maxHeight: "90vh", overflowY: "auto", padding: 24 }} onClick={e => e.stopPropagation()}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
                <h2 style={{ fontSize: "1.25rem", fontFamily: "'DM Serif Display', serif", color: "#1A1A2E" }}>Create Invoice</h2>
                <button onClick={() => setShowCreateInvoice(false)} style={{ color: "#6B7280" }}><X size={20} /></button>
              </div>

              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, marginBottom: 16 }}>
                <div>
                  <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#6B7280", marginBottom: 4 }}>Customer</label>
                  <select value={newInvoice.customer_id} onChange={e => setNewInvoice({ ...newInvoice, customer_id: e.target.value })} style={{ ...INPUT_STYLE, appearance: "auto" }}>
                    <option value="">Select customer</option>
                    {customers.map(c => <option key={c.id} value={c.id}>{c.name}</option>)}
                  </select>
                </div>
                <div>
                  <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#6B7280", marginBottom: 4 }}>Invoice Date</label>
                  <input type="date" value={newInvoice.invoice_date} onChange={e => setNewInvoice({ ...newInvoice, invoice_date: e.target.value })} style={INPUT_STYLE} />
                </div>
                <div>
                  <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#6B7280", marginBottom: 4 }}>Due Date</label>
                  <input type="date" value={newInvoice.due_date} onChange={e => setNewInvoice({ ...newInvoice, due_date: e.target.value })} style={INPUT_STYLE} />
                </div>
              </div>

              <div style={{ marginBottom: 12 }}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
                  <span style={{ fontSize: "0.85rem", fontWeight: 600, color: "#1A1A2E" }}>Line Items</span>
                  <button onClick={() => setNewInvoice({ ...newInvoice, line_items: [...newInvoice.line_items, { product_name: "", quantity: 1, unit_price: 0 }] })} style={{ display: "flex", alignItems: "center", gap: 4, fontSize: "0.8rem", color: "#1B4332", fontWeight: 500 }}>
                    <Plus size={14} /> Add Line
                  </button>
                </div>
                {newInvoice.line_items.map((li, idx) => (
                  <div key={idx} style={{ display: "grid", gridTemplateColumns: "2fr 1fr 1fr auto", gap: 8, marginBottom: 8, alignItems: "center" }}>
                    <input placeholder="Product name" value={li.product_name} onChange={e => {
                      const items = [...newInvoice.line_items];
                      items[idx] = { ...items[idx], product_name: e.target.value };
                      setNewInvoice({ ...newInvoice, line_items: items });
                    }} style={INPUT_STYLE} />
                    <input type="number" placeholder="Qty" min={1} value={li.quantity} onChange={e => {
                      const items = [...newInvoice.line_items];
                      items[idx] = { ...items[idx], quantity: parseInt(e.target.value) || 0 };
                      setNewInvoice({ ...newInvoice, line_items: items });
                    }} style={INPUT_STYLE} />
                    <input type="number" placeholder="Unit Price" min={0} value={li.unit_price || ""} onChange={e => {
                      const items = [...newInvoice.line_items];
                      items[idx] = { ...items[idx], unit_price: parseFloat(e.target.value) || 0 };
                      setNewInvoice({ ...newInvoice, line_items: items });
                    }} style={INPUT_STYLE} />
                    {newInvoice.line_items.length > 1 && (
                      <button onClick={() => {
                        const items = newInvoice.line_items.filter((_, i) => i !== idx);
                        setNewInvoice({ ...newInvoice, line_items: items });
                      }} style={{ color: "#DC2626", padding: "4px" }}><X size={16} /></button>
                    )}
                  </div>
                ))}
              </div>

              <div style={{ background: "#F8F9FA", borderRadius: 8, padding: 16, marginBottom: 20 }}>
                <div style={{ display: "flex", justifyContent: "space-between", fontSize: "0.85rem", marginBottom: 6 }}>
                  <span style={{ color: "#6B7280" }}>Subtotal</span>
                  <span style={{ color: "#1A1A2E" }}>{fmt(newTotals.subtotal)}</span>
                </div>
                <div style={{ display: "flex", justifyContent: "space-between", fontSize: "0.85rem", marginBottom: 6 }}>
                  <span style={{ color: "#6B7280" }}>VAT (7.5%)</span>
                  <span style={{ color: "#1A1A2E" }}>{fmt(newTotals.vat)}</span>
                </div>
                <div style={{ display: "flex", justifyContent: "space-between", fontSize: "1rem", fontWeight: 700, borderTop: "1px solid #E9ECEF", paddingTop: 8 }}>
                  <span style={{ color: "#1B4332" }}>Grand Total</span>
                  <span style={{ color: "#1B4332" }}>{fmt(newTotals.total)}</span>
                </div>
              </div>

              <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
                <button onClick={() => setShowCreateInvoice(false)} style={{ padding: "8px 16px", borderRadius: 6, background: "#F8F9FA", border: "1px solid #E9ECEF", fontSize: "0.85rem", color: "#1A1A2E" }}>Cancel</button>
                <button onClick={handleCreateInvoice} disabled={!newInvoice.customer_id || !newInvoice.due_date || newInvoice.line_items.some(li => !li.product_name || li.quantity <= 0)} style={{ padding: "8px 16px", borderRadius: 6, background: "#1B4332", color: "#FFFFFF", fontSize: "0.85rem", fontWeight: 500, opacity: !newInvoice.customer_id || !newInvoice.due_date ? 0.5 : 1 }}>Create Invoice</button>
              </div>
            </div>
          </div>
        )}

        {showRecordPayment && selectedInvoice && (
          <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }} onClick={() => setShowRecordPayment(false)}>
            <div style={{ background: "#FFFFFF", borderRadius: 12, maxWidth: 480, width: "90%", padding: 24 }} onClick={e => e.stopPropagation()}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
                <h2 style={{ fontSize: "1.25rem", fontFamily: "'DM Serif Display', serif", color: "#1A1A2E" }}>Record Payment</h2>
                <button onClick={() => setShowRecordPayment(false)} style={{ color: "#6B7280" }}><X size={20} /></button>
              </div>
              <p style={{ fontSize: "0.85rem", color: "#6B7280", marginBottom: 16 }}>
                Invoice: <strong style={{ color: "#1A1A2E" }}>{selectedInvoice.invoice_number}</strong> — Balance: <strong style={{ color: "#1B4332" }}>{fmt(selectedInvoice.total_amount)}</strong>
              </p>
              <div style={{ marginBottom: 12 }}>
                <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#6B7280", marginBottom: 4 }}>Amount</label>
                <input type="number" min={0} step="0.01" value={paymentAmount} onChange={e => setPaymentAmount(e.target.value)} placeholder={fmt(selectedInvoice.total_amount)} style={INPUT_STYLE} />
              </div>
              <div style={{ marginBottom: 12 }}>
                <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#6B7280", marginBottom: 4 }}>Payment Date</label>
                <input type="date" value={paymentDate} onChange={e => setPaymentDate(e.target.value)} style={INPUT_STYLE} />
              </div>
              <div style={{ marginBottom: 20 }}>
                <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#6B7280", marginBottom: 4 }}>Payment Method</label>
                <select value={paymentMethod} onChange={e => setPaymentMethod(e.target.value)} style={{ ...INPUT_STYLE, appearance: "auto" }}>
                  <option value="bank_transfer">Bank Transfer</option>
                  <option value="cash">Cash</option>
                  <option value="cheque">Cheque</option>
                  <option value="pos">POS</option>
                </select>
              </div>
              <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
                <button onClick={() => setShowRecordPayment(false)} style={{ padding: "8px 16px", borderRadius: 6, background: "#F8F9FA", border: "1px solid #E9ECEF", fontSize: "0.85rem", color: "#1A1A2E" }}>Cancel</button>
                <button onClick={handleRecordPayment} disabled={!paymentAmount} style={{ padding: "8px 16px", borderRadius: 6, background: "#16A34A", color: "#FFFFFF", fontSize: "0.85rem", fontWeight: 500, opacity: !paymentAmount ? 0.5 : 1 }}>Record Payment</button>
              </div>
            </div>
          </div>
        )}
      </div>
    </WorkspaceShell>
  );
}
