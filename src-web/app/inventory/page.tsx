"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { KPICard } from "@/components/ui/kpi-card";
import { getToken } from "@/lib/session";
import { apiGet } from "@/lib/api";
import { Package, Plus, ArrowDownRight, ArrowUpRight, AlertTriangle } from "lucide-react";

interface Product {
  id: string;
  code: string;
  name: string;
  category: string;
  unit: string;
  unit_price: number;
  stock_quantity: number;
  reorder_level: number;
  is_active: boolean;
}

interface StockMovement {
  id: string;
  product_name: string;
  movement_type: string;
  quantity: number;
  reference: string;
  date: string;
  notes: string;
}

export default function InventoryPage() {
  const token = getToken();
  const [products, setProducts] = useState<Product[]>([]);
  const [movements, setMovements] = useState<StockMovement[]>([]);
  const [tab, setTab] = useState<"products" | "movements">("products");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const [prodRes, movRes] = await Promise.all([
          apiGet("/inventory/products", token),
          apiGet("/inventory/movements", token),
        ]);
        if (prodRes.ok) setProducts((await prodRes.json()).data || []);
        if (movRes.ok) setMovements((await movRes.json()).data || []);
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  const totalValue = products.reduce((s, p) => s + p.stock_quantity * p.unit_price, 0);
  const lowStock = products.filter((p) => p.stock_quantity <= p.reorder_level).length;

  const productColumns: Column<Product>[] = [
    { key: "code", label: "Code", sortable: true, width: "100px" },
    { key: "name", label: "Product", sortable: true },
    { key: "category", label: "Category", width: "120px" },
    { key: "unit", label: "Unit", width: "60px" },
    { key: "unit_price", label: "Price", align: "right", width: "120px", render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}` },
    { key: "stock_quantity", label: "Qty", align: "right", width: "80px" },
    {
      key: "reorder_level",
      label: "Alert",
      width: "60px",
      render: (_: number, row: Product) =>
        row.stock_quantity <= row.reorder_level ? (
          <AlertTriangle size={14} style={{ color: "#DC3545" }} />
        ) : null,
    },
  ];

  const movementColumns: Column<StockMovement>[] = [
    { key: "date", label: "Date", sortable: true, width: "110px" },
    { key: "product_name", label: "Product" },
    {
      key: "movement_type",
      label: "Type",
      width: "90px",
      render: (v: string) => (
        <span style={{ display: "flex", alignItems: "center", gap: 4, fontSize: "0.8rem", color: v === "in" ? "#198754" : "#DC3545" }}>
          {v === "in" ? <ArrowDownRight size={12} /> : <ArrowUpRight size={12} />}
          {v === "in" ? "In" : "Out"}
        </span>
      ),
    },
    { key: "quantity", label: "Qty", align: "right", width: "80px" },
    { key: "reference", label: "Reference", width: "140px" },
    { key: "notes", label: "Notes" },
  ];

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <Package size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> Inventory
          </h1>
          <button className="btn btn-primary" style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <Plus size={16} /> New Product
          </button>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", gap: 16, marginBottom: 20 }}>
          <KPICard title="Total Value" value={totalValue} change={0} trend="up" color="primary" icon={<Package size={16} />} />
          <KPICard title="Low Stock Items" value={lowStock} change={0} trend={lowStock > 0 ? "down" : "up"} color="danger" icon={<AlertTriangle size={16} />} />
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: "2px solid #E9ECEF" }}>
          {(["products", "movements"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{ padding: "10px 16px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400, color: tab === t ? "#1B4332" : "#868E96", borderBottom: tab === t ? "2px solid #1B4332" : "2px solid transparent", marginBottom: -2 }}>
              {t.charAt(0).toUpperCase() + t.slice(1)}
            </button>
          ))}
        </div>

        <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
          {loading ? <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
            : tab === "products" ? <DataTable columns={productColumns} data={products} pageSize={15} emptyMessage="No products" />
            : <DataTable columns={movementColumns} data={movements} pageSize={15} emptyMessage="No movements" />}
        </div>
      </div>
    </WorkspaceShell>
  );
}
