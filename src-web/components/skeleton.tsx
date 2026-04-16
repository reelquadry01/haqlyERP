// Author: Quadri Atharu
'use client'

const pulseStyle = {
  background: 'linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%)',
  backgroundSize: '200% 100%',
  animation: 'pulse 1.5s ease-in-out infinite',
}

export function CardSkeleton() {
  return (
    <div style={{ ...pulseStyle, height: 120, borderRadius: 8, marginBottom: 16 }} />
  )
}

export function TableSkeleton({ rows = 5 }: { rows?: number }) {
  return (
    <div>
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} style={{ ...pulseStyle, height: 40, borderRadius: 4, marginBottom: 8, opacity: 1 - i * 0.15 }} />
      ))}
    </div>
  )
}

export function FormSkeleton() {
  return (
    <div>
      {Array.from({ length: 4 }).map((_, i) => (
        <div key={i} style={{ marginBottom: 16 }}>
          <div style={{ ...pulseStyle, height: 14, width: 120, borderRadius: 4, marginBottom: 8 }} />
          <div style={{ ...pulseStyle, height: 40, borderRadius: 6 }} />
        </div>
      ))}
    </div>
  )
}

export function KPISkeleton() {
  return (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: 16 }}>
      {Array.from({ length: 4 }).map((_, i) => (
        <div key={i} style={{ ...pulseStyle, height: 100, borderRadius: 8 }} />
      ))}
    </div>
  )
}

export function Skeleton() {
  return <div style={{ ...pulseStyle, height: 200, borderRadius: 8, width: '100%' }} />
}

export function DataTableSkeleton({ rows = 5 }: { rows?: number }) {
  return (
    <div>
      <div style={{ ...pulseStyle, height: 40, borderRadius: 6, marginBottom: 12, opacity: 0.7 }} />
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} style={{ ...pulseStyle, height: 36, borderRadius: 4, marginBottom: 6, opacity: 1 - i * 0.12 }} />
      ))}
    </div>
  )
}

export function CardGridSkeleton({ count = 4 }: { count?: number }) {
  return (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: 16 }}>
      {Array.from({ length: count }).map((_, i) => (
        <div key={i} style={{ ...pulseStyle, height: 140, borderRadius: 8 }} />
      ))}
    </div>
  )
}
