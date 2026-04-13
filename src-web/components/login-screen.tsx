"use client";

import { useState, FormEvent } from "react";
import { apiPost } from "@/lib/api";
import { saveToken, saveCompanyContext } from "@/lib/session";
import { BrandLockup } from "@/components/ui/brand-lockup";
import { Eye, EyeOff, Shield, AlertCircle, Loader2 } from "lucide-react";

interface LoginResponse {
  token: string;
  user: { id: string; email: string; name: string };
  companies: { id: string; name: string }[];
}

export function LoginScreen() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [mfaCode, setMfaCode] = useState("");
  const [mfaToken, setMfaToken] = useState("");
  const [step, setStep] = useState<"credentials" | "mfa">("credentials");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleCredentialsSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      const res = await apiPost("/auth/login", { email, password });
      const data = await res.json();

      if (!res.ok) {
        setError(data.message || "Invalid credentials");
        return;
      }

      if (data.mfaRequired) {
        setMfaToken(data.mfaToken);
        setStep("mfa");
        return;
      }

      const loginData = data as LoginResponse;
      saveToken(loginData.token);
      if (loginData.companies.length > 0) {
        saveCompanyContext(loginData.companies[0].id);
      }
      window.location.replace("/dashboard");
    } catch (err: any) {
      setError(err.message || "Network error. Please check your connection.");
    } finally {
      setLoading(false);
    }
  }

  async function handleMfaSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      const res = await apiPost("/auth/mfa/verify", { mfaToken, code: mfaCode });
      const data = await res.json();

      if (!res.ok) {
        setError(data.message || "Invalid MFA code");
        return;
      }

      const loginData = data as LoginResponse;
      saveToken(loginData.token);
      if (loginData.companies.length > 0) {
        saveCompanyContext(loginData.companies[0].id);
      }
      window.location.replace("/dashboard");
    } catch (err: any) {
      setError(err.message || "Network error. Please check your connection.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="flex-center full-viewport" style={{ background: "#0a0e17" }}>
      <div
        style={{
          width: 400,
          maxWidth: "90vw",
          background: "#1a2234",
          border: "1px solid #2a3754",
          borderRadius: 16,
          padding: 32,
        }}
        className="fade-in"
      >
        <div style={{ textAlign: "center", marginBottom: 32 }}>
          <BrandLockup size="large" />
          <p style={{ color: "#64748b", marginTop: 8, fontSize: "0.85rem" }}>
            Enterprise Resource Planning for Nigerian Businesses
          </p>
        </div>

        {error && (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 8,
              padding: "10px 14px",
              marginBottom: 16,
              background: "rgba(239,68,68,0.12)",
              border: "1px solid rgba(239,68,68,0.25)",
              borderRadius: 8,
              color: "#ef4444",
              fontSize: "0.85rem",
            }}
          >
            <AlertCircle size={16} />
            {error}
          </div>
        )}

        {step === "credentials" ? (
          <form onSubmit={handleCredentialsSubmit}>
            <div style={{ marginBottom: 16 }}>
              <label style={{ display: "block", marginBottom: 6, fontSize: "0.85rem", color: "#94a3b8" }}>
                Email Address
              </label>
              <input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="admin@haqly.com"
                required
                style={{ width: "100%", background: "#111827", borderColor: "#2a3754" }}
              />
            </div>

            <div style={{ marginBottom: 24 }}>
              <label style={{ display: "block", marginBottom: 6, fontSize: "0.85rem", color: "#94a3b8" }}>
                Password
              </label>
              <div style={{ position: "relative" }}>
                <input
                  type={showPassword ? "text" : "password"}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Enter your password"
                  required
                  style={{ width: "100%", background: "#111827", borderColor: "#2a3754", paddingRight: 40 }}
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  style={{ position: "absolute", right: 10, top: "50%", transform: "translateY(-50%)", color: "#64748b" }}
                >
                  {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
                </button>
              </div>
            </div>

            <button
              type="submit"
              disabled={loading}
              className="btn btn-primary"
              style={{ width: "100%", justifyContent: "center", padding: "10px 16px" }}
            >
              {loading ? <Loader2 size={16} /> : "Sign In"}
            </button>
          </form>
        ) : (
          <form onSubmit={handleMfaSubmit}>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 8,
                marginBottom: 16,
                color: "#06b6d4",
                fontSize: "0.85rem",
              }}
            >
              <Shield size={16} />
              Multi-factor authentication required
            </div>

            <div style={{ marginBottom: 24 }}>
              <label style={{ display: "block", marginBottom: 6, fontSize: "0.85rem", color: "#94a3b8" }}>
                Verification Code
              </label>
              <input
                type="text"
                value={mfaCode}
                onChange={(e) => setMfaCode(e.target.value)}
                placeholder="000000"
                maxLength={6}
                required
                style={{
                  width: "100%",
                  background: "#111827",
                  borderColor: "#2a3754",
                  textAlign: "center",
                  fontSize: "1.25rem",
                  letterSpacing: "0.3em",
                  fontFamily: "var(--font-mono)",
                }}
              />
            </div>

            <button
              type="submit"
              disabled={loading}
              className="btn btn-primary"
              style={{ width: "100%", justifyContent: "center", padding: "10px 16px" }}
            >
              {loading ? <Loader2 size={16} /> : "Verify"}
            </button>

            <button
              type="button"
              className="btn btn-ghost"
              style={{ width: "100%", justifyContent: "center", marginTop: 8 }}
              onClick={() => {
                setStep("credentials");
                setMfaCode("");
                setMfaToken("");
                setError(null);
              }}
            >
              Back to sign in
            </button>
          </form>
        )}

        <p style={{ textAlign: "center", marginTop: 24, fontSize: "0.75rem", color: "#64748b" }}>
          HAQLY ERP v0.1.0 — Quadri Atharu
        </p>
      </div>
    </div>
  );
}
