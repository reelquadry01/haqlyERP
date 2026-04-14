// Author: Quadri Atharu
"use client";

import { useEffect, useState, useCallback } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { KPICard } from "@/components/ui/kpi-card";
import { getToken } from "@/lib/session";
import { apiGet, apiPost } from "@/lib/api";
import { Users, Plus, FileText, Play, X, AlertCircle, BookOpen } from "lucide-react";

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

interface Employee {
  id: string;
  full_name: string;
  department: string;
  designation: string;
  salary_amount: number;
  start_date: string;
  is_active: boolean;
  bank_name?: string;
  bank_account?: string;
  tax_id?: string;
  pension_provider?: string;
}

interface PayrollRun {
  id: string;
  run_name: string;
  period_month: number;
  period_year: number;
  total_gross: number;
  total_paye: number;
  total_pension: number;
  total_nhf: number;
  total_nsitf: number;
  total_net: number;
  status: string;
  employee_count: number;
}

interface Payslip {
  id: string;
  employee_name: string;
  net_pay: number;
  date: string;
  run_id: string;
  gross_earnings: number;
  paye: number;
  pension: number;
  nhf: number;
  nsitf: number;
  other_deductions: number;
}

const MONTHS = ["January","February","March","April","May","June","July","August","September","October","November","December"];
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

export default function HrPayrollPage() {
  const token = getToken();
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [payrollRuns, setPayrollRuns] = useState<PayrollRun[]>([]);
  const [payslips, setPayslips] = useState<Payslip[]>([]);
  const [selectedPayslip, setSelectedPayslip] = useState<Payslip | null>(null);
  const [tab, setTab] = useState<"employees" | "run" | "payslips">("employees");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddEmployee, setShowAddEmployee] = useState(false);
  const [showRunPayroll, setShowRunPayroll] = useState(false);
  const [runMonth, setRunMonth] = useState(new Date().getMonth() + 1);
  const [runYear, setRunYear] = useState(new Date().getFullYear());
  const [runPreview, setRunPreview] = useState<PayrollRun | null>(null);
  const [processing, setProcessing] = useState(false);
  const [posting, setPosting] = useState(false);
  const [payeYtd, setPayeYtd] = useState(0);
  const [nextFiling, setNextFiling] = useState("");
  const [newEmp, setNewEmp] = useState({ full_name: "", department: "", designation: "", salary_amount: "", bank_name: "", bank_account: "", tax_id: "", pension_provider: "" });

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [empRes, runRes, payeRes] = await Promise.all([
        apiGet("/payroll/employees", token),
        apiGet("/payroll/runs", token),
        apiGet("/payroll/paye-summary", token),
      ]);
      if (empRes.ok) setEmployees((await empRes.json()).data || []);
      if (runRes.ok) setPayrollRuns((await runRes.json()).data || []);
      if (payeRes.ok) {
        const d = await payeRes.json();
        setPayeYtd(d.data?.total_paye_ytd || 0);
        setNextFiling(d.data?.next_filing_deadline || "Not set");
      }
    } catch {
      setError("Failed to load payroll data");
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadData(); }, [loadData]);

  const loadPayslips = useCallback(async (runId: string) => {
    try {
      const res = await apiGet(`/payroll/runs/${runId}/payslips`, token);
      if (res.ok) setPayslips((await res.json()).data || []);
    } catch { /* */ }
  }, [token]);

  const handleAddEmployee = async () => {
    try {
      const res = await apiPost("/payroll/employees", { ...newEmp, salary_amount: Number(newEmp.salary_amount) }, token);
      if (res.ok) {
        setShowAddEmployee(false);
        setNewEmp({ full_name: "", department: "", designation: "", salary_amount: "", bank_name: "", bank_account: "", tax_id: "", pension_provider: "" });
        loadData();
      }
    } catch { setError("Failed to add employee"); }
  };

  const handlePreviewPayroll = async () => {
    try {
      const res = await apiPost("/payroll/runs", { period_month: runMonth, period_year: runYear, preview: true }, token);
      if (res.ok) setRunPreview((await res.json()).data || null);
    } catch { setError("Failed to preview payroll"); }
  };

  const handleProcessPayroll = async () => {
    if (!runPreview) return;
    setProcessing(true);
    try {
      const res = await apiPost(`/payroll/runs/${runPreview.id}/process`, {}, token);
      if (res.ok) { setShowRunPayroll(false); setRunPreview(null); loadData(); }
    } catch { setError("Failed to process payroll"); }
    setProcessing(false);
  };

  const handlePostToGL = async (runId: string) => {
    setPosting(true);
    try {
      const res = await apiPost(`/payroll/runs/${runId}/post`, {}, token);
      if (res.ok) loadData();
    } catch { setError("Failed to post to GL"); }
    setPosting(false);
  };

  const handleViewPayslip = async (payslipId: string) => {
    try {
      const res = await apiGet(`/payroll/payslip/${payslipId}`, token);
      if (res.ok) setSelectedPayslip((await res.json()).data || null);
    } catch { /* */ }
  };

  const employeeColumns: Column<Employee>[] = [
    { key: "full_name", label: "Name", sortable: true },
    { key: "department", label: "Department", width: "120px" },
    { key: "designation", label: "Designation", width: "130px" },
    { key: "salary_amount", label: "Gross Salary", align: "right", width: "130px", sortable: true, render: (v: number) => fmt(v) },
    { key: "start_date", label: "Start Date", width: "110px" },
    {
      key: "is_active", label: "Status", width: "90px",
      render: (v: boolean) => (
        <span style={{ fontSize: "0.75rem", padding: "2px 10px", borderRadius: 12, fontWeight: 600, background: v ? "rgba(22,163,74,0.12)" : "rgba(220,38,38,0.12)", color: v ? T.success : T.error }}>
          {v ? "Active" : "Inactive"}
        </span>
      ),
    },
  ];

  const payslipColumns: Column<Payslip>[] = [
    { key: "employee_name", label: "Employee", sortable: true },
    {
      key: "gross_earnings", label: "Gross", align: "right", width: "110px",
      render: (v: number) => <span style={{ color: T.primary, fontWeight: 600 }}>{fmt(v)}</span>,
    },
    { key: "net_pay", label: "Net Pay", align: "right", width: "130px", render: (v: number) => fmt(v) },
    {
      key: "paye", label: "PAYE", align: "right", width: "100px",
      render: (v: number) => <span style={{ color: T.error }}>{fmt(v)}</span>,
    },
    {
      key: "pension", label: "Pension", align: "right", width: "100px",
      render: (v: number) => <span style={{ color: T.error }}>{fmt(v)}</span>,
    },
    { key: "date", label: "Date", width: "120px" },
  ];

  const runColumns: Column<PayrollRun>[] = [
    { key: "run_name", label: "Run Name", sortable: true },
    { key: "period_month", label: "Month", width: "80px", render: (v: number) => MONTHS[v - 1] || v },
    { key: "period_year", label: "Year", width: "70px" },
    { key: "employee_count", label: "Employees", align: "right", width: "90px" },
    { key: "total_gross", label: "Gross", align: "right", width: "130px", render: (v: number) => fmt(v) },
    { key: "total_net", label: "Net Pay", align: "right", width: "130px", render: (v: number) => fmt(v) },
    {
      key: "status", label: "Status", width: "100px",
      render: (v: string) => {
        const map: Record<string, { bg: string; color: string }> = {
          draft: { bg: "rgba(107,114,128,0.12)", color: T.muted },
          processing: { bg: "rgba(59,130,246,0.12)", color: "#3B82F6" },
          completed: { bg: "rgba(22,163,74,0.12)", color: T.success },
          posted: { bg: "rgba(27,67,50,0.12)", color: T.primary },
        };
        const c = map[v] || map.draft;
        return <span style={{ fontSize: "0.75rem", padding: "2px 10px", borderRadius: 12, fontWeight: 600, background: c.bg, color: c.color }}>{v}</span>;
      },
    },
  ];

  const btnPrimary: React.CSSProperties = { display: "inline-flex", alignItems: "center", gap: 6, padding: "8px 16px", borderRadius: T.radius, fontSize: "0.85rem", fontWeight: 600, background: T.primary, color: "#fff", border: "none", cursor: "pointer" };
  const btnAccent: React.CSSProperties = { display: "inline-flex", alignItems: "center", gap: 6, padding: "8px 16px", borderRadius: T.radius, fontSize: "0.85rem", fontWeight: 600, background: T.accent, color: T.text, border: "none", cursor: "pointer" };
  const btnSecondary: React.CSSProperties = { padding: "8px 16px", borderRadius: T.radiusSm, border: `1px solid ${T.border}`, background: T.surface, color: T.text, cursor: "pointer", fontSize: "0.85rem" };
  const btnSubmit: React.CSSProperties = { padding: "8px 20px", borderRadius: T.radiusSm, background: T.primary, color: "#fff", border: "none", cursor: "pointer", fontWeight: 600, fontSize: "0.85rem" };

  return (
    <WorkspaceShell>
      <div style={{ padding: 24, fontFamily: T.font, background: T.bg, minHeight: "100%" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontFamily: T.fontDisplay, fontSize: "1.35rem", color: T.text, display: "flex", alignItems: "center", gap: 8 }}>
            <Users size={22} style={{ color: T.primary }} /> HR &amp; Payroll
          </h1>
          <div style={{ display: "flex", gap: 8 }}>
            <button onClick={() => setShowAddEmployee(true)} style={btnPrimary}><Plus size={16} /> Add Employee</button>
            <button onClick={() => setShowRunPayroll(true)} style={btnAccent}><Play size={16} /> Run Payroll</button>
          </div>
        </div>

        <div style={{ display: "flex", gap: 16, marginBottom: 20 }}>
          <div style={{ flex: 1, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
            <div style={{ fontSize: "0.75rem", color: T.muted, fontWeight: 500, marginBottom: 4 }}>PAYE Remitted YTD</div>
            <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.primary }}>{fmt(payeYtd)}</div>
          </div>
          <div style={{ flex: 1, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
            <div style={{ fontSize: "0.75rem", color: T.muted, fontWeight: 500, marginBottom: 4 }}>Next Filing Deadline</div>
            <div style={{ fontSize: "1.1rem", fontWeight: 600, color: T.accent, display: "flex", alignItems: "center", gap: 6 }}>
              <AlertCircle size={16} /> {nextFiling || "Not configured"}
            </div>
          </div>
          <div style={{ flex: 1, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
            <div style={{ fontSize: "0.75rem", color: T.muted, fontWeight: 500, marginBottom: 4 }}>Active Employees</div>
            <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.success }}>{employees.filter(e => e.is_active).length}</div>
          </div>
          <div style={{ flex: 1, background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16 }}>
            <div style={{ fontSize: "0.75rem", color: T.muted, fontWeight: 500, marginBottom: 4 }}>Total Payroll Runs</div>
            <div style={{ fontSize: "1.3rem", fontWeight: 700, color: T.primary }}>{payrollRuns.length}</div>
          </div>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: `2px solid ${T.border}` }}>
          {(["employees", "run", "payslips"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{
              padding: "10px 18px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400, color: tab === t ? T.primary : T.muted,
              borderBottom: tab === t ? `2px solid ${T.primary}` : "2px solid transparent", marginBottom: -2, background: "none", borderLeft: "none", borderRight: "none", borderTop: "none", cursor: "pointer",
            }}>
              {t === "run" ? "Run Payroll" : t.charAt(0).toUpperCase() + t.slice(1)}
            </button>
          ))}
        </div>

        {error && <div style={{ background: "rgba(220,38,38,0.08)", border: `1px solid ${T.error}`, borderRadius: T.radius, padding: 12, marginBottom: 16, color: T.error, fontSize: "0.85rem" }}>{error}</div>}

        <div style={{ background: T.surface, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 20, boxShadow: "0 1px 3px rgba(0,0,0,0.06)" }}>
          {loading ? (
            <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
          ) : tab === "employees" ? (
            <DataTable columns={employeeColumns} data={employees} pageSize={15} emptyMessage="No employees found" />
          ) : tab === "run" ? (
            payrollRuns.length === 0 ? (
              <div style={{ textAlign: "center", padding: 40, color: T.muted }}>
                <FileText size={32} style={{ marginBottom: 8, opacity: 0.4 }} />
                <p>No payroll runs yet. Click &quot;Run Payroll&quot; to get started.</p>
              </div>
            ) : (
              <div>
                <DataTable columns={runColumns} data={payrollRuns} pageSize={10} emptyMessage="No payroll runs"
                  actions={[
                    { label: "View Payslips", icon: <FileText size={14} />, onClick: (row: PayrollRun) => { loadPayslips(row.id); setTab("payslips"); } },
                    { label: "Post to GL", icon: <BookOpen size={14} />, onClick: (row: PayrollRun) => handlePostToGL(row.id) },
                  ]}
                />
                <div style={{ marginTop: 16, display: "flex", gap: 8, justifyContent: "flex-end" }}>
                  {payrollRuns.filter(r => r.status === "completed").map(r => (
                    <button key={r.id} onClick={() => handlePostToGL(r.id)} disabled={posting} style={btnAccent}>
                      <BookOpen size={14} /> Post &quot;{r.run_name}&quot; to GL
                    </button>
                  ))}
                </div>
              </div>
            )
          ) : (
            payslips.length === 0 ? (
              <div style={{ textAlign: "center", padding: 40, color: T.muted }}>
                <FileText size={32} style={{ marginBottom: 8, opacity: 0.4 }} />
                <p>Select a payroll run to view payslips</p>
              </div>
            ) : (
              <DataTable columns={payslipColumns} data={payslips} pageSize={15} emptyMessage="No payslips"
                onRowClick={(row: Payslip) => handleViewPayslip(row.id)}
              />
            )
          )}
        </div>

        <Modal open={showAddEmployee} onClose={() => setShowAddEmployee(false)} title="Add Employee">
          <Field label="Full Name"><input style={inputStyle} value={newEmp.full_name} onChange={(e) => setNewEmp({ ...newEmp, full_name: e.target.value })} /></Field>
          <div style={{ display: "flex", gap: 12 }}>
            <div style={{ flex: 1 }}>
              <Field label="Department"><input style={inputStyle} value={newEmp.department} onChange={(e) => setNewEmp({ ...newEmp, department: e.target.value })} /></Field>
            </div>
            <div style={{ flex: 1 }}>
              <Field label="Designation"><input style={inputStyle} value={newEmp.designation} onChange={(e) => setNewEmp({ ...newEmp, designation: e.target.value })} /></Field>
            </div>
          </div>
          <Field label="Gross Salary"><input type="number" style={inputStyle} value={newEmp.salary_amount} onChange={(e) => setNewEmp({ ...newEmp, salary_amount: e.target.value })} placeholder="₦0" /></Field>
          <div style={{ display: "flex", gap: 12 }}>
            <div style={{ flex: 1 }}>
              <Field label="Bank Name"><input style={inputStyle} value={newEmp.bank_name} onChange={(e) => setNewEmp({ ...newEmp, bank_name: e.target.value })} /></Field>
            </div>
            <div style={{ flex: 1 }}>
              <Field label="Bank Account"><input style={inputStyle} value={newEmp.bank_account} onChange={(e) => setNewEmp({ ...newEmp, bank_account: e.target.value })} /></Field>
            </div>
          </div>
          <div style={{ display: "flex", gap: 12 }}>
            <div style={{ flex: 1 }}>
              <Field label="Tax ID (TIN)"><input style={inputStyle} value={newEmp.tax_id} onChange={(e) => setNewEmp({ ...newEmp, tax_id: e.target.value })} /></Field>
            </div>
            <div style={{ flex: 1 }}>
              <Field label="Pension Provider"><input style={inputStyle} value={newEmp.pension_provider} onChange={(e) => setNewEmp({ ...newEmp, pension_provider: e.target.value })} /></Field>
            </div>
          </div>
          <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 20 }}>
            <button onClick={() => setShowAddEmployee(false)} style={btnSecondary}>Cancel</button>
            <button onClick={handleAddEmployee} style={btnSubmit}>Add Employee</button>
          </div>
        </Modal>

        <Modal open={showRunPayroll} onClose={() => { setShowRunPayroll(false); setRunPreview(null); }} title="Run Payroll">
          <div style={{ display: "flex", gap: 12, marginBottom: 20 }}>
            <div style={{ flex: 1 }}>
              <Field label="Month">
                <select style={selectStyle} value={runMonth} onChange={(e) => setRunMonth(Number(e.target.value))}>
                  {MONTHS.map((m, i) => <option key={i} value={i + 1}>{m}</option>)}
                </select>
              </Field>
            </div>
            <div style={{ flex: 1 }}>
              <Field label="Year">
                <input type="number" style={inputStyle} value={runYear} onChange={(e) => setRunYear(Number(e.target.value))} />
              </Field>
            </div>
          </div>
          <button onClick={handlePreviewPayroll} style={{ width: "100%", padding: "10px", borderRadius: T.radiusSm, background: T.primary, color: "#fff", border: "none", cursor: "pointer", fontWeight: 600, marginBottom: 20 }}>Preview Payroll</button>
          {runPreview && (
            <div style={{ background: T.bg, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 16, marginBottom: 16 }}>
              <h4 style={{ fontSize: "0.9rem", fontWeight: 600, color: T.text, marginBottom: 12 }}>Payroll Summary — {MONTHS[runMonth - 1]} {runYear}</h4>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
                {([
                  ["Total Gross", runPreview.total_gross],
                  ["Total PAYE", runPreview.total_paye],
                  ["Total Pension", runPreview.total_pension],
                  ["Total NHF", runPreview.total_nhf],
                  ["Total NSITF", runPreview.total_nsitf],
                  ["Total Net Pay", runPreview.total_net],
                ] as [string, number][]).map(([label, val]) => (
                  <div key={label} style={{ display: "flex", justifyContent: "space-between", padding: "6px 0", borderBottom: `1px solid ${T.border}` }}>
                    <span style={{ fontSize: "0.85rem", color: T.muted }}>{label}</span>
                    <span style={{ fontSize: "0.85rem", fontWeight: 600, color: T.text }}>{fmt(val)}</span>
                  </div>
                ))}
              </div>
              <div style={{ fontSize: "0.8rem", color: T.muted, marginTop: 8 }}>{runPreview.employee_count} employee(s)</div>
              <button onClick={handleProcessPayroll} disabled={processing} style={{ width: "100%", marginTop: 16, padding: "10px", borderRadius: T.radiusSm, background: T.accent, color: T.text, border: "none", cursor: processing ? "wait" : "pointer", fontWeight: 600 }}>
                {processing ? "Processing..." : "Process & Generate Payslips"}
              </button>
            </div>
          )}
        </Modal>

        <Modal open={!!selectedPayslip} onClose={() => setSelectedPayslip(null)} title="Payslip Detail">
          {selectedPayslip && (
            <div>
              <div style={{ marginBottom: 16, paddingBottom: 12, borderBottom: `1px solid ${T.border}` }}>
                <div style={{ fontSize: "1rem", fontWeight: 600, color: T.text }}>{selectedPayslip.employee_name}</div>
                <div style={{ fontSize: "0.8rem", color: T.muted, marginTop: 2 }}>Pay Period: {selectedPayslip.date}</div>
                <div style={{ fontSize: "0.8rem", color: T.muted, marginTop: 2 }}>Run ID: {selectedPayslip.run_id}</div>
              </div>
              <div style={{ background: T.bg, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 12, marginBottom: 16 }}>
                <h4 style={{ fontSize: "0.85rem", fontWeight: 600, color: T.primary, marginBottom: 8 }}>Earnings</h4>
                <div style={{ display: "flex", justifyContent: "space-between", padding: "6px 0", borderBottom: `1px solid ${T.border}` }}>
                  <span style={{ fontSize: "0.85rem", color: T.muted }}>Basic Salary</span>
                  <span style={{ fontSize: "0.85rem", fontWeight: 600, color: T.text }}>{fmt(selectedPayslip.gross_earnings * 0.7)}</span>
                </div>
                <div style={{ display: "flex", justifyContent: "space-between", padding: "6px 0", borderBottom: `1px solid ${T.border}` }}>
                  <span style={{ fontSize: "0.85rem", color: T.muted }}>Housing Allowance</span>
                  <span style={{ fontSize: "0.85rem", fontWeight: 600, color: T.text }}>{fmt(selectedPayslip.gross_earnings * 0.2)}</span>
                </div>
                <div style={{ display: "flex", justifyContent: "space-between", padding: "6px 0", borderBottom: `1px solid ${T.border}` }}>
                  <span style={{ fontSize: "0.85rem", color: T.muted }}>Transport Allowance</span>
                  <span style={{ fontSize: "0.85rem", fontWeight: 600, color: T.text }}>{fmt(selectedPayslip.gross_earnings * 0.1)}</span>
                </div>
                <div style={{ display: "flex", justifyContent: "space-between", padding: "8px 0", marginTop: 4, borderTop: `1px dashed ${T.border}` }}>
                  <span style={{ fontSize: "0.85rem", fontWeight: 600, color: T.primary }}>Total Gross Earnings</span>
                  <span style={{ fontSize: "0.85rem", fontWeight: 600, color: T.primary }}>{fmt(selectedPayslip.gross_earnings)}</span>
                </div>
              </div>
              <div style={{ background: T.bg, border: `1px solid ${T.border}`, borderRadius: T.radius, padding: 12, marginBottom: 16 }}>
                <h4 style={{ fontSize: "0.85rem", fontWeight: 600, color: T.error, marginBottom: 8 }}>Deductions</h4>
                {([
                  ["PAYE Tax", selectedPayslip.paye],
                  ["Pension (8%)", selectedPayslip.pension],
                  ["NHF (2.5%)", selectedPayslip.nhf],
                  ["NSITF", selectedPayslip.nsitf],
                  ["Other Deductions", selectedPayslip.other_deductions],
                ] as [string, number][]).map(([label, val]) => (
                  <div key={label} style={{ display: "flex", justifyContent: "space-between", padding: "6px 0", borderBottom: `1px solid ${T.border}` }}>
                    <span style={{ fontSize: "0.85rem", color: T.muted }}>{label}</span>
                    <span style={{ fontSize: "0.85rem", fontWeight: 600, color: T.error }}>-{fmt(val)}</span>
                  </div>
                ))}
                <div style={{ display: "flex", justifyContent: "space-between", padding: "8px 0", marginTop: 4, borderTop: `1px dashed ${T.border}` }}>
                  <span style={{ fontSize: "0.85rem", fontWeight: 600, color: T.error }}>Total Deductions</span>
                  <span style={{ fontSize: "0.85rem", fontWeight: 600, color: T.error }}>-{fmt(selectedPayslip.paye + selectedPayslip.pension + selectedPayslip.nhf + selectedPayslip.nsitf + selectedPayslip.other_deductions)}</span>
                </div>
              </div>
              <div style={{ background: "rgba(27,67,50,0.06)", border: `1px solid ${T.primary}`, borderRadius: T.radius, padding: 16 }}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <span style={{ fontSize: "1.1rem", fontWeight: 700, color: T.primary, fontFamily: T.fontDisplay }}>Net Pay</span>
                  <span style={{ fontSize: "1.3rem", fontWeight: 700, color: T.primary, fontFamily: T.fontDisplay }}>{fmt(selectedPayslip.net_pay)}</span>
                </div>
              </div>
              <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 16 }}>
                <button onClick={() => setSelectedPayslip(null)} style={btnSecondary}>Close</button>
              </div>
            </div>
          )}
        </Modal>
      </div>
    </WorkspaceShell>
  );
}
