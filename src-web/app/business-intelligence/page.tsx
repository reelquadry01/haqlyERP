"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { getToken } from "@/lib/session";
import { apiGet, apiPost } from "@/lib/api";
import { BarChart3, Plus, Settings, Grid3X3 } from "lucide-react";

interface Dashboard {
  id: string;
  name: string;
  description: string;
  is_default: boolean;
  is_shared: boolean;
}

interface Widget {
  id: string;
  widget_type: string;
  title: string;
  dataset_id: string;
  config: Record<string, unknown>;
  position_x: number;
  position_y: number;
  width: number;
  height: number;
}

interface Dataset {
  id: string;
  name: string;
  source_type: string;
  row_count: number;
  is_active: boolean;
}

interface Query {
  id: string;
  name: string;
  query_type: string;
  query_text: string;
  is_active: boolean;
}

export default function BusinessIntelligencePage() {
  const token = getToken();
  const [dashboards, setDashboards] = useState<Dashboard[]>([]);
  const [datasets, setDatasets] = useState<Dataset[]>([]);
  const [queries, setQueries] = useState<Query[]>([]);
  const [tab, setTab] = useState<"dashboards" | "datasets" | "queries">("dashboards");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const [dashRes, dsRes, qRes] = await Promise.all([
          apiGet("/bi/dashboards", token),
          apiGet("/bi/datasets", token),
          apiGet("/bi/queries", token),
        ]);
        if (dashRes.ok) setDashboards((await dashRes.json()).data || []);
        if (dsRes.ok) setDatasets((await dsRes.json()).data || []);
        if (qRes.ok) setQueries((await qRes.json()).data || []);
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <BarChart3 size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> Business Intelligence
          </h1>
          <div style={{ display: "flex", gap: 8 }}>
            <button className="btn btn-primary" style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <Plus size={16} /> New Dashboard
            </button>
          </div>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: "2px solid #E9ECEF" }}>
          {(["dashboards", "datasets", "queries"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{ padding: "10px 16px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400, color: tab === t ? "#1B4332" : "#868E96", borderBottom: tab === t ? "2px solid #1B4332" : "2px solid transparent", marginBottom: -2, textTransform: "capitalize" }}>
              {t}
            </button>
          ))}
        </div>

        {loading ? (
          <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
        ) : tab === "dashboards" ? (
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))", gap: 16 }}>
            {dashboards.length === 0 ? (
              <div style={{ gridColumn: "1 / -1", textAlign: "center", padding: 40, color: "#868E96" }}>No dashboards. Create one to get started.</div>
            ) : (
              dashboards.map((d) => (
                <div
                  key={d.id}
                  style={{
                    background: "#FFFFFF",
                    border: "1px solid #E9ECEF",
                    borderRadius: 12,
                    padding: 20,
                    boxShadow: "0 2px 8px rgba(0,0,0,0.08)",
                    cursor: "pointer",
                    transition: "all 150ms ease",
                  }}
                  onMouseEnter={(e) => { e.currentTarget.style.boxShadow = "0 4px 16px rgba(0,0,0,0.10)"; }}
                  onMouseLeave={(e) => { e.currentTarget.style.boxShadow = "0 2px 8px rgba(0,0,0,0.08)"; }}
                >
                  <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
                    <Grid3X3 size={16} style={{ color: "#1B4332" }} />
                    <span style={{ fontWeight: 600, fontSize: "0.95rem", color: "#1A1A2E" }}>{d.name}</span>
                    {d.is_default && <span style={{ fontSize: "0.7rem", padding: "1px 6px", borderRadius: 4, background: "rgba(212,175,55,0.12)", color: "#D4AF37" }}>Default</span>}
                  </div>
                  <p style={{ fontSize: "0.8rem", color: "#868E96" }}>{d.description || "No description"}</p>
                </div>
              ))
            )}
          </div>
        ) : tab === "datasets" ? (
          <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
            <table style={{ width: "100%", borderCollapse: "collapse" }}>
              <thead>
                <tr>
                  <th style={{ padding: "10px 12px", textAlign: "left", fontWeight: 600, fontSize: "0.8rem", color: "#868E96", textTransform: "uppercase", borderBottom: "1px solid #E9ECEF" }}>Name</th>
                  <th style={{ padding: "10px 12px", textAlign: "left", fontWeight: 600, fontSize: "0.8rem", color: "#868E96", textTransform: "uppercase", borderBottom: "1px solid #E9ECEF" }}>Source</th>
                  <th style={{ padding: "10px 12px", textAlign: "right", fontWeight: 600, fontSize: "0.8rem", color: "#868E96", textTransform: "uppercase", borderBottom: "1px solid #E9ECEF" }}>Rows</th>
                  <th style={{ padding: "10px 12px", textAlign: "center", fontWeight: 600, fontSize: "0.8rem", color: "#868E96", textTransform: "uppercase", borderBottom: "1px solid #E9ECEF" }}>Active</th>
                </tr>
              </thead>
              <tbody>
                {datasets.length === 0 ? (
                  <tr><td colSpan={4} style={{ textAlign: "center", padding: 32, color: "#868E96" }}>No datasets</td></tr>
                ) : datasets.map((ds) => (
                  <tr key={ds.id} style={{ borderBottom: "1px solid #E9ECEF" }}>
                    <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: "#1A1A2E", fontWeight: 500 }}>{ds.name}</td>
                    <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: "#495057" }}>{ds.source_type}</td>
                    <td style={{ padding: "10px 12px", fontSize: "0.85rem", textAlign: "right", color: "#1A1A2E" }}>{ds.row_count?.toLocaleString() || 0}</td>
                    <td style={{ padding: "10px 12px", textAlign: "center" }}>
                      <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: ds.is_active ? "rgba(25,135,84,0.12)" : "rgba(220,53,69,0.12)", color: ds.is_active ? "#198754" : "#DC3545" }}>
                        {ds.is_active ? "Active" : "Inactive"}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
            <table style={{ width: "100%", borderCollapse: "collapse" }}>
              <thead>
                <tr>
                  <th style={{ padding: "10px 12px", textAlign: "left", fontWeight: 600, fontSize: "0.8rem", color: "#868E96", textTransform: "uppercase", borderBottom: "1px solid #E9ECEF" }}>Name</th>
                  <th style={{ padding: "10px 12px", textAlign: "left", fontWeight: 600, fontSize: "0.8rem", color: "#868E96", textTransform: "uppercase", borderBottom: "1px solid #E9ECEF" }}>Type</th>
                  <th style={{ padding: "10px 12px", textAlign: "left", fontWeight: 600, fontSize: "0.8rem", color: "#868E96", textTransform: "uppercase", borderBottom: "1px solid #E9ECEF" }}>Query</th>
                  <th style={{ padding: "10px 12px", textAlign: "center", fontWeight: 600, fontSize: "0.8rem", color: "#868E96", textTransform: "uppercase", borderBottom: "1px solid #E9ECEF" }}>Active</th>
                </tr>
              </thead>
              <tbody>
                {queries.length === 0 ? (
                  <tr><td colSpan={4} style={{ textAlign: "center", padding: 32, color: "#868E96" }}>No queries</td></tr>
                ) : queries.map((q) => (
                  <tr key={q.id} style={{ borderBottom: "1px solid #E9ECEF" }}>
                    <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: "#1A1A2E", fontWeight: 500 }}>{q.name}</td>
                    <td style={{ padding: "10px 12px", fontSize: "0.85rem", color: "#495057" }}>{q.query_type}</td>
                    <td style={{ padding: "10px 12px", fontSize: "0.8rem", color: "#868E96", maxWidth: 300, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{q.query_text}</td>
                    <td style={{ padding: "10px 12px", textAlign: "center" }}>
                      <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: q.is_active ? "rgba(25,135,84,0.12)" : "rgba(220,53,69,0.12)", color: q.is_active ? "#198754" : "#DC3545" }}>
                        {q.is_active ? "Active" : "Inactive"}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </WorkspaceShell>
  );
}
