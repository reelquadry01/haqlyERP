// Author: Quadri Atharu

// Dynamic imports with loading skeletons for heavy ERP components
// Uses next/dynamic for code splitting

import dynamic from "next/dynamic";
import React, { useState, useEffect, useRef, useCallback, ComponentType } from "react";

const HAQLY_SKELETON_COLOR = "#1B4332";

function HaqlyLoadingSkeleton() {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        minHeight: "200px",
        width: "100%",
        padding: "24px",
      }}
    >
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: "12px",
          width: "100%",
          maxWidth: "480px",
        }}
      >
        <div
          style={{
            width: "60%",
            height: "18px",
            borderRadius: "8px",
            background: HAQLY_SKELETON_COLOR,
            opacity: 0.15,
            animation: "haqlyPulse 1.5s ease-in-out infinite",
          }}
        />
        <div
          style={{
            width: "100%",
            height: "140px",
            borderRadius: "8px",
            background: HAQLY_SKELETON_COLOR,
            opacity: 0.12,
            animation: "haqlyPulse 1.5s ease-in-out infinite 0.2s",
          }}
        />
        <div
          style={{
            width: "40%",
            height: "14px",
            borderRadius: "8px",
            background: HAQLY_SKELETON_COLOR,
            opacity: 0.1,
            animation: "haqlyPulse 1.5s ease-in-out infinite 0.4s",
          }}
        />
      </div>
      <style>{`
        @keyframes haqlyPulse {
          0%, 100% { opacity: 0.12; }
          50% { opacity: 0.28; }
        }
      `}</style>
    </div>
  );
}

function HaqlyWideSkeleton() {
  return (
    <div
      style={{
        display: "flex",
        gap: "16px",
        width: "100%",
        minHeight: "280px",
        padding: "24px",
      }}
    >
      {Array.from({ length: 3 }).map((_, i) => (
        <div
          key={i}
          style={{
            flex: 1,
            borderRadius: "12px",
            background: HAQLY_SKELETON_COLOR,
            opacity: 0.1,
            animation: `haqlyPulse 1.5s ease-in-out infinite ${i * 0.15}s`,
          }}
        />
      ))}
      <style>{`
        @keyframes haqlyPulse {
          0%, 100% { opacity: 0.10; }
          50% { opacity: 0.24; }
        }
      `}</style>
    </div>
  );
}

function HaqlyFormSkeleton() {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "16px",
        width: "100%",
        maxWidth: "640px",
        padding: "24px",
      }}
    >
      {Array.from({ length: 6 }).map((_, i) => (
        <div key={i} style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
          <div
            style={{
              width: "30%",
              height: "12px",
              borderRadius: "6px",
              background: HAQLY_SKELETON_COLOR,
              opacity: 0.14,
              animation: `haqlyPulse 1.5s ease-in-out infinite ${i * 0.1}s`,
            }}
          />
          <div
            style={{
              width: "100%",
              height: "36px",
              borderRadius: "8px",
              background: HAQLY_SKELETON_COLOR,
              opacity: 0.08,
              animation: `haqlyPulse 1.5s ease-in-out infinite ${i * 0.1 + 0.05}s`,
            }}
          />
        </div>
      ))}
      <style>{`
        @keyframes haqlyPulse {
          0%, 100% { opacity: 0.08; }
          50% { opacity: 0.20; }
        }
      `}</style>
    </div>
  );
}

export const LazyDashboardCharts = dynamic(
  () => import("@/components/erp/dashboard-charts"),
  {
    ssr: false,
    loading: () => <HaqlyWideSkeleton />,
  }
);

export const LazyJournalEntryForm = dynamic(
  () => import("@/components/erp/journal-entry-form"),
  {
    ssr: false,
    loading: () => <HaqlyFormSkeleton />,
  }
);

export const LazyTaxComputationEngine = dynamic(
  () => import("@/components/erp/tax-computation-engine"),
  {
    ssr: false,
    loading: () => <HaqlyFormSkeleton />,
  }
);

export const LazyFinancialStatements = dynamic(
  () => import("@/components/erp/financial-statements"),
  {
    ssr: false,
    loading: () => <HaqlyWideSkeleton />,
  }
);

export const LazyPayrollProcessor = dynamic(
  () => import("@/components/erp/payroll-processor"),
  {
    ssr: false,
    loading: () => <HaqlyFormSkeleton />,
  }
);

export const LazyBIWidgets = dynamic(
  () => import("@/components/erp/bi-widgets"),
  {
    ssr: false,
    loading: () => <HaqlyWideSkeleton />,
  }
);

export const LazyCRMBoard = dynamic(
  () => import("@/components/erp/crm-board"),
  {
    ssr: false,
    loading: () => <HaqlyLoadingSkeleton />,
  }
);

export const LazyOCRUploader = dynamic(
  () => import("@/components/erp/ocr-uploader"),
  {
    ssr: false,
    loading: () => <HaqlyLoadingSkeleton />,
  }
);

export const LazyEInvoiceManager = dynamic(
  () => import("@/components/erp/einvoice-manager"),
  {
    ssr: false,
    loading: () => <HaqlyFormSkeleton />,
  }
);

export const LazyAdminPanel = dynamic(
  () => import("@/components/erp/admin-panel"),
  {
    ssr: false,
    loading: () => <HaqlyWideSkeleton />,
  }
);

interface UseLazyImportState<T> {
  module: T | null;
  loading: boolean;
  error: Error | null;
}

export function useLazyImport<T>(importFn: () => Promise<T>): UseLazyImportState<T> {
  const [state, setState] = useState<UseLazyImportState<T>>({
    module: null,
    loading: false,
    error: null,
  });

  const importFnRef = useRef(importFn);
  importFnRef.current = importFn;

  const execute = useCallback(async () => {
    setState((prev) => ({ ...prev, loading: true, error: null }));
    try {
      const mod = await importFnRef.current();
      setState({ module: mod, loading: false, error: null });
    } catch (err) {
      const error =
        err instanceof Error
          ? err
          : new Error(typeof err === "string" ? err : "Failed to import module");
      setState((prev) => ({ ...prev, loading: false, error }));
    }
  }, []);

  useEffect(() => {
    execute();
  }, [execute]);

  return state;
}

interface LazySectionProps {
  children: React.ReactNode;
  rootMargin?: string;
  threshold?: number;
  placeholder?: React.ReactNode;
  className?: string;
  once?: boolean;
}

export function LazySection({
  children,
  rootMargin = "200px 0px",
  threshold = 0.01,
  placeholder,
  className,
  once = true,
}: LazySectionProps) {
  const [isVisible, setIsVisible] = useState(false);
  const ref = useRef<HTMLDivElement | null>(null);
  const observerRef = useRef<IntersectionObserver | null>(null);

  useEffect(() => {
    const element = ref.current;
    if (!element) return;

    if (typeof IntersectionObserver === "undefined") {
      setIsVisible(true);
      return;
    }

    observerRef.current = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setIsVisible(true);
          if (once && observerRef.current) {
            observerRef.current.unobserve(element);
            observerRef.current.disconnect();
          }
        } else if (!once) {
          setIsVisible(false);
        }
      },
      { rootMargin, threshold }
    );

    observerRef.current.observe(element);

    return () => {
      if (observerRef.current) {
        observerRef.current.disconnect();
      }
    };
  }, [rootMargin, threshold, once]);

  const defaultPlaceholder = (
    <div
      style={{
        minHeight: "120px",
        width: "100%",
        borderRadius: "8px",
        background: HAQLY_SKELETON_COLOR,
        opacity: 0.06,
        animation: "haqlyPulse 1.5s ease-in-out infinite",
      }}
    >
      <style>{`
        @keyframes haqlyPulse {
          0%, 100% { opacity: 0.06; }
          50% { opacity: 0.14; }
        }
      `}</style>
    </div>
  );

  return (
    <div ref={ref} className={className}>
      {isVisible ? children : (placeholder ?? defaultPlaceholder)}
    </div>
  );
}

type DynamicOptions = Parameters<typeof dynamic>[1];

export function createLazyComponent<P = object>(
  importFn: () => Promise<{ default: ComponentType<P> }>,
  options: Omit<NonNullable<DynamicOptions>, "loading"> & {
    loading?: () => React.ReactNode;
    ssr?: boolean;
  } = {}
) {
  const { loading, ssr = false, ...rest } = options;
  const loadingComponent = loading
    ? () => <>{loading()}</>
    : () => <HaqlyLoadingSkeleton />;

  return dynamic(importFn, {
    ssr,
    loading: loadingComponent,
    ...rest,
  });
}
