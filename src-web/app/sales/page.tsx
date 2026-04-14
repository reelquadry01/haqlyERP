"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet, apiPost } from "@/lib/api";
import { ShoppingCart, Plus, X, FileText } from "lucide-react";

interface Customer {
  id: string;
  name: string;
  code: string;
  email: string;
  phone: string;
  tin: string;
  outstanding_balance: number;
}

interface Invoice {
  id: string;
  invoice_number: string;
  customer_name: string;
  invoice_date: string;
  due_date: string;
  total_amount: number;
  status: string;
}

export default function SalesPage() {
  const token = getToken();
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [tab, setTab] = useState<"customers" | "invoices">("invoices");
  const [showCreate, setShowCreate] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const [custRes, invRes] = await Promise.all([
          apiGet("/sales/customers", token),
          apiGet("/sales/invoices", token),
        ]);
        if (custRes.ok) setCustomers((await custRes.json()).data || []);
        if (invRes.ok) setInvoices((await invRes.json()).data || []);
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  const customerColumns: Column<Customer>[] = [
    { key: "code", label: "Code", sortable: true, width: "100px" },
    { key: "name", label: "Customer Name", sortable: true },
    { key: "email", label: "Email", width: "180px" },
    { key: "phone", label: "Phone", width: "120px" },
    { key: "tin", label: "TIN", width: "120px" },
    {
      key: "outstanding_balance",
      label: "Balance",
      align: "right",
      width: "130px",
      render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}`,
    },
  ];

  const invoiceColumns: Column<Invoice>[] = [
    { key: "invoice_number", label: "Invoice #", sortable: true, width: "140px" },
    { key: "customer_name", label: "Customer", sortable: true },
    { key: "invoice_date", label: "Date", sortable: true, width: "110px" },
    { key: "due_date", label: "Due", width: "110px" },
    {
      key: "total_amount",
      label: "Amount",
      align: "right",
      sortable: true,
      width: "130px",
      render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}`,
    },
    {
      key: "status",
      label: "Status",
      width: "100px",
      render: (v: string) => {
        const map: Record<string, { bg: string; color: string }> = {
          draft: { bg: "rgba(134,142,150,0.12)", color: "#868E96" },
          sent: { bg: "rgba(13,202,240,0.12)", color: "#0DCAF0" },
          paid: { bg: "rgba(25,135,84,0.12)", color: "#198754" },
          overdue: { bg: "rgba(220,53,69,0.12)", color: "#DC3545" },
          partial: { bg: "rgba(255,193,7,0.12)", color: "#B8860B" },
        };
        const c = map[v] || map.draft;
        return <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: c.bg, color: c.color }}>{v}</span>;
      },
    },
  ];

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <ShoppingCart size={20} style={{ verticalAlign: "middle", marginRight: 8 }} />
            Sales
          </h1>
          <div style={{ display: "flex", gap: 8 }}>
            <button className="btn btn-primary" onClick={() => setShowCreate(true)} style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <Plus size={16} /> New Invoice
            </button>
          </div>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: "2px solid #E9ECEF" }}>
          {(["invoices", "customers"] as const).map((t) => (
            <button
              key={t}
              onClick={() => setTab(t)}
              style={{
                padding: "10px 16px",
                fontSize: "0.85rem",
                fontWeight: tab === t ? 600 : 400,
                color: tab === t ? "#1B4332" : "#868E96",
                borderBottom: tab === t ? "2px solid #1B4332" : "2px solid transparent",
                marginBottom: -2,
              }}
            >
              {t.charAt(0).toUpperCase() + t.slice(1)}
            </button>
          ))}
        </div>

        <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
          {loading ? (
            <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
          ) : tab === "invoices" ? (
            <DataTable columns={invoiceColumns} data={invoices} pageSize={15} emptyMessage="No invoices" />
          ) : (
            <DataTable columns={customerColumns} data={customers} pageSize={15} emptyMessage="No customers" />
          )}
        </div>
      </div>
    </WorkspaceShell>
  );
}
