// Author: Quadri Atharu
"use client";

import React, { useEffect, useState, useCallback, useRef } from "react";
import { WorkspaceShell } from "@/components/workspace-shell";
import { getToken, getCompanyContext } from "@/lib/session";
import { apiGet, apiDelete } from "@/lib/api";
import {
  FolderOpen, Upload, Download, Trash2, FileText, File,
  Image, FileSpreadsheet, RefreshCw, AlertTriangle, X, Filter,
} from "lucide-react";

interface DocumentAttachment {
  id: string;
  company_id: string;
  entity_type: string;
  entity_id: string;
  file_name: string;
  file_path: string;
  file_size: number;
  mime_type: string;
  description: string | null;
  uploaded_by: string | null;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

const ENTITY_TYPES = [
  { value: "", label: "All Types" },
  { value: "journal", label: "Journal" },
  { value: "invoice", label: "Invoice" },
  { value: "bill", label: "Bill" },
  { value: "voucher", label: "Voucher" },
  { value: "asset", label: "Asset" },
  { value: "employee", label: "Employee" },
  { value: "report", label: "Report" },
  { value: "other", label: "Other" },
];

const ENTITY_COLORS: Record<string, { bg: string; color: string }> = {
  journal: { bg: "rgba(59,130,246,0.12)", color: "#2563EB" },
  invoice: { bg: "rgba(22,163,74,0.12)", color: "#16A34A" },
  bill: { bg: "rgba(220,38,38,0.12)", color: "#DC2626" },
  voucher: { bg: "rgba(212,175,55,0.12)", color: "#D4AF37" },
  asset: { bg: "rgba(139,92,246,0.12)", color: "#8B5CF6" },
  employee: { bg: "rgba(236,72,153,0.12)", color: "#EC4899" },
  report: { bg: "rgba(6,182,212,0.12)", color: "#06B6D4" },
  other: { bg: "rgba(107,114,128,0.12)", color: "#6B7280" },
};

function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function getFileIcon(mime: string) {
  if (mime.startsWith("image/")) return <Image size={16} style={{ color: "#8B5CF6" }} />;
  if (mime.includes("spreadsheet") || mime.includes("csv")) return <FileSpreadsheet size={16} style={{ color: "#16A34A" }} />;
  if (mime.includes("pdf")) return <FileText size={16} style={{ color: "#DC2626" }} />;
  return <File size={16} style={{ color: "#6B7280" }} />;
}

const BASE_URL = typeof window !== 'undefined' && (window as any).__TAURI__
  ? 'http://localhost:8100/api/v1'
  : (process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8100/api/v1');

export default function DocumentsPage() {
  const token = getToken();
  const companyId = getCompanyContext();
  const [documents, setDocuments] = useState<DocumentAttachment[]>([]);
  const [allDocuments, setAllDocuments] = useState<DocumentAttachment[]>([]);
  const [filterType, setFilterType] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [uploading, setUploading] = useState(false);
  const [dragOver, setDragOver] = useState(false);
  const [showUpload, setShowUpload] = useState(false);
  const [uploadEntityType, setUploadEntityType] = useState("other");
  const [uploadEntityId, setUploadEntityId] = useState("");
  const [uploadDescription, setUploadDescription] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await apiGet(`/file-storage?company_id=${companyId}`, token);
      if (!res.ok) throw new Error("Failed to load documents");
      const data = await res.json();
      setAllDocuments(data.data || []);
      setDocuments(data.data || []);
    } catch (e: any) {
      setError(e.message || "Failed to load data");
    } finally {
      setLoading(false);
    }
  }, [token, companyId]);

  useEffect(() => {
    if (companyId) loadData();
  }, [loadData, companyId]);

  useEffect(() => {
    if (!filterType) {
      setDocuments(allDocuments);
    } else {
      setDocuments(allDocuments.filter((d) => d.entity_type === filterType));
    }
  }, [filterType, allDocuments]);

  async function handleUpload(files: FileList | null) {
    if (!files || files.length === 0 || !companyId) return;
    setUploading(true);
    try {
      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        const formData = new FormData();
        formData.append("file", file);
        formData.append("company_id", companyId);
        formData.append("entity_type", uploadEntityType);
        formData.append("entity_id", uploadEntityId || "00000000-0000-0000-0000-000000000000");
        if (uploadDescription) formData.append("description", uploadDescription);

        const response = await fetch(`${BASE_URL}/file-storage/upload`, {
          method: "POST",
          headers: {
            Authorization: `Bearer ${token}`,
          },
          body: formData,
        });
        if (!response.ok) throw new Error(`Upload failed for ${file.name}`);
      }
      setShowUpload(false);
      setUploadDescription("");
      setUploadEntityId("");
      loadData();
    } catch (e: any) {
      setError(e.message);
    } finally {
      setUploading(false);
    }
  }

  async function handleDelete(id: string) {
    try {
      const res = await apiDelete(`/file-storage/${id}`, token);
      if (res.ok) loadData();
    } catch {}
  }

  function handleDownload(doc: DocumentAttachment) {
    const url = `${BASE_URL}/file-storage/${doc.id}/download`;
    const a = document.createElement("a");
    a.href = url;
    a.download = doc.file_name;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
  }

  const totalSize = documents.reduce((s, d) => s + d.file_size, 0);

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
            <FolderOpen size={22} style={{ color: "#1B4332" }} /> Documents
          </h1>
          <button onClick={() => setShowUpload(true)} style={{ display: "flex", alignItems: "center", gap: 6, background: "#1B4332", color: "#FFFFFF", padding: "8px 16px", borderRadius: 6, fontSize: "0.85rem", fontWeight: 500 }}>
            <Upload size={16} /> Upload
          </button>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))", gap: 16, marginBottom: 24 }}>
          <div style={{ background: "#FFFFFF", borderRadius: 8, padding: 20, border: "1px solid #E9ECEF" }}>
            <div style={{ fontSize: "0.8rem", color: "#6B7280", fontWeight: 500, marginBottom: 4 }}>Total Files</div>
            <div style={{ fontSize: "1.5rem", fontWeight: 700, color: "#1A1A2E" }}>{documents.length}</div>
          </div>
          <div style={{ background: "#FFFFFF", borderRadius: 8, padding: 20, border: "1px solid #E9ECEF" }}>
            <div style={{ fontSize: "0.8rem", color: "#6B7280", fontWeight: 500, marginBottom: 4 }}>Total Size</div>
            <div style={{ fontSize: "1.5rem", fontWeight: 700, color: "#1A1A2E" }}>{formatFileSize(totalSize)}</div>
          </div>
          <div style={{ background: "#FFFFFF", borderRadius: 8, padding: 20, border: "1px solid #E9ECEF" }}>
            <div style={{ fontSize: "0.8rem", color: "#6B7280", fontWeight: 500, marginBottom: 4 }}>Entity Types</div>
            <div style={{ fontSize: "1.5rem", fontWeight: 700, color: "#1A1A2E" }}>{new Set(documents.map((d) => d.entity_type)).size}</div>
          </div>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 16 }}>
          <Filter size={16} style={{ color: "#6B7280" }} />
          <select value={filterType} onChange={(e) => setFilterType(e.target.value)} style={{ padding: "8px 12px", fontSize: "0.85rem", border: "1px solid #D1D5DB", borderRadius: 6, background: "#FFFFFF", color: "#1A1A2E", outline: "none", minWidth: 160 }}>
            {ENTITY_TYPES.map((t) => (
              <option key={t.value} value={t.value}>{t.label}</option>
            ))}
          </select>
        </div>

        {loading ? (
          <div style={{ background: "#FFFFFF", borderRadius: 8, border: "1px solid #E9ECEF", padding: 40, textAlign: "center", color: "#6B7280" }}>
            Loading documents...
          </div>
        ) : (
          <div style={{ background: "#FFFFFF", borderRadius: 8, border: "1px solid #E9ECEF", overflow: "hidden" }}>
            <div style={{ overflowX: "auto" }}>
              <table style={{ width: "100%", borderCollapse: "collapse" }}>
                <thead>
                  <tr>
                    {["File Name", "Entity", "Size", "Type", "Uploaded", "Actions"].map((h) => (
                      <th key={h} style={{ padding: "12px 14px", textAlign: h === "Size" || h === "Actions" ? "center" : "left", fontWeight: 600, fontSize: "0.8rem", color: "#6B7280", textTransform: "uppercase", letterSpacing: "0.05em", borderBottom: "2px solid #E9ECEF", background: "#F8F9FA" }}>
                        {h}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {documents.length === 0 ? (
                    <tr>
                      <td colSpan={6} style={{ textAlign: "center", padding: 32, color: "#6B7280" }}>
                        No documents found. Click Upload to add files.
                      </td>
                    </tr>
                  ) : (
                    documents.map((doc) => {
                      const ec = ENTITY_COLORS[doc.entity_type] || ENTITY_COLORS.other;
                      return (
                        <tr key={doc.id} style={{ transition: "background 0.1s" }} onMouseEnter={(e) => (e.currentTarget.style.background = "#F8F9FA")} onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}>
                          <td style={{ padding: "12px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", display: "flex", alignItems: "center", gap: 8 }}>
                            {getFileIcon(doc.mime_type)}
                            <span style={{ color: "#1A1A2E", fontWeight: 500 }}>{doc.file_name}</span>
                          </td>
                          <td style={{ padding: "12px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF" }}>
                            <span style={{ fontSize: "0.75rem", fontWeight: 500, padding: "2px 8px", borderRadius: 4, background: ec.bg, color: ec.color }}>
                              {doc.entity_type}
                            </span>
                          </td>
                          <td style={{ padding: "12px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", textAlign: "center", color: "#6B7280" }}>
                            {formatFileSize(doc.file_size)}
                          </td>
                          <td style={{ padding: "12px 14px", fontSize: "0.75rem", borderBottom: "1px solid #E9ECEF", color: "#6B7280" }}>
                            {doc.mime_type.split("/").pop()}
                          </td>
                          <td style={{ padding: "12px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", color: "#6B7280" }}>
                            {new Date(doc.created_at).toLocaleDateString()}
                          </td>
                          <td style={{ padding: "12px 14px", fontSize: "0.875rem", borderBottom: "1px solid #E9ECEF", textAlign: "center" }}>
                            <div style={{ display: "flex", gap: 8, justifyContent: "center" }}>
                              <button onClick={() => handleDownload(doc)} title="Download" style={{ padding: "4px 8px", borderRadius: 4, background: "#F8F9FA", border: "1px solid #E9ECEF", color: "#1B4332" }}>
                                <Download size={14} />
                              </button>
                              <button onClick={() => handleDelete(doc.id)} title="Delete" style={{ padding: "4px 8px", borderRadius: 4, background: "#F8F9FA", border: "1px solid #E9ECEF", color: "#DC2626" }}>
                                <Trash2 size={14} />
                              </button>
                            </div>
                          </td>
                        </tr>
                      );
                    })
                  )}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {showUpload && (
          <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }} onClick={() => setShowUpload(false)}>
            <div style={{ background: "#FFFFFF", borderRadius: 12, maxWidth: 520, width: "90%", padding: 24 }} onClick={(e) => e.stopPropagation()}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
                <h2 style={{ fontSize: "1.25rem", fontFamily: "'DM Serif Display', serif", color: "#1A1A2E" }}>Upload Document</h2>
                <button onClick={() => setShowUpload(false)} style={{ color: "#6B7280" }}><X size={20} /></button>
              </div>

              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, marginBottom: 16 }}>
                <div>
                  <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#6B7280", marginBottom: 4 }}>Entity Type</label>
                  <select value={uploadEntityType} onChange={(e) => setUploadEntityType(e.target.value)} style={{ width: "100%", padding: "8px 12px", fontSize: "0.85rem", border: "1px solid #D1D5DB", borderRadius: 6, background: "#FFFFFF", color: "#1A1A2E", outline: "none" }}>
                    {ENTITY_TYPES.filter((t) => t.value).map((t) => (
                      <option key={t.value} value={t.value}>{t.label}</option>
                    ))}
                  </select>
                </div>
                <div>
                  <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#6B7280", marginBottom: 4 }}>Entity ID</label>
                  <input value={uploadEntityId} onChange={(e) => setUploadEntityId(e.target.value)} placeholder="UUID (optional)" style={{ width: "100%", padding: "8px 12px", fontSize: "0.85rem", border: "1px solid #D1D5DB", borderRadius: 6, background: "#FFFFFF", color: "#1A1A2E", outline: "none" }} />
                </div>
              </div>

              <div style={{ marginBottom: 16 }}>
                <label style={{ display: "block", fontSize: "0.8rem", fontWeight: 500, color: "#6B7280", marginBottom: 4 }}>Description</label>
                <input value={uploadDescription} onChange={(e) => setUploadDescription(e.target.value)} placeholder="Optional description" style={{ width: "100%", padding: "8px 12px", fontSize: "0.85rem", border: "1px solid #D1D5DB", borderRadius: 6, background: "#FFFFFF", color: "#1A1A2E", outline: "none" }} />
              </div>

              <div
                onDragOver={(e) => { e.preventDefault(); setDragOver(true); }}
                onDragLeave={() => setDragOver(false)}
                onDrop={(e) => { e.preventDefault(); setDragOver(false); handleUpload(e.dataTransfer.files); }}
                onClick={() => fileInputRef.current?.click()}
                style={{
                  border: `2px dashed ${dragOver ? "#1B4332" : "#D1D5DB"}`,
                  borderRadius: 8,
                  padding: 32,
                  textAlign: "center",
                  cursor: "pointer",
                  background: dragOver ? "rgba(27,67,50,0.05)" : "#F8F9FA",
                  transition: "all 0.2s",
                  marginBottom: 16,
                }}
              >
                <Upload size={32} style={{ color: dragOver ? "#1B4332" : "#6B7280", marginBottom: 8 }} />
                <p style={{ fontSize: "0.9rem", color: dragOver ? "#1B4332" : "#6B7280", fontWeight: 500 }}>
                  {dragOver ? "Drop files here" : "Drag & drop files here, or click to browse"}
                </p>
              </div>
              <input ref={fileInputRef} type="file" multiple style={{ display: "none" }} onChange={(e) => handleUpload(e.target.files)} />

              <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
                <button onClick={() => setShowUpload(false)} style={{ padding: "8px 16px", borderRadius: 6, background: "#F8F9FA", border: "1px solid #E9ECEF", fontSize: "0.85rem", color: "#1A1A2E" }}>Cancel</button>
                <button disabled={uploading} onClick={() => fileInputRef.current?.click()} style={{ padding: "8px 16px", borderRadius: 6, background: "#1B4332", color: "#FFFFFF", fontSize: "0.85rem", fontWeight: 500, opacity: uploading ? 0.5 : 1 }}>
                  {uploading ? "Uploading..." : "Select Files"}
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </WorkspaceShell>
  );
}
