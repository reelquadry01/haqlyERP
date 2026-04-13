"use client";

import { useState, useMemo, ReactNode } from "react";
import { ChevronUp, ChevronDown, ChevronLeft, ChevronRight, Search, Filter } from "lucide-react";

export interface Column<T> {
  key: string;
  label: string;
  sortable?: boolean;
  filterable?: boolean;
  render?: (value: any, row: T) => ReactNode;
  align?: "left" | "center" | "right";
  width?: string;
}

export interface ActionItem<T> {
  label: string;
  icon?: ReactNode;
  onClick: (row: T) => void;
  variant?: "default" | "danger";
}

interface DataTableProps<T> {
  columns: Column<T>[];
  data: T[];
  pageSize?: number;
  actions?: ActionItem<T>[];
  onRowClick?: (row: T) => void;
  emptyMessage?: string;
  keyField?: string;
}

export function DataTable<T extends Record<string, any>>({
  columns,
  data,
  pageSize = 10,
  actions,
  onRowClick,
  emptyMessage = "No data available",
  keyField = "id",
}: DataTableProps<T>) {
  const [sortKey, setSortKey] = useState<string | null>(null);
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");
  const [page, setPage] = useState(0);
  const [filterText, setFilterText] = useState("");
  const [activeFilterCol, setActiveFilterCol] = useState<string | null>(null);
  const [openActionRow, setOpenActionRow] = useState<string | null>(null);

  const filteredData = useMemo(() => {
    if (!filterText) return data;
    return data.filter((row) =>
      columns.some((col) => {
        const val = row[col.key];
        return val != null && String(val).toLowerCase().includes(filterText.toLowerCase());
      })
    );
  }, [data, filterText, columns]);

  const sortedData = useMemo(() => {
    if (!sortKey) return filteredData;
    return [...filteredData].sort((a, b) => {
      const aVal = a[sortKey];
      const bVal = b[sortKey];
      if (aVal == null) return 1;
      if (bVal == null) return -1;
      const cmp = typeof aVal === "number" ? aVal - bVal : String(aVal).localeCompare(String(bVal));
      return sortDir === "asc" ? cmp : -cmp;
    });
  }, [filteredData, sortKey, sortDir]);

  const totalPages = Math.ceil(sortedData.length / pageSize);
  const pageData = sortedData.slice(page * pageSize, (page + 1) * pageSize);

  function handleSort(key: string) {
    if (sortKey === key) {
      setSortDir(sortDir === "asc" ? "desc" : "asc");
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
  }

  const thStyle: React.CSSProperties = {
    padding: "10px 12px",
    textAlign: "left",
    fontWeight: 600,
    fontSize: "0.8rem",
    color: "#94a3b8",
    textTransform: "uppercase",
    letterSpacing: "0.05em",
    borderBottom: "1px solid #2a3754",
    whiteSpace: "nowrap",
  };

  const tdStyle: React.CSSProperties = {
    padding: "10px 12px",
    fontSize: "0.875rem",
    borderBottom: "1px solid #1e2a3e",
  };

  return (
    <div>
      {/* Search / Filter Bar */}
      <div style={{ display: "flex", gap: 8, marginBottom: 12, alignItems: "center" }}>
        <div style={{ position: "relative", flex: 1, maxWidth: 320 }}>
          <Search
            size={14}
            style={{ position: "absolute", left: 10, top: "50%", transform: "translateY(-50%)", color: "#64748b" }}
          />
          <input
            type="text"
            value={filterText}
            onChange={(e) => {
              setFilterText(e.target.value);
              setPage(0);
            }}
            placeholder="Search..."
            style={{
              width: "100%",
              background: "#111827",
              borderColor: "#2a3754",
              paddingLeft: 32,
              fontSize: "0.85rem",
            }}
          />
        </div>
        <span style={{ fontSize: "0.8rem", color: "#64748b" }}>
          {filteredData.length} record{filteredData.length !== 1 ? "s" : ""}
        </span>
      </div>

      {/* Table */}
      <div style={{ overflowX: "auto" }}>
        <table style={{ width: "100%", borderCollapse: "collapse" }}>
          <thead>
            <tr>
              {columns.map((col) => (
                <th
                  key={col.key}
                  style={{
                    ...thStyle,
                    textAlign: col.align || "left",
                    cursor: col.sortable ? "pointer" : "default",
                    width: col.width,
                  }}
                  onClick={() => col.sortable && handleSort(col.key)}
                >
                  <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
                    {col.label}
                    {col.sortable && sortKey === col.key && (
                      sortDir === "asc" ? <ChevronUp size={12} /> : <ChevronDown size={12} />
                    )}
                    {col.filterable && (
                      <button
                        className="btn btn-ghost btn-sm"
                        style={{ padding: 0, marginLeft: 4 }}
                        onClick={(e) => {
                          e.stopPropagation();
                          setActiveFilterCol(activeFilterCol === col.key ? null : col.key);
                        }}
                      >
                        <Filter size={12} />
                      </button>
                    )}
                  </div>
                </th>
              ))}
              {actions && actions.length > 0 && (
                <th style={{ ...thStyle, textAlign: "right", width: 60 }}>Actions</th>
              )}
            </tr>
          </thead>
          <tbody>
            {pageData.length === 0 ? (
              <tr>
                <td
                  colSpan={columns.length + (actions ? 1 : 0)}
                  style={{ textAlign: "center", padding: 32, color: "#64748b" }}
                >
                  {emptyMessage}
                </td>
              </tr>
            ) : (
              pageData.map((row) => (
                <tr
                  key={row[keyField]}
                  style={{ cursor: onRowClick ? "pointer" : "default" }}
                  onClick={() => onRowClick?.(row)}
                  onMouseEnter={(e) => (e.currentTarget.style.background = "#1a2234")}
                  onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
                >
                  {columns.map((col) => (
                    <td key={col.key} style={{ ...tdStyle, textAlign: col.align || "left" }}>
                      {col.render ? col.render(row[col.key], row) : row[col.key]}
                    </td>
                  ))}
                  {actions && actions.length > 0 && (
                    <td style={{ ...tdStyle, textAlign: "right" }}>
                      <div style={{ position: "relative", display: "inline-block" }}>
                        <button
                          className="btn btn-ghost btn-sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            setOpenActionRow(openActionRow === row[keyField] ? null : row[keyField]);
                          }}
                        >
                          •••
                        </button>
                        {openActionRow === row[keyField] && (
                          <div
                            style={{
                              position: "absolute",
                              right: 0,
                              top: "100%",
                              marginTop: 4,
                              background: "#1a2234",
                              border: "1px solid #2a3754",
                              borderRadius: 8,
                              minWidth: 160,
                              boxShadow: "0 8px 24px rgba(0,0,0,0.5)",
                              zIndex: 100,
                            }}
                          >
                            {actions.map((action, i) => (
                              <button
                                key={i}
                                className="btn btn-ghost"
                                style={{
                                  width: "100%",
                                  justifyContent: "flex-start",
                                  padding: "8px 12px",
                                  fontSize: "0.85rem",
                                  color: action.variant === "danger" ? "#ef4444" : "#f0f4f8",
                                }}
                                onClick={(e) => {
                                  e.stopPropagation();
                                  action.onClick(row);
                                  setOpenActionRow(null);
                                }}
                              >
                                {action.icon}
                                {action.label}
                              </button>
                            ))}
                          </div>
                        )}
                      </div>
                    </td>
                  )}
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            marginTop: 12,
            paddingTop: 12,
            borderTop: "1px solid #1e2a3e",
          }}
        >
          <span style={{ fontSize: "0.8rem", color: "#64748b" }}>
            Page {page + 1} of {totalPages}
          </span>
          <div style={{ display: "flex", gap: 4 }}>
            <button
              className="btn btn-ghost btn-sm"
              disabled={page === 0}
              onClick={() => setPage(page - 1)}
            >
              <ChevronLeft size={14} />
            </button>
            {Array.from({ length: Math.min(totalPages, 5) }, (_, i) => {
              const pageNum = Math.max(0, Math.min(page - 2, totalPages - 5)) + i;
              if (pageNum >= totalPages) return null;
              return (
                <button
                  key={pageNum}
                  className="btn btn-sm"
                  style={{
                    background: pageNum === page ? "#3b82f6" : "transparent",
                    color: pageNum === page ? "#fff" : "#94a3b8",
                    minWidth: 32,
                  }}
                  onClick={() => setPage(pageNum)}
                >
                  {pageNum + 1}
                </button>
              );
            })}
            <button
              className="btn btn-ghost btn-sm"
              disabled={page >= totalPages - 1}
              onClick={() => setPage(page + 1)}
            >
              <ChevronRight size={14} />
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
