"use client";

import { useEffect, useState } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { DataTable, Column } from "@/components/ui/data-table";
import { getToken } from "@/lib/session";
import { apiGet } from "@/lib/api";
import { Users, Plus, Phone, Mail, Calendar, CheckCircle } from "lucide-react";

interface Contact {
  id: string;
  contact_type: string;
  name: string;
  organization: string;
  email: string;
  phone: string;
  stage: string;
  estimated_value: number;
  assigned_to: string;
}

interface Deal {
  id: string;
  name: string;
  contact_name: string;
  value: number;
  probability: number;
  stage: string;
  expected_close_date: string;
  assigned_to: string;
}

interface Activity {
  id: string;
  activity_type: string;
  subject: string;
  contact_name: string;
  status: string;
  due_date: string;
  assigned_to: string;
}

const PIPELINE_STAGES = ["prospecting", "qualification", "proposal", "negotiation", "closed_won", "closed_lost"];

export default function CrmPage() {
  const token = getToken();
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [deals, setDeals] = useState<Deal[]>([]);
  const [activities, setActivities] = useState<Activity[]>([]);
  const [tab, setTab] = useState<"contacts" | "pipeline" | "activities">("contacts");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const [cRes, dRes, aRes] = await Promise.all([
          apiGet("/crm/contacts", token),
          apiGet("/crm/deals", token),
          apiGet("/crm/activities", token),
        ]);
        if (cRes.ok) setContacts((await cRes.json()).data || []);
        if (dRes.ok) setDeals((await dRes.json()).data || []);
        if (aRes.ok) setActivities((await aRes.json()).data || []);
      } catch {
        // offline
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [token]);

  const contactColumns: Column<Contact>[] = [
    { key: "name", label: "Name", sortable: true },
    { key: "organization", label: "Organization", width: "160px" },
    { key: "contact_type", label: "Type", width: "80px" },
    { key: "email", label: "Email", width: "180px" },
    { key: "stage", label: "Stage", width: "110px" },
    { key: "estimated_value", label: "Value", align: "right", width: "120px", render: (v: number) => v ? `₦${v.toLocaleString("en-NG")}` : "-" },
  ];

  const activityColumns: Column<Activity>[] = [
    { key: "subject", label: "Subject", sortable: true },
    { key: "activity_type", label: "Type", width: "80px", render: (v: string) => { const icons: Record<string, typeof Phone> = { call: Phone, email: Mail, meeting: Calendar, task: CheckCircle }; const Icon = icons[v] || CheckCircle; return <span style={{ display: "flex", alignItems: "center", gap: 4 }}><Icon size={12} />{v}</span>; } },
    { key: "contact_name", label: "Contact", width: "140px" },
    { key: "status", label: "Status", width: "90px", render: (v: string) => { const c: Record<string, { bg: string; color: string }> = { completed: { bg: "rgba(25,135,84,0.12)", color: "#198754" }, pending: { bg: "rgba(255,193,7,0.12)", color: "#B8860B" }, cancelled: { bg: "rgba(134,142,150,0.12)", color: "#868E96" } }; const s = c[v] || c.pending; return <span style={{ fontSize: "0.75rem", padding: "2px 8px", borderRadius: 4, background: s.bg, color: s.color }}>{v}</span>; } },
    { key: "due_date", label: "Due", width: "110px" },
  ];

  const stageTotals = PIPELINE_STAGES.map((stage) => ({
    stage,
    deals: deals.filter((d) => d.stage === stage),
    total: deals.filter((d) => d.stage === stage).reduce((s, d) => s + d.value, 0),
  }));

  return (
    <WorkspaceShell>
      <div style={{ padding: 24 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h1 style={{ fontSize: "1.25rem", fontWeight: 600, color: "#1A1A2E" }}>
            <Users size={20} style={{ verticalAlign: "middle", marginRight: 8 }} /> CRM
          </h1>
          <button className="btn btn-primary" style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <Plus size={16} /> New Contact
          </button>
        </div>

        <div style={{ display: "flex", gap: 4, marginBottom: 20, borderBottom: "2px solid #E9ECEF" }}>
          {(["contacts", "pipeline", "activities"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{ padding: "10px 16px", fontSize: "0.85rem", fontWeight: tab === t ? 600 : 400, color: tab === t ? "#1B4332" : "#868E96", borderBottom: tab === t ? "2px solid #1B4332" : "2px solid transparent", marginBottom: -2, textTransform: "capitalize" }}>
              {t}
            </button>
          ))}
        </div>

        {loading ? <div style={{ display: "flex", justifyContent: "center", padding: 40 }}><div className="splash-loader" /></div>
          : tab === "contacts" ? (
            <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
              <DataTable columns={contactColumns} data={contacts} pageSize={15} emptyMessage="No contacts" />
            </div>
          ) : tab === "pipeline" ? (
            <div style={{ display: "flex", gap: 12, overflowX: "auto", paddingBottom: 8 }}>
              {stageTotals.map(({ stage, deals: stageDeals, total }) => (
                <div key={stage} style={{ minWidth: 240, flex: 1, background: "#F8F9FA", border: "1px solid #E9ECEF", borderRadius: 12, padding: 16 }}>
                  <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 12 }}>
                    <span style={{ fontSize: "0.8rem", fontWeight: 600, color: "#1A1A2E", textTransform: "capitalize" }}>{stage.replace("_", " ")}</span>
                    <span style={{ fontSize: "0.7rem", padding: "1px 6px", borderRadius: 4, background: "rgba(27,67,50,0.12)", color: "#1B4332" }}>{stageDeals.length}</span>
                  </div>
                  <div style={{ fontSize: "0.85rem", fontWeight: 600, color: "#1B4332", marginBottom: 12 }}>₦{total.toLocaleString("en-NG")}</div>
                  {stageDeals.map((deal) => (
                    <div key={deal.id} style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 8, padding: 10, marginBottom: 8, boxShadow: "0 1px 2px rgba(0,0,0,0.04)" }}>
                      <div style={{ fontSize: "0.8rem", fontWeight: 500, color: "#1A1A2E" }}>{deal.name}</div>
                      <div style={{ fontSize: "0.75rem", color: "#868E96" }}>{deal.contact_name}</div>
                      <div style={{ fontSize: "0.8rem", fontWeight: 600, color: "#495057", marginTop: 4 }}>₦{deal.value.toLocaleString("en-NG")} · {deal.probability}%</div>
                    </div>
                  ))}
                </div>
              ))}
            </div>
          ) : (
            <div style={{ background: "#FFFFFF", border: "1px solid #E9ECEF", borderRadius: 12, padding: 20, boxShadow: "0 2px 8px rgba(0,0,0,0.08)" }}>
              <DataTable columns={activityColumns} data={activities} pageSize={15} emptyMessage="No activities" />
            </div>
          )}
      </div>
    </WorkspaceShell>
  );
}
