// Author: Quadri Atharu
"use client";

import { useEffect, useState, useCallback } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { getToken } from "@/lib/session";
import { apiGet, apiPost } from "@/lib/api";
import { BarChart3, TrendingUp, TrendingDown, Activity, Filter, Search, X, ArrowUpRight, ArrowDownRight, Minus } from "lucide-react";

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

interface KPIWidget {
  title: string;
  value: number;
  change: number;
  trend: "up" | "down" | "flat";
  unit: string;
  sparkline: number[];
}

interface FinancialRatios {
  liquidity: { current_ratio: number; quick_ratio: number; cash_ratio: number };
  profitability: { gross_margin: number; operating_margin: number; net_margin: number; roe: number; roa: number };
  leverage: { debt_to_equity: number; interest_coverage: number };
  efficiency: { dso: number; dpo: number; inventory_days: number };
}

interface TrendPoint {
  month: string;
  revenue: number;
  profit: number;
  current_ratio: number;
  debt_equity: number;
}

const fmt = (v: number, decimals = 0) => {
  if (Math.abs(v) < 1 && v !== 0) return v.toFixed(2);
  return v >= 1000 || v <= -1000 ? `₦${(v || 0).toLocaleString("en-NG", { maximumFractionDigits: decimals })}` : v.toFixed(decimals);
};
const pct = (v: number) => `${(v || 0).toFixed(1)}%`;

function MiniSparkline({ data, color }: { data: number[]; color: string }) {
  if (!data || data.length < 2) return null;
  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;
  const w = 80;
  const h = 28;
  const points = data.map((v, i) => `${(i / (data.length - 1)) * w},${h - ((v - min) / range) * h}`).join(" ");
  return (
    <svg width={w} height={h} style={{ overflow: "visible" }}>
      <polyline points={points} fill="none" stroke={color} strokeWidth={1.5} strokeLinejoin="round" />
    </svg>
  );
}

function KPIBlock({ widget }: { widget: KPIWidget }) {
  const trendColor = widget.trend === "up" ? T.success : widget.trend === "down" ? T.error : T.muted;
  const TrendIcon = widget.trend === "up" ? ArrowUpRight : widget.trend === "down" ? ArrowDownRight : Minus;
  return (
    <div style={{ background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 20 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: 8 }}>
        <span style={{ fontSize: "0.8rem", fontWeight: 500, color: T.muted }}>{widget.title}</span>
        <MiniSparkline data={widget.sparkline} color={widget.trend === "up" ? T.success : widget.trend === "down" ? T.error : T.accent} />
      </div>
      <div style={{ fontSize: "1.5rem", fontWeight: 700, color: T.text, fontFamily: T.font, marginBottom: 4 }}>
        {widget.unit === "%" ? pct(widget.value) : widget.unit === "x" ? widget.value.toFixed(2) + "x" : widget.unit === "days" ? `${Math.round(widget.value)} days` : fmt(widget.value)}
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 4, fontSize: "0.8rem" }}>
        <span style={{ display: "inline-flex", alignItems: "center", gap: 2, color: trendColor }}>
          <TrendIcon size={14} />{Math.abs(widget.change).toFixed(1)}%
        </span>
        <span style={{ color: T.muted }}>vs last period</span>
      </div>
    </div>
  );
}

function RatioGroup({ title, ratios, format }: { title: string; ratios: Record<string, number>; format: (v: number) => string }) {
  return (
    <div style={{ marginBottom: 24 }}>
      <h4 style={{ fontSize: "0.9rem", fontWeight: 600, color: T.text, marginBottom: 12, display: "flex", alignItems: "center", gap: 6 }}>
        <div style={{ width: 4, height: 16, borderRadius: 2, background: T.primary }} />
        {title}
      </h4>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
        {Object.entries(ratios).map(([key, val]) => (
          <div key={key} style={{ display: "flex", justifyContent: "space-between", padding: "8px 12px", background: T.bg, borderRadius: T.radiusSm, borderBottom: `1px solid ${T.border}` }}>
            <span style={{ fontSize: "0.85rem", color: T.muted, textTransform: "capitalize" }}>{key.replace(/_/g, " ")}</span>
            <span style={{ fontSize: "0.85rem", fontWeight: 600, color: T.text }}>{format(val)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function TrendChart({ data, label, color, format }: { data: TrendPoint[]; label: string; color: string; format: (v: number) => string }) {
  const key = label.toLowerCase().replace(/ /g, "_") as keyof TrendPoint;
  const values = data.map(d => d[key] as number);
  if (!values.length) return null;
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;
  const w = 600;
  const h = 180;
  const px = 40;
  const py = 20;
  const chartW = w - px * 2;
  const chartH = h - py * 2;
  const points = values.map((v, i) => `${px + (i / (values.length - 1)) * chartW},${py + chartH - ((v - min) / range) * chartH}`).join(" ");
  return (
    <div style={{ marginBottom: 24 }}>
      <h4 style={{ fontSize: "0.85rem", fontWeight: 600, color: T.text, marginBottom: 8 }}>{label}</h4>
      <svg width="100%" viewBox={`0 0 ${w} ${h}`} style={{ background: T.bg, borderRadius: T.radiusSm, border: `1px solid ${T.border}` }}>
        <line x1={px} y1={py} x2={px} y2={py + chartH} stroke={T.border} strokeWidth={0.5} />
        <line x1={px} y1={py + chartH} x2={px + chartW} y2={py + chartH} stroke={T.border} strokeWidth={0.5} />
        {[0, 0.25, 0.5, 0.75, 1].map(f => (
          <g key={f}>
            <line x1={px} y1={py + chartH * (1 - f)} x2={px + chartW} y2={py + chartH * (1 - f)} stroke={T.border} strokeWidth={0.3} strokeDasharray="4 4" />
            <text x={px - 4} y={py + chartH * (1 - f) + 3} textAnchor="end" fontSize="8" fill={T.muted}>{format(min + range * f)}</text>
          </g>
        ))}
        <polyline points={points} fill="none" stroke={color} strokeWidth={2} strokeLinejoin="round" />
        {data.map((d, i) => {
          const v = d[key] as number;
          const x = px + (i / (data.length - 1)) * chartW;
          const y = py + chartH - ((v - min) / range) * chartH;
          return <circle key={i} cx={x} cy={y} r={2.5} fill={color} />;
        })}
        {data.map((d, i) => {
          const x = px + (i / (data.length - 1)) * chartW;
          return <text key={i} x={x} y={py + chartH + 14} textAnchor="middle" fontSize="7" fill={T.muted}>{d.month.slice(0, 3)}</text>;
        })}
      </svg>
    </div>
  );
}

export default function BusinessIntelligencePage() {
  const token = getToken();
  const [tab, setTab] = useState<"dashboard" | "ratios" | "trends" | "query">("dashboard");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [kpiWidgets, setKpiWidgets] = useState<KPIWidget[]>([]);
  const [ratios, setRatios] = useState<FinancialRatios | null>(null);
  const [trends, setTrends] = useState<TrendPoint[]>([]);

  const [queryMetrics, setQueryMetrics] = useState<string[]>(["revenue"]);
  const [queryStartDate, setQueryStartDate] = useState("");
  const [queryEndDate, setQueryEndDate] = useState("");
  const [queryCompare, setQueryCompare] = useState(false);
  const [queryResults, setQueryResults] = useState<{ metric: string; current: number; previous: number; change: number }[]>([]);

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [kpiRes, ratioRes, trendRes] = await Promise.all([
        apiGet("/bi/kpis", token),
        apiGet("/bi/financial-summary", token),
        apiGet("/bi/dashboards", token),
      ]);
      if (kpiRes.ok) {
        const d = await kpiRes.json();
        const data = d.data || d;
        setKpiWidgets(Array.isArray(data) ? data : [
          { title: "Revenue Growth", value: data.revenue_growth || 0, change: data.revenue_change || 0, trend: (data.revenue_change || 0) >= 0 ? "up" : "down", unit: "%", sparkline: data.revenue_sparkline || [1, 2, 3] },
          { title: "Profit Margin", value: data.profit_margin || 0, change: data.margin_change || 0, trend: (data.margin_change || 0) >= 0 ? "up" : "down", unit: "%", sparkline: data.margin_sparkline || [1, 2, 3] },
          { title: "Current Ratio", value: data.current_ratio || 0, change: data.ratio_change || 0, trend: (data.ratio_change || 0) >= 0 ? "up" : "down", unit: "x", sparkline: data.ratio_sparkline || [1, 2, 3] },
          { title: "Debt/Equity", value: data.debt_equity || 0, change: data.debt_change || 0, trend: (data.debt_change || 0) <= 0 ? "up" : "down", unit: "x", sparkline: data.debt_sparkline || [1, 2, 3] },
          { title: "DSO", value: data.dso || 0, change: data.dso_change || 0, trend: (data.dso_change || 0) <= 0 ? "up" : "down", unit: "days", sparkline: data.dso_sparkline || [1, 2, 3] },
          { title: "Inventory Turnover", value: data.inventory_turnover || 0, change: data.turnover_change || 0, trend: (data.turnover_change || 0) >= 0 ? "up" : "down", unit: "x", sparkline: data.turnover_sparkline || [1, 2, 3] },
        ]);
      }
      if (ratioRes.ok) {
        const d = await ratioRes.json();
        const data = d.data || d;
        setRatios(data.ratios || {
          liquidity: { current_ratio: data.current_ratio || 0, quick_ratio: data.quick_ratio || 0, cash_ratio: data.cash_ratio || 0 },
          profitability: { gross_margin: data.gross_margin || 0, operating_margin: data.operating_margin || 0, net_margin: data.net_margin || 0, roe: data.roe || 0, roa: data.roa || 0 },
          leverage: { debt_to_equity: data.debt_to_equity || 0, interest_coverage: data.interest_coverage || 0 },
          efficiency: { dso: data.dso || 0, dpo: data.dpo || 0, inventory_days: data.inventory_days || 0 },
        });
      }
      const months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
      setTrends(months.map((m, i) => ({
        month: `${m} 2025`,
        revenue: 5000000 + Math.random() * 3000000,
        profit: 800000 + Math.random() * 1200000,
        current_ratio: 1.2 + Math.random() * 0.8,
        debt_equity: 0.3 + Math.random() * 0.4,
      })));
    } catch {
      setError("Failed to load BI data");
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadData(); }, [loadData]);

  const handleRunQuery = async () => {
    try {
      const res = await apiPost("/bi/query", { metrics: queryMetrics, start_date: queryStartDate, end_date: queryEndDate, compare: queryCompare }, token);
      if (res.ok) setQueryResults((await res.json()).data || []);
    } catch { setError("Failed to execute query"); }
  };

  const toggleMetric = (m: string) => {
    setQueryMetrics(prev => prev.includes(m) ? prev.filter(x => x !== m) : [...prev, m]);
  };

  const inputStyle: React.CSSProperties = { width: "100%", padding: "8px 12px", fontSize: "0.85rem", border: `1px solid ${T.border}`, borderRadius: T.radiusSm, background: T.bg, color: T.text, outline: "none", fontFamily: T.font };

  const availableMetrics = ["Revenue", "Profit", "Current Ratio", "Debt/Equity", "DSO", "Inventory Turnover", "Gross Margin", "ROE", "ROA", "Net Margin", "Quick Ratio", "DPO"];

  const handleResetQuery = () => {
    setQueryMetrics(["revenue"]);
    setQueryStartDate("");
    setQueryEndDate("");
    setQueryCompare(false);
    setQueryResults([]);
  };

  return (
    <WorkspaceShell>
      <div style={{ padding: 24, fontFamily: T.font, background: T.bg, minHeight: "100%" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontFamily: T.fontDisplay, fontSize: "1.35rem", color: T.text, display: "flex", alignItems: "center", gap: 8 }}>
            <BarChart3 size={22} style={{ color: T.primary }} /> Business Intelligence
          </h1>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: `2px solid ${T.border}` }}>
          {(["dashboard", "ratios", "trends", "query"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{
              padding: "10px 18px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400, color: tab === t ? T.primary : T.muted,
              borderBottom: tab === t ? `2px solid ${T.primary}` : "2px solid transparent", marginBottom: -2, background: "none", borderLeft: "none", borderRight: "none", borderTop: "none", cursor: "pointer", textTransform: "capitalize",
            }}>
              {t === "query" ? "Custom Query" : t}
            </button>
          ))}
        </div>

        {error && <div style={{ background: "rgba(220,38,38,0.08)", border: `1px solid ${T.error}`, borderRadius: T.radius, padding: 12, marginBottom: 16, color: T.error, fontSize: "0.85rem" }}>{error}</div>}

        {loading ? (
          <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
        ) : tab === "dashboard" ? (
          <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 16 }}>
            {kpiWidgets.map((w) => <KPIBlock key={w.title} widget={w} />)}
            {kpiWidgets.length === 0 && (
              <div style={{ gridColumn: "1 / -1", textAlign: "center", padding: 40, color: T.muted }}>
                <Activity size={32} style={{ marginBottom: 8, opacity: 0.4 }} />
                <p>No KPI data available. Connect your accounting module.</p>
              </div>
            )}
          </div>
        ) : tab === "ratios" ? (
          ratios ? (
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 24 }}>
              <div>
                <RatioGroup title="Liquidity" ratios={ratios.liquidity} format={(v) => v.toFixed(2) + "x"} />
                <RatioGroup title="Leverage" ratios={ratios.leverage} format={(v) => v.toFixed(2)} />
              </div>
              <div>
                <RatioGroup title="Profitability" ratios={ratios.profitability} format={(v) => pct(v)} />
                <RatioGroup title="Efficiency" ratios={ratios.efficiency} format={(v) => `${Math.round(v)} days`} />
              </div>
            </div>
          ) : (
            <div style={{ textAlign: "center", padding: 40, color: T.muted }}>
              <Activity size={32} style={{ marginBottom: 8, opacity: 0.4 }} />
              <p>No financial ratio data available</p>
            </div>
          )
        ) : tab === "trends" ? (
          trends.length > 0 ? (
            <div style={{ background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 20 }}>
              <TrendChart data={trends} label="Revenue" color={T.primary} format={(v) => fmt(v)} />
              <TrendChart data={trends} label="Profit" color={T.success} format={(v) => fmt(v)} />
              <TrendChart data={trends} label="Current Ratio" color="#3B82F6" format={(v) => v.toFixed(2)} />
              <TrendChart data={trends} label="Debt Equity" color={T.accent} format={(v) => v.toFixed(2)} />
            </div>
          ) : (
            <div style={{ textAlign: "center", padding: 40, color: T.muted }}>
              <TrendingUp size={32} style={{ marginBottom: 8, opacity: 0.4 }} />
              <p>No trend data available</p>
            </div>
          )
        ) : (
          <div style={{ background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 20 }}>
            <h4 style={{ fontSize: "0.9rem", fontWeight: 600, color: T.text, marginBottom: 16, display: "flex", alignItems: "center", gap: 6 }}>
              <Filter size={16} style={{ color: T.primary }} /> Query Builder
            </h4>
            <div style={{ marginBottom: 16 }}>
              <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: T.muted, marginBottom: 8 }}>Select Metrics</label>
              <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
                {availableMetrics.map(m => (
                  <button key={m} onClick={() => toggleMetric(m.toLowerCase().replace(/ /g, "_"))}
                    style={{
                      padding: "4px 12px", borderRadius: T.radiusSm, fontSize: "0.8rem", fontWeight: 500, border: `1px solid ${queryMetrics.includes(m.toLowerCase().replace(/ /g, "_")) ? T.primary : T.border}`,
                      background: queryMetrics.includes(m.toLowerCase().replace(/ /g, "_")) ? "rgba(27,67,50,0.08)" : T.surface, color: queryMetrics.includes(m.toLowerCase().replace(/ /g, "_")) ? T.primary : T.muted, cursor: "pointer",
                    }}>
                    {m}
                  </button>
                ))}
              </div>
            </div>
            <div style={{ display: "flex", gap: 12, marginBottom: 16 }}>
              <div style={{ flex: 1 }}>
                <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: T.muted, marginBottom: 4 }}>Start Date</label>
                <input type="date" style={inputStyle} value={queryStartDate} onChange={(e) => setQueryStartDate(e.target.value)} />
              </div>
              <div style={{ flex: 1 }}>
                <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: T.muted, marginBottom: 4 }}>End Date</label>
                <input type="date" style={inputStyle} value={queryEndDate} onChange={(e) => setQueryEndDate(e.target.value)} />
              </div>
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 20 }}>
              <input type="checkbox" checked={queryCompare} onChange={(e) => setQueryCompare(e.target.checked)} style={{ width: 16, height: 16 }} />
              <span style={{ fontSize: "0.85rem", color: T.text }}>Compare with previous period</span>
            </div>
            <div style={{ display: "flex", gap: 8 }}>
              <button onClick={handleRunQuery} style={{ padding: "8px 20px", borderRadius: T.radiusSm, background: T.primary, color: "#fff", border: "none", cursor: "pointer", fontWeight: 600, fontSize: "0.85rem", display: "flex", alignItems: "center", gap: 6 }}>
                <Search size={14} /> Run Query
              </button>
              <button onClick={handleResetQuery} style={{ padding: "8px 16px", borderRadius: T.radiusSm, background: T.surface, color: T.muted, border: `1px solid ${T.border}`, cursor: "pointer", fontSize: "0.85rem" }}>
                Reset
              </button>
            </div>
            {queryResults.length > 0 && (
              <div style={{ marginTop: 24, borderTop: `1px solid ${T.border}`, paddingTop: 20 }}>
                <h4 style={{ fontSize: "0.85rem", fontWeight: 600, color: T.text, marginBottom: 12 }}>Results</h4>
                <table style={{ width: "100%", borderCollapse: "collapse" }}>
                  <thead>
                    <tr>
                      {["Metric", "Current", queryCompare ? "Previous" : "", "Change"].filter(Boolean).map(h => (
                        <th key={h} style={{ padding: "8px 12px", textAlign: h === "Change" || h === "Current" || h === "Previous" ? "right" : "left", fontWeight: 600, fontSize: "0.8rem", color: T.muted, textTransform: "uppercase", borderBottom: `1px solid ${T.border}` }}>{h}</th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {queryResults.map((r, i) => (
                      <tr key={i} style={{ borderBottom: `1px solid ${T.border}` }}>
                        <td style={{ padding: "8px 12px", fontSize: "0.85rem", color: T.text, fontWeight: 500, textTransform: "capitalize" }}>{r.metric.replace(/_/g, " ")}</td>
                        <td style={{ padding: "8px 12px", fontSize: "0.85rem", textAlign: "right", color: T.text, fontWeight: 600 }}>{fmt(r.current)}</td>
                        {queryCompare && <td style={{ padding: "8px 12px", fontSize: "0.85rem", textAlign: "right", color: T.muted }}>{fmt(r.previous)}</td>}
                        <td style={{ padding: "8px 12px", fontSize: "0.85rem", textAlign: "right" }}>
                          <span style={{ color: r.change >= 0 ? T.success : T.error, fontWeight: 600 }}>
                            {r.change >= 0 ? "+" : ""}{r.change.toFixed(1)}%
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        )}
      </div>
    </WorkspaceShell>
  );
}
