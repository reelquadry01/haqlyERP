// Author: Quadri Atharu
"use client";

import React, { useState, useRef } from "react";
import { Upload, X, FileText } from "lucide-react";

interface FileUploadProps {
  accept?: string;
  multiple?: boolean;
  onFiles: (files: FileList) => void;
  uploading?: boolean;
  progress?: number;
}

export function FileUpload({
  accept,
  multiple = false,
  onFiles,
  uploading = false,
  progress,
}: FileUploadProps) {
  const [dragOver, setDragOver] = useState(false);
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);

  function handleFiles(files: FileList) {
    const names = Array.from(files).map((f) => f.name);
    setSelectedFiles(names);
    onFiles(files);
  }

  function removeFile(index: number) {
    setSelectedFiles((prev) => prev.filter((_, i) => i !== index));
  }

  return (
    <div>
      <div
        onDragOver={(e) => {
          e.preventDefault();
          setDragOver(true);
        }}
        onDragLeave={() => setDragOver(false)}
        onDrop={(e) => {
          e.preventDefault();
          setDragOver(false);
          if (e.dataTransfer.files.length > 0) {
            handleFiles(e.dataTransfer.files);
          }
        }}
        onClick={() => inputRef.current?.click()}
        style={{
          border: `2px dashed ${dragOver ? "#1B4332" : "#D1D5DB"}`,
          borderRadius: 8,
          padding: "24px 16px",
          textAlign: "center",
          cursor: uploading ? "wait" : "pointer",
          background: dragOver ? "rgba(27,67,50,0.05)" : "#F8F9FA",
          transition: "all 0.2s",
        }}
      >
        <Upload
          size={28}
          style={{ color: dragOver ? "#1B4332" : "#6B7280", marginBottom: 8, margin: "0 auto", display: "block" }}
        />
        <p style={{ fontSize: "0.85rem", color: dragOver ? "#1B4332" : "#6B7280", fontWeight: 500 }}>
          {dragOver ? "Drop here" : "Drag & drop or click to browse"}
        </p>
        {accept && (
          <p style={{ fontSize: "0.75rem", color: "#9CA3AF", marginTop: 4 }}>
            Accepted: {accept}
          </p>
        )}
      </div>

      <input
        ref={inputRef}
        type="file"
        accept={accept}
        multiple={multiple}
        style={{ display: "none" }}
        onChange={(e) => {
          if (e.target.files && e.target.files.length > 0) {
            handleFiles(e.target.files);
          }
        }}
      />

      {selectedFiles.length > 0 && (
        <div style={{ marginTop: 12 }}>
          {selectedFiles.map((name, i) => (
            <div
              key={i}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 8,
                padding: "6px 10px",
                background: "#F8F9FA",
                borderRadius: 6,
                marginBottom: 4,
                fontSize: "0.85rem",
                color: "#1A1A2E",
              }}
            >
              <FileText size={14} style={{ color: "#6B7280", flexShrink: 0 }} />
              <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{name}</span>
              <button
                onClick={(e) => { e.stopPropagation(); removeFile(i); }}
                style={{ color: "#9CA3AF", padding: 0, background: "none", border: "none", cursor: "pointer", flexShrink: 0 }}
              >
                <X size={14} />
              </button>
            </div>
          ))}
        </div>
      )}

      {uploading && typeof progress === "number" && (
        <div style={{ marginTop: 12 }}>
          <div style={{ height: 4, background: "#E9ECEF", borderRadius: 2, overflow: "hidden" }}>
            <div
              style={{
                height: "100%",
                width: `${Math.min(progress, 100)}%`,
                background: "#1B4332",
                borderRadius: 2,
                transition: "width 0.3s",
              }}
            />
          </div>
          <p style={{ fontSize: "0.75rem", color: "#6B7280", marginTop: 4, textAlign: "center" }}>
            {progress}%
          </p>
        </div>
      )}
    </div>
  );
}
