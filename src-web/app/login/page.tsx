// Author: Quadri Atharu

"use client";

import { useState, FormEvent, useRef, useEffect } from "react";
import { apiPost } from "@/lib/api";
import { saveToken, saveCompanyContext } from "@/lib/session";
import { BrandLockup } from "@/components/ui/brand-lockup";
import {
  Eye,
  EyeOff,
  Shield,
  AlertCircle,
  Loader2,
  KeyRound,
  ArrowLeft,
  Mail,
  Lock,
} from "lucide-react";

const TOKENS = {
  primary: "#1B4332",
  primaryHover: "#2D6A4F",
  primaryLight: "rgba(27,67,50,0.08)",
  accent: "#D4AF37",
  accentLight: "rgba(212,175,55,0.12)",
  bg: "#F8F9FA",
  surface: "#FFFFFF",
  border: "#DEE2E6",
  borderSubtle: "#E9ECEF",
  text: "#1A1A2E",
  textSecondary: "#495057",
  textTertiary: "#868E96",
  error: "#DC2626",
  errorLight: "rgba(220,38,38,0.08)",
  success: "#16A34A",
  successLight: "rgba(22,163,74,0.08)",
  radiusMd: 8,
  radiusSm: 6,
  shadowMd: "0 4px 16px rgba(0,0,0,0.10)",
  fontUi: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  fontHeading: '"DM Serif Display", Georgia, serif',
};

interface LoginResponse {
  token: string;
  user: { id: string; email: string; name: string };
  companies: { id: string; name: string }[];
}

interface MfaRequiredResponse {
  mfaRequired: true;
  mfaToken: string;
}

type LoginStep = "credentials" | "mfa" | "recovery" | "forgot_password";

export default function LoginPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [rememberMe, setRememberMe] = useState(false);
  const [mfaCode, setMfaCode] = useState("");
  const [mfaToken, setMfaToken] = useState("");
  const [recoveryCode, setRecoveryCode] = useState("");
  const [step, setStep] = useState<LoginStep>("credentials");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [mfaAttempts, setMfaAttempts] = useState(0);
  const [forgotEmail, setForgotEmail] = useState("");
  const [forgotSent, setForgotSent] = useState(false);
  const mfaInputsRef = useRef<(HTMLInputElement | null)[]>([]);

  useEffect(() => {
    const remembered = localStorage.getItem("haqly_remember_email");
    if (remembered) {
      setEmail(remembered);
      setRememberMe(true);
    }
  }, []);

  useEffect(() => {
    if (step === "mfa" && mfaInputsRef.current[0]) {
      mfaInputsRef.current[0].focus();
    }
  }, [step]);

  function handleMfaDigitChange(index: number, value: string) {
    if (!/^\d*$/.test(value)) return;
    const digits = mfaCode.split("");
    while (digits.length < 6) digits.push("");
    digits[index] = value.slice(-1);
    const newCode = digits.join("");
    setMfaCode(newCode);
    if (value && index < 5) {
      mfaInputsRef.current[index + 1]?.focus();
    }
  }

  function handleMfaKeyDown(index: number, e: React.KeyboardEvent) {
    if (e.key === "Backspace" && !mfaCode[index] && index > 0) {
      mfaInputsRef.current[index - 1]?.focus();
    }
  }

  async function handleCredentialsSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);
    try {
      const res = await apiPost("/auth/login", { email, password, remember_me: rememberMe });
      const data = await res.json();
      if (!res.ok) {
        if (data.code === "ACCOUNT_LOCKED") {
          setError("Account locked. Contact your administrator.");
        } else if (data.code === "MFA_REQUIRED") {
          setMfaToken(data.mfaToken);
          setStep("mfa");
          return;
        } else {
          setError(data.message || "Invalid email or password");
        }
        return;
      }
      if (data.mfaRequired) {
        setMfaToken(data.mfaToken);
        setStep("mfa");
        return;
      }
      const loginData = data as LoginResponse;
      saveToken(loginData.token);
      if (rememberMe) {
        localStorage.setItem("haqly_remember_email", email);
      }
      if (loginData.companies.length > 0) {
        saveCompanyContext(loginData.companies[0].id);
      }
      window.location.replace("/dashboard");
    } catch {
      setError("Network error. Please check your connection.");
    } finally {
      setLoading(false);
    }
  }

  async function handleMfaSubmit(e: FormEvent) {
    e.preventDefault();
    if (mfaCode.length < 6) {
      setError("Enter all 6 digits");
      return;
    }
    setError(null);
    setLoading(true);
    try {
      const res = await apiPost("/auth/mfa/verify", { mfaToken, code: mfaCode });
      const data = await res.json();
      if (!res.ok) {
        const newAttempts = mfaAttempts + 1;
        setMfaAttempts(newAttempts);
        if (newAttempts >= 3) {
          setError("Too many failed attempts. Use a recovery code.");
          setStep("recovery");
          return;
        }
        setError(data.message || `Invalid MFA code. ${3 - newAttempts} attempts remaining.`);
        setMfaCode("");
        return;
      }
      const loginData = data as LoginResponse;
      saveToken(loginData.token);
      if (loginData.companies.length > 0) {
        saveCompanyContext(loginData.companies[0].id);
      }
      window.location.replace("/dashboard");
    } catch {
      setError("Network error. Please check your connection.");
    } finally {
      setLoading(false);
    }
  }

  async function handleRecoverySubmit(e: FormEvent) {
    e.preventDefault();
    if (!recoveryCode.trim()) {
      setError("Enter your recovery code");
      return;
    }
    setError(null);
    setLoading(true);
    try {
      const res = await apiPost("/auth/mfa/recover", { mfaToken, recoveryCode: recoveryCode.trim() });
      const data = await res.json();
      if (!res.ok) {
        setError(data.message || "Invalid recovery code");
        return;
      }
      const loginData = data as LoginResponse;
      saveToken(loginData.token);
      if (loginData.companies.length > 0) {
        saveCompanyContext(loginData.companies[0].id);
      }
      window.location.replace("/dashboard");
    } catch {
      setError("Network error. Please check your connection.");
    } finally {
      setLoading(false);
    }
  }

  async function handleForgotPassword(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);
    try {
      const res = await apiPost("/auth/forgot-password", { email: forgotEmail });
      if (res.ok) {
        setForgotSent(true);
      } else {
        const data = await res.json();
        setError(data.message || "Failed to send reset email");
      }
    } catch {
      setError("Network error. Please check your connection.");
    } finally {
      setLoading(false);
    }
  }

  function resetToCredentials() {
    setStep("credentials");
    setMfaCode("");
    setMfaToken("");
    setRecoveryCode("");
    setError(null);
    setMfaAttempts(0);
  }

  const inputStyle: React.CSSProperties = {
    width: "100%",
    background: TOKENS.bg,
    border: `1px solid ${TOKENS.border}`,
    borderRadius: TOKENS.radiusSm,
    padding: "10px 12px",
    fontSize: "0.9rem",
    color: TOKENS.text,
    fontFamily: TOKENS.fontUi,
    transition: "border-color 150ms ease",
  };

  const labelStyle: React.CSSProperties = {
    display: "block",
    marginBottom: 6,
    fontSize: "0.8rem",
    fontWeight: 600,
    color: TOKENS.textSecondary,
  };

  const buttonStyle: React.CSSProperties = {
    width: "100%",
    padding: "12px 16px",
    borderRadius: TOKENS.radiusSm,
    fontSize: "0.9rem",
    fontWeight: 600,
    background: TOKENS.primary,
    color: "#FFFFFF",
    border: "none",
    cursor: "pointer",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    gap: 8,
    transition: "background 150ms ease",
  };

  return (
    <div style={{ minHeight: "100vh", display: "flex", alignItems: "center", justifyContent: "center", background: `linear-gradient(135deg, ${TOKENS.primary} 0%, #2D6A4F 50%, ${TOKENS.primary} 100%)` }}>
      <div style={{ width: 420, maxWidth: "90vw", background: TOKENS.surface, borderRadius: TOKENS.radiusMd, padding: 36, boxShadow: TOKENS.shadowMd }} className="fade-in">
        <div style={{ textAlign: "center", marginBottom: 32 }}>
          <div style={{ width: 56, height: 56, borderRadius: 12, background: `linear-gradient(135deg, ${TOKENS.primary}, ${TOKENS.accent})`, display: "flex", alignItems: "center", justifyContent: "center", margin: "0 auto 16px", fontWeight: 800, fontSize: "1.4rem", color: "#FFFFFF", letterSpacing: "-0.02em" }}>
            H
          </div>
          <h1 style={{ fontSize: "1.5rem", fontWeight: 700, color: TOKENS.text, fontFamily: TOKENS.fontHeading, letterSpacing: "-0.02em" }}>
            HAQLY ERP
          </h1>
          <p style={{ color: TOKENS.textTertiary, marginTop: 4, fontSize: "0.85rem" }}>
            Enterprise Resource Planning for Nigerian Businesses
          </p>
        </div>

        {error && (
          <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "10px 14px", marginBottom: 16, background: TOKENS.errorLight, border: `1px solid ${TOKENS.error}`, borderRadius: TOKENS.radiusSm, color: TOKENS.error, fontSize: "0.85rem" }}>
            <AlertCircle size={16} />
            {error}
          </div>
        )}

        {step === "credentials" && (
          <form onSubmit={handleCredentialsSubmit}>
            <div style={{ marginBottom: 16 }}>
              <label style={labelStyle}>Email Address</label>
              <div style={{ position: "relative" }}>
                <Mail size={16} style={{ position: "absolute", left: 12, top: "50%", transform: "translateY(-50%)", color: TOKENS.textTertiary }} />
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="admin@haqly.com"
                  required
                  style={{ ...inputStyle, paddingLeft: 36 }}
                />
              </div>
            </div>

            <div style={{ marginBottom: 16 }}>
              <label style={labelStyle}>Password</label>
              <div style={{ position: "relative" }}>
                <Lock size={16} style={{ position: "absolute", left: 12, top: "50%", transform: "translateY(-50%)", color: TOKENS.textTertiary }} />
                <input
                  type={showPassword ? "text" : "password"}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Enter your password"
                  required
                  style={{ ...inputStyle, paddingLeft: 36, paddingRight: 40 }}
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  style={{ position: "absolute", right: 10, top: "50%", transform: "translateY(-50%)", color: TOKENS.textTertiary, cursor: "pointer" }}
                >
                  {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
                </button>
              </div>
            </div>

            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 24 }}>
              <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: "0.85rem", color: TOKENS.textSecondary, cursor: "pointer" }}>
                <input type="checkbox" checked={rememberMe} onChange={(e) => setRememberMe(e.target.checked)} style={{ accentColor: TOKENS.primary }} />
                Remember me
              </label>
              <button
                type="button"
                onClick={() => { setStep("forgot_password"); setForgotEmail(email); setError(null); setForgotSent(false); }}
                style={{ fontSize: "0.85rem", color: TOKENS.primary, fontWeight: 500, cursor: "pointer", background: "none", border: "none" }}
              >
                Forgot Password?
              </button>
            </div>

            <button type="submit" disabled={loading} style={buttonStyle}>
              {loading ? <Loader2 size={16} style={{ animation: "spin 0.8s linear infinite" }} /> : "Sign In"}
            </button>
          </form>
        )}

        {step === "mfa" && (
          <form onSubmit={handleMfaSubmit}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 20, color: TOKENS.primary, fontSize: "0.85rem" }}>
              <Shield size={18} />
              <span style={{ fontWeight: 600 }}>Multi-factor authentication required</span>
            </div>
            <p style={{ fontSize: "0.85rem", color: TOKENS.textSecondary, marginBottom: 16 }}>
              Enter the 6-digit code from your authenticator app.
            </p>
            <div style={{ display: "flex", gap: 8, justifyContent: "center", marginBottom: 24 }}>
              {Array.from({ length: 6 }).map((_, i) => (
                <input
                  key={i}
                  ref={(el) => { mfaInputsRef.current[i] = el; }}
                  type="text"
                  inputMode="numeric"
                  maxLength={1}
                  value={mfaCode[i] || ""}
                  onChange={(e) => handleMfaDigitChange(i, e.target.value)}
                  onKeyDown={(e) => handleMfaKeyDown(i, e)}
                  style={{
                    width: 44,
                    height: 52,
                    textAlign: "center",
                    fontSize: "1.25rem",
                    fontWeight: 600,
                    fontFamily: TOKENS.fontUi,
                    background: TOKENS.bg,
                    border: `2px solid ${mfaCode[i] ? TOKENS.primary : TOKENS.border}`,
                    borderRadius: TOKENS.radiusSm,
                    color: TOKENS.text,
                    transition: "border-color 150ms ease",
                  }}
                />
              ))}
            </div>
            <button type="submit" disabled={loading} style={buttonStyle}>
              {loading ? <Loader2 size={16} style={{ animation: "spin 0.8s linear infinite" }} /> : "Verify Code"}
            </button>
            <button
              type="button"
              onClick={resetToCredentials}
              style={{ width: "100%", padding: "10px 16px", borderRadius: TOKENS.radiusSm, fontSize: "0.85rem", color: TOKENS.textTertiary, cursor: "pointer", marginTop: 8, background: "none", border: "none", display: "flex", alignItems: "center", justifyContent: "center", gap: 4 }}
            >
              <ArrowLeft size={14} /> Back to sign in
            </button>
          </form>
        )}

        {step === "recovery" && (
          <form onSubmit={handleRecoverySubmit}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 20, color: TOKENS.accent, fontSize: "0.85rem" }}>
              <KeyRound size={18} />
              <span style={{ fontWeight: 600 }}>Recovery Code</span>
            </div>
            <p style={{ fontSize: "0.85rem", color: TOKENS.textSecondary, marginBottom: 16 }}>
              Enter one of your recovery codes to regain access.
            </p>
            <div style={{ marginBottom: 24 }}>
              <label style={labelStyle}>Recovery Code</label>
              <input
                type="text"
                value={recoveryCode}
                onChange={(e) => setRecoveryCode(e.target.value)}
                placeholder="XXXXX-XXXXX"
                style={{ ...inputStyle, textAlign: "center", fontSize: "1rem", letterSpacing: "0.15em", fontFamily: TOKENS.fontUi, textTransform: "uppercase" }}
              />
            </div>
            <button type="submit" disabled={loading} style={buttonStyle}>
              {loading ? <Loader2 size={16} style={{ animation: "spin 0.8s linear infinite" }} /> : "Verify Recovery Code"}
            </button>
            <button
              type="button"
              onClick={resetToCredentials}
              style={{ width: "100%", padding: "10px 16px", borderRadius: TOKENS.radiusSm, fontSize: "0.85rem", color: TOKENS.textTertiary, cursor: "pointer", marginTop: 8, background: "none", border: "none", display: "flex", alignItems: "center", justifyContent: "center", gap: 4 }}
            >
              <ArrowLeft size={14} /> Back to sign in
            </button>
          </form>
        )}

        {step === "forgot_password" && (
          <form onSubmit={handleForgotPassword}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 20, color: TOKENS.primary, fontSize: "0.85rem" }}>
              <Mail size={18} />
              <span style={{ fontWeight: 600 }}>Reset Password</span>
            </div>
            {forgotSent ? (
              <div style={{ textAlign: "center", padding: "16px 0" }}>
                <div style={{ width: 48, height: 48, borderRadius: "50%", background: TOKENS.successLight, display: "flex", alignItems: "center", justifyContent: "center", margin: "0 auto 12px" }}>
                  <Mail size={24} style={{ color: TOKENS.success }} />
                </div>
                <p style={{ fontSize: "0.9rem", color: TOKENS.text, fontWeight: 500, marginBottom: 4 }}>Check your email</p>
                <p style={{ fontSize: "0.85rem", color: TOKENS.textSecondary }}>We sent a password reset link to <strong>{forgotEmail}</strong></p>
              </div>
            ) : (
              <>
                <p style={{ fontSize: "0.85rem", color: TOKENS.textSecondary, marginBottom: 16 }}>
                  Enter your email and we&apos;ll send you a reset link.
                </p>
                <div style={{ marginBottom: 24 }}>
                  <label style={labelStyle}>Email Address</label>
                  <input
                    type="email"
                    value={forgotEmail}
                    onChange={(e) => setForgotEmail(e.target.value)}
                    placeholder="admin@haqly.com"
                    required
                    style={inputStyle}
                  />
                </div>
                <button type="submit" disabled={loading} style={buttonStyle}>
                  {loading ? <Loader2 size={16} style={{ animation: "spin 0.8s linear infinite" }} /> : "Send Reset Link"}
                </button>
              </>
            )}
            <button
              type="button"
              onClick={resetToCredentials}
              style={{ width: "100%", padding: "10px 16px", borderRadius: TOKENS.radiusSm, fontSize: "0.85rem", color: TOKENS.textTertiary, cursor: "pointer", marginTop: 8, background: "none", border: "none", display: "flex", alignItems: "center", justifyContent: "center", gap: 4 }}
            >
              <ArrowLeft size={14} /> Back to sign in
            </button>
          </form>
        )}

        <div style={{ textAlign: "center", marginTop: 24, paddingTop: 16, borderTop: `1px solid ${TOKENS.borderSubtle}` }}>
          <p style={{ fontSize: "0.75rem", color: TOKENS.textTertiary }}>
            HAQLY ERP v0.1.0 — Quadri Atharu
          </p>
        </div>
      </div>

      <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
    </div>
  );
}
