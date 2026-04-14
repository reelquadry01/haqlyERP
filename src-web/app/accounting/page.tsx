"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet, apiPost, apiDelete } from "@/lib/api";
import { BookOpen, Plus, ChevronRight, ChevronDown, FolderOpen } from "lucide-react";

interface Account {
  id: string;
  code: string;
  name: string;
  account_type: string;
  parent_id: string | null;
  is_active: boolean;
  balance: number;
  children?: Account[];
}

interface FiscalPeriod {
  id: string;
  name: string;
  start_date: string;
  end_date: string;
  status: string;
}

export default function AccountingPage() {
  const token = getToken();
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [periods, setPeriods] = useState<FiscalPeriod[]>([]);
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const [accRes, perRes] = await Promise.all([
          apiGet("/accounting/accounts", token),
          apiGet("/accounting/periods", token),
        ]);
        if (accRes.ok) setAccounts((await accRes.json()).data || []);
        if (perRes.ok) setPeriods((await perRes.json()).data || []);
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  function toggleExpand(id: string) {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function flattenAccounts(accounts: Account[], level = 0): (Account & { level: number })[] {
    const result: (Account & { level: number })[] = [];
    for (const acc of accounts) {
      result.push({ ...acc, level });
      if (expandedIds.has(acc.id) && acc.children?.length) {
        result.push(...flattenAccounts(acc.children, level + 1));
      }
    }
    return result;
  }

  const flatAccounts = flattenAccounts(accounts);

  const periodColumns: Column<FiscalPeriod>[] = [
    { key: "name", label: "Period", sortable: true },
    { key: "start_date", label: "Start", sortable: true, width: "120px" },
    { key: "end_date", label: "End", sortable: true, width: "120px" },
    {
      key: "status",
      label: "Status",
      width: "100px",
      render: (v: string) => (
        <span
          style={{
            fontSize: "0.75rem",
            padding: "2px 8px",
            borderRadius: 4,
            background: v === "Open" ? "rgba(25,135,84,0.12)" : "rgba(134,142,150,0.12)",
            color: v === "Open" ? "#198754" : "#868E96",
          }}
        >
          {v}
        </span>
      ),
    },
  ];

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <BookOpen size={20} style={{ verticalAlign: "middle", marginRight: 8 }} />
            Chart of Accounts
          </h1>
          <button
            className="btn btn-primary"
            onClick={() => setShowCreateForm(!showCreateForm)}
            style={{ display: "flex", alignItems: "center", gap: 6 }}
          >
            <Plus size={16} />
            New Account
          </button>
        </div>

        {loading ? (
          <div style={{ display: "flex", justifyContent: "center", padding: 40 }}>
            <div className="splash-loader" />
          </div>
        ) : (
          <div style={{ display: "grid", gridTemplateColumns: "2fr 1fr", gap: 20 }}>
            <div
              style={{
                background: "#FFFFFF",
                border: "1px solid #E9ECEF",
                borderRadius: 12,
                padding: 20,
                boxShadow: "0 2px 8px rgba(0,0,0,0.08)",
              }}
            >
              <h3 style={{ fontSize: "0.95rem", fontWeight: 600, marginBottom: 16, color: "#1A1A2E" }}>
                Account Tree
              </h3>
              <div style={{ maxHeight: 600, overflowY: "auto" }} className="scroll-y">
                {flatAccounts.length === 0 ? (
                  <div style={{ textAlign: "center", padding: 32, color: "#868E96" }}>
                    <FolderOpen size={32} style={{ marginBottom: 8 }} />
                    <p>No accounts created yet</p>
                  </div>
                ) : (
                  flatAccounts.map((acc) => (
                    <div
                      key={acc.id}
                      style={{
                        display: "flex",
                        alignItems: "center",
                        padding: "8px 12px",
                        paddingLeft: 12 + acc.level * 24,
                        borderBottom: "1px solid #E9ECEF",
                        fontSize: "0.85rem",
                        cursor: acc.children?.length ? "pointer" : "default",
                      }}
                      onClick={() => acc.children?.length && toggleExpand(acc.id)}
                    >
                      {acc.children?.length ? (
                        expandedIds.has(acc.id) ? (
                          <ChevronDown size={14} style={{ marginRight: 4, color: "#868E96" }} />
                        ) : (
                          <ChevronRight size={14} style={{ marginRight: 4, color: "#868E96" }} />
                        )
                      ) : (
                        <span style={{ width: 18, marginRight: 4 }} />
                      )}
                      <span style={{ fontFamily: '"JetBrains Mono", monospace', color: "#1B4332", marginRight: 12, fontWeight: 500 }}>
                        {acc.code}
                      </span>
                      <span style={{ flex: 1, color: "#1A1A2E" }}>{acc.name}</span>
                      <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: "rgba(27,67,50,0.08)", color: "#1B4332", marginRight: 8 }}>
                        {acc.account_type}
                      </span>
                    </div>
                  ))
                )}
              </div>
            </div>

            <div
              style={{
                background: "#FFFFFF",
                border: "1px solid #E9ECEF",
                borderRadius: 12,
                padding: 20,
                boxShadow: "0 2px 8px rgba(0,0,0,0.08)",
              }}
            >
              <h3 style={{ fontSize: "0.95rem", fontWeight: 600, marginBottom: 16, color: "#1A1A2E" }}>
                Fiscal Periods
              </h3>
              <DataTable columns={periodColumns} data={periods} pageSize={10} emptyMessage="No periods defined" />
            </div>
          </div>
        )}
      </div>
    </WorkspaceShell>
  );
}
