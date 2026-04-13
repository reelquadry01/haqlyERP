"use client";

import { Check, FileText, Send, BookOpen } from "lucide-react";

export type ApprovalStep = "draft" | "submitted" | "approved" | "posted";

interface ApprovalStepperProps {
  currentStep: ApprovalStep;
  size?: "sm" | "md" | "lg";
}

const STEPS: { key: ApprovalStep; label: string; icon: typeof Check }[] = [
  { key: "draft", label: "Draft", icon: FileText },
  { key: "submitted", label: "Submitted", icon: Send },
  { key: "approved", label: "Approved", icon: Check },
  { key: "posted", label: "Posted", icon: BookOpen },
];

const STEP_INDEX: Record<ApprovalStep, number> = {
  draft: 0,
  submitted: 1,
  approved: 2,
  posted: 3,
};

export function ApprovalStepper({ currentStep, size = "md" }: ApprovalStepperProps) {
  const currentIndex = STEP_INDEX[currentStep];
  const iconSize = size === "sm" ? 12 : size === "lg" ? 18 : 14;
  const labelSize = size === "sm" ? "0.65rem" : size === "lg" ? "0.85rem" : "0.75rem";
  const circleSize = size === "sm" ? 22 : size === "lg" ? 34 : 28;

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 0 }}>
      {STEPS.map((step, i) => {
        const isCompleted = i < currentIndex;
        const isCurrent = i === currentIndex;
        const isFuture = i > currentIndex;

        const Icon = step.icon;

        return (
          <div key={step.key} style={{ display: "flex", alignItems: "center" }}>
            <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 4 }}>
              <div
                style={{
                  width: circleSize,
                  height: circleSize,
                  borderRadius: "50%",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  background: isCompleted
                    ? "#10b981"
                    : isCurrent
                    ? "#3b82f6"
                    : "#1e2a3e",
                  border: isCurrent ? "2px solid #3b82f6" : "2px solid transparent",
                  color: isCompleted || isCurrent ? "#fff" : "#64748b",
                  transition: "all 0.2s",
                }}
              >
                <Icon size={iconSize} />
              </div>
              <span
                style={{
                  fontSize: labelSize,
                  color: isCompleted
                    ? "#10b981"
                    : isCurrent
                    ? "#3b82f6"
                    : "#64748b",
                  fontWeight: isCurrent ? 600 : 400,
                  whiteSpace: "nowrap",
                }}
              >
                {step.label}
              </span>
            </div>
            {i < STEPS.length - 1 && (
              <div
                style={{
                  width: size === "sm" ? 24 : size === "lg" ? 48 : 36,
                  height: 2,
                  background: isCompleted ? "#10b981" : "#1e2a3e",
                  margin: `0 ${size === "sm" ? 4 : 8}px`,
                  marginBottom: 18,
                  borderRadius: 1,
                  transition: "background 0.2s",
                }}
              />
            )}
          </div>
        );
      })}
    </div>
  );
}
