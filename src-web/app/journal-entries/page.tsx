"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet, apiPost } from "@/lib/api";
import { FileText, Plus, CheckCircle, Clock, Edit3, X } from "lucide-react";

interface JournalEntry {
  id: string;
  reference: string;
  entry_date: string;
  narration: string;
  total_debit: number;
  total_credit: number;
  status: "draft" | "submitted" | "posted";
  created_by: string;
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

const TABS = ["draft", "submitted", "posted"] as const;
type TabType = (typeof TABS)[number];

const TAB_ICONS: Record<TabType, typeof Edit3> = {
  draft: Edit3,
  submitted: Clock,
  posted: CheckCircle,
};

export default function JournalEntriesPage() {
  const token = getToken();
  const [activeTab, setActiveTab] = useState<TabType>("draft");
  const [entries, setEntries] = useState<JournalEntry[]>([]);
  const [showCreate, setShowCreate] = useState(false);
  const [loading, setLoading] = useState(true);
  const [selectedEntry, setSelectedEntry] = useState<JournalEntry | null>(null);
  const [newEntry, setNewEntry] = useState({
    entry_date: new Date().toISOString().split("T")[0],
    narration: "",
    line_items: [
      { account_code: "", account_name: "", debit: 0, credit: 0, description: "" },
      { account_code: "", account_name: "", debit: 0, credit: 0, description: "" },
    ],
  });

  useEffect(() => {
    async function load() {
      try {
        const res = await apiGet(`/journals?status=${activeTab}`, token);
        if (res.ok) setEntries((await res.json()).data || []);
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [activeTab, token]);

  const totalDebit = newEntry.line_items.reduce((s, l) => s + (l.debit || 0), 0);
  const totalCredit = newEntry.line_items.reduce((s, l) => s + (l.credit || 0), 0);
  const isBalanced = totalDebit === totalCredit && totalDebit > 0;

  function addLineItem() {
    setNewEntry({
      ...newEntry,
      line_items: [
        ...newEntry.line_items,
        { account_code: "", account_name: "", debit: 0, credit: 0, description: "" },
      ],
    });
  }

  function removeLineItem(index: number) {
    setNewEntry({
      ...newEntry,
      line_items: newEntry.line_items.filter((_, i) => i !== index),
    });
  }

  function updateLineItem(index: number, field: string, value: string | number) {
    const items = [...newEntry.line_items];
    items[index] = { ...items[index], [field]: value };
    setNewEntry({ ...newEntry, line_items: items });
  }

  async function handleSubmit(status: "draft" | "submitted") {
    try {
      await apiPost("/journals", { ...newEntry, status }, token);
      setShowCreate(false);
      setNewEntry({
        entry_date: new Date().toISOString().split("T")[0],
        narration: "",
        line_items: [
          { account_code: "", account_name: "", debit: 0, credit: 0, description: "" },
          { account_code: "", account_name: "", debit: 0, credit: 0, description: "" },
        ],
      });
    } catch {
      // handle error
    }
  }

  const columns: Column<JournalEntry>[] = [
    { key: "reference", label: "Reference", sortable: true, width: "140px" },
    { key: "entry_date", label: "Date", sortable: true, width: "110px" },
    { key: "narration", label: "Narration" },
    {
      key: "total_debit",
      label: "Debit",
      align: "right",
      width: "130px",
      render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}`,
    },
    {
      key: "total_credit",
      label: "Credit",
      align: "right",
      width: "130px",
      render: (v: number) => `₦${(v || 0).toLocaleString("en-NG")}`,
    },
    {
      key: "status",
      label: "Status",
      width: "100px",
      render: (v: string) => {
        const colors: Record<string, { bg: string; color: string }> = {
          draft: { bg: "rgba(134,142,150,0.12)", color: "#868E96" },
          submitted: { bg: "rgba(255,193,7,0.12)", color: "#B8860B" },
          posted: { bg: "rgba(25,135,84,0.12)", color: "#198754" },
        };
        const c = colors[v] || colors.draft;
        return (
          <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: c.bg, color: c.color }}>
            {v.charAt(0).toUpperCase() + v.slice(1)}
          </span>
        );
      },
    },
  ];

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <FileText size={20} style={{ verticalAlign: "middle", marginRight: 8 }} />
            Journal Entries
          </h1>
          <button className="btn btn-primary" onClick={() => setShowCreate(true)} style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <Plus size={16} />
            New Entry
          </button>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: "2px solid #E9ECEF" }}>
          {TABS.map((tab) => {
            const Icon = TAB_ICONS[tab];
            const count = entries.filter((e) => e.status === tab).length;
            const isActive = activeTab === tab;
            return (
              <button
                key={tab}
                onClick={() => setActiveTab(tab)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 6,
                  padding: "10px 16px",
                  fontSize: "0.85rem",
                  fontWeight: isActive ? 600 : 400,
                  color: isActive ? "#1B4332" : "#868E96",
                  borderBottom: isActive ? "2px solid #1B4332" : "2px solid transparent",
                  marginBottom: -2,
                  transition: "all 150ms ease",
                }}
              >
                <Icon size={14} />
                {tab.charAt(0).toUpperCase() + tab.slice(1)}
                {count > 0 && (
                  <span
                    style={{
                      fontSize: "0.7rem",
                      padding: "1px 6px",
                      borderRadius: 10,
                      background: "rgba(27,67,50,0.12)",
                      color: "#1B4332",
                    }}
                  >
                    {count}
                  </span>
                )}
              </button>
            );
          })}
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
          <DataTable
            columns={columns}
            data={entries}
            pageSize={15}
            emptyMessage={`No ${activeTab} entries`}
            onRowClick={(row) => setSelectedEntry(row)}
          />
        </div>

        {showCreate && (
          <div
            style={{
              position: "fixed",
              inset: 0,
              background: "rgba(26,26,46,0.4)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              zIndex: 1000,
            }}
            onClick={(e) => {
              if (e.target === e.currentTarget) setShowCreate(false);
            }}
          >
            <div
              style={{
                background: "#FFFFFF",
                borderRadius: 16,
                boxShadow: "0 12px 40px rgba(0,0,0,0.16)",
                width: "90%",
                maxWidth: 960,
                maxHeight: "85vh",
                overflow: "auto",
              }}
              className="scroll-y"
            >
              <div style={{ padding: "20px 24px", borderBottom: "1px solid #E9ECEF", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <h2 style={{ fontSize: "1.1rem", fontWeight: 600, color: "#1A1A2E" }}>New Journal Entry</h2>
                <button className="btn btn-ghost btn-sm" onClick={() => setShowCreate(false)}>
                  <X size={16} />
                </button>
              </div>

              <div style={{ padding: 24 }}>
                <div style={{ display: "flex", gap: 16, marginBottom: 20 }}>
                  <div style={{ flex: 1 }}>
                    <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#495057", marginBottom: 4 }}>Date</label>
                    <input
                      type="date"
                      value={newEntry.entry_date}
                      onChange={(e) => setNewEntry({ ...newEntry, entry_date: e.target.value })}
                      style={{ width: "100%" }}
                    />
                  </div>
                  <div style={{ flex: 2 }}>
                    <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#495057", marginBottom: 4 }}>Narration</label>
                    <input
                      type="text"
                      value={newEntry.narration}
                      onChange={(e) => setNewEntry({ ...newEntry, narration: e.target.value })}
                      placeholder="Enter narration for this journal entry"
                      style={{ width: "100%" }}
                    />
                  </div>
                </div>

                <h3 style={{ fontSize: "0.9rem", fontWeight: 600, marginBottom: 12, color: "#1A1A2E" }}>Line Items</h3>
                <table style={{ width: "100%", borderCollapse: "collapse", marginBottom: 12 }}>
                  <thead>
                    <tr>
                      <th style={{ padding: "8px 12px", textAlign: "left", fontSize: "0.8rem", fontWeight: 600, color: "#868E96", borderBottom: "1px solid #E9ECEF", textTransform: "uppercase" }}>Account</th>
                      <th style={{ padding: "8px 12px", textAlign: "right", fontSize: "0.8rem", fontWeight: 600, color: "#868E96", borderBottom: "1px solid #E9ECEF", textTransform: "uppercase", width: 140 }}>Debit</th>
                      <th style={{ padding: "8px 12px", textAlign: "right", fontSize: "0.8rem", fontWeight: 600, color: "#868E96", borderBottom: "1px solid #E9ECEF", textTransform: "uppercase", width: 140 }}>Credit</th>
                      <th style={{ padding: "8px 12px", textAlign: "left", fontSize: "0.8rem", fontWeight: 600, color: "#868E96", borderBottom: "1px solid #E9ECEF", textTransform: "uppercase" }}>Description</th>
                      <th style={{ width: 40 }} />
                    </tr>
                  </thead>
                  <tbody>
                    {newEntry.line_items.map((item, i) => (
                      <tr key={i}>
                        <td style={{ padding: "6px 12px" }}>
                          <input
                            type="text"
                            value={item.account_code}
                            onChange={(e) => updateLineItem(i, "account_code", e.target.value)}
                            placeholder="Account code"
                            style={{ width: "100%" }}
                          />
                        </td>
                        <td style={{ padding: "6px 12px" }}>
                          <input
                            type="number"
                            value={item.debit || ""}
                            onChange={(e) => updateLineItem(i, "debit", parseFloat(e.target.value) || 0)}
                            placeholder="0"
                            style={{ width: "100%", textAlign: "right" }}
                          />
                        </td>
                        <td style={{ padding: "6px 12px" }}>
                          <input
                            type="number"
                            value={item.credit || ""}
                            onChange={(e) => updateLineItem(i, "credit", parseFloat(e.target.value) || 0)}
                            placeholder="0"
                            style={{ width: "100%", textAlign: "right" }}
                          />
                        </td>
                        <td style={{ padding: "6px 12px" }}>
                          <input
                            type="text"
                            value={item.description}
                            onChange={(e) => updateLineItem(i, "description", e.target.value)}
                            placeholder="Line description"
                            style={{ width: "100%" }}
                          />
                        </td>
                        <td style={{ padding: "6px 4px" }}>
                          {newEntry.line_items.length > 2 && (
                            <button className="btn btn-ghost btn-sm" onClick={() => removeLineItem(i)} style={{ color: "#DC3545" }}>
                              <X size={14} />
                            </button>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                  <tfoot>
                    <tr style={{ fontWeight: 600 }}>
                      <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: "#1A1A2E" }}>Totals</td>
                      <td style={{ padding: "10px 12px", textAlign: "right", fontSize: "0.85rem", color: "#1A1A2E", fontFamily: '"JetBrains Mono", monospace' }}>₦{totalDebit.toLocaleString("en-NG")}</td>
                      <td style={{ padding: "10px 12px", textAlign: "right", fontSize: "0.85rem", color: "#1A1A2E", fontFamily: '"JetBrains Mono", monospace' }}>₦{totalCredit.toLocaleString("en-NG")}</td>
                      <td colSpan={2} />
                    </tr>
                  </tfoot>
                </table>

                <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 20 }}>
                  <button className="btn btn-secondary btn-sm" onClick={addLineItem} style={{ display: "flex", alignItems: "center", gap: 4 }}>
                    <Plus size={14} />
                    Add Line
                  </button>
                  <span
                    style={{
                      fontSize: "0.8rem",
                      padding: "2px 10px",
                      borderRadius: 4,
                      background: isBalanced ? "rgba(25,135,84,0.12)" : totalDebit || totalCredit ? "rgba(220,53,69,0.12)" : "rgba(134,142,150,0.08)",
                      color: isBalanced ? "#198754" : totalDebit || totalCredit ? "#DC3545" : "#868E96",
                    }}
                  >
                    {isBalanced ? "Entry Balanced" : totalDebit || totalCredit ? `Difference: ₦${Math.abs(totalDebit - totalCredit).toLocaleString("en-NG")}` : "Enter amounts to check balance"}
                  </span>
                </div>
              </div>

              <div style={{ padding: "16px 24px", borderTop: "1px solid #E9ECEF", display: "flex", justifyContent: "flex-end", gap: 8 }}>
                <button className="btn btn-secondary" onClick={() => setShowCreate(false)}>Cancel</button>
                <button className="btn btn-secondary" onClick={() => handleSubmit("draft")}>Save Draft</button>
                <button className="btn btn-primary" disabled={!isBalanced} onClick={() => handleSubmit("submitted")}>Submit for Approval</button>
              </div>
            </div>
          </div>
        )}
      </div>
    </WorkspaceShell>
  );
}
