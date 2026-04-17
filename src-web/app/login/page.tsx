"use client";

import { useState, FormEvent, useRef, useEffect } from "react";
import { apiPost } from "@/lib/api";
import { saveToken, saveCompanyContext } from "@/lib/session";
import {
  Eye, EyeOff, Shield, AlertCircle, Loader2, KeyRound,
  ArrowLeft, Mail, Lock, Building2, TrendingUp, Receipt,
  Brain, UserPlus, Check, ChevronRight,
} from "lucide-react";

interface LoginResponse {
  token: string;
  user: { id: string; email: string; name: string };
  companies: { id: string; name: string }[];
}

type AuthTab = "login" | "register";
type LoginStep = "credentials" | "mfa" | "recovery" | "forgot_password";

export default function LoginPage() {
  const [appLoading, setAppLoading] = useState(true);
  const [tab, setTab] = useState<AuthTab>("login");
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
  const [success, setSuccess] = useState<string | null>(null);
  const [mfaAttempts, setMfaAttempts] = useState(0);
  const [forgotEmail, setForgotEmail] = useState("");
  const [forgotSent, setForgotSent] = useState(false);
  const [regName, setRegName] = useState("");
  const [regCompany, setRegCompany] = useState("");
  const [regEmail, setRegEmail] = useState("");
  const [regPassword, setRegPassword] = useState("");
  const [regConfirm, setRegConfirm] = useState("");
  const [regAgree, setRegAgree] = useState(false);
  const mfaInputsRef = useRef<(HTMLInputElement | null)[]>([]);

  useEffect(() => { const t = setTimeout(() => setAppLoading(false), 1500); return () => clearTimeout(t); }, []);
  useEffect(() => { const r = localStorage.getItem("haqly_remember_email"); if (r) { setEmail(r); setRememberMe(true); } }, []);
  useEffect(() => { if (step === "mfa" && mfaInputsRef.current[0]) mfaInputsRef.current[0].focus(); }, [step]);

  function handleMfaDigitChange(i: number, v: string) {
    if (!/^\d*$/.test(v)) return;
    const d = mfaCode.split(""); while (d.length < 6) d.push("");
    d[i] = v.slice(-1); const c = d.join(""); setMfaCode(c);
    if (v && i < 5) mfaInputsRef.current[i + 1]?.focus();
  }
  function handleMfaKeyDown(i: number, e: React.KeyboardEvent) {
    if (e.key === "Backspace" && !mfaCode[i] && i > 0) mfaInputsRef.current[i - 1]?.focus();
  }

  async function handleLogin(e: FormEvent) {
    e.preventDefault(); setError(null); setLoading(true);
    try {
      const res = await apiPost("/auth/login", { email, password, remember_me: rememberMe });
      const data = await res.json();
      if (!res.ok) {
        if (data.code === "MFA_REQUIRED") { setMfaToken(data.mfaToken); setStep("mfa"); return; }
        setError(data.message || "Invalid email or password"); return;
      }
      if (data.mfaRequired) { setMfaToken(data.mfaToken); setStep("mfa"); return; }
      const ld = data as LoginResponse; saveToken(ld.token);
      if (rememberMe) localStorage.setItem("haqly_remember_email", email);
      if (ld.companies?.length > 0) saveCompanyContext(ld.companies[0].id);
      window.location.replace("/dashboard");
    } catch { setError("Unable to connect to server."); } finally { setLoading(false); }
  }

  async function handleRegister(e: FormEvent) {
    e.preventDefault(); setError(null);
    if (regPassword !== regConfirm) { setError("Passwords do not match"); return; }
    if (!regAgree) { setError("Please agree to the Terms of Service"); return; }
    setLoading(true);
    try {
      const res = await apiPost("/auth/register", { email: regEmail, password: regPassword, full_name: regName, company_name: regCompany });
      if (res.ok) { setSuccess("Account created! You can now sign in."); setTab("login"); setEmail(regEmail); }
      else { const d = await res.json(); setError(d.message || "Registration failed"); }
    } catch { setError("Unable to connect to server."); } finally { setLoading(false); }
  }

  async function handleMfa(e: FormEvent) {
    e.preventDefault(); if (mfaCode.length < 6) { setError("Enter all 6 digits"); return; }
    setError(null); setLoading(true);
    try {
      const res = await apiPost("/auth/mfa/verify", { mfaToken, code: mfaCode }); const d = await res.json();
      if (!res.ok) { const a = mfaAttempts + 1; setMfaAttempts(a);
        if (a >= 3) { setError("Too many attempts. Use recovery."); setStep("recovery"); return; }
        setError(d.message || `Invalid code. ${3 - a} left.`); setMfaCode(""); return; }
      const ld = d as LoginResponse; saveToken(ld.token);
      if (ld.companies?.length > 0) saveCompanyContext(ld.companies[0].id);
      window.location.replace("/dashboard");
    } catch { setError("Connection error."); } finally { setLoading(false); }
  }

  async function handleRecovery(e: FormEvent) {
    e.preventDefault(); if (!recoveryCode.trim()) { setError("Enter recovery code"); return; }
    setError(null); setLoading(true);
    try {
      const res = await apiPost("/auth/mfa/recover", { mfaToken, recoveryCode: recoveryCode.trim() }); const d = await res.json();
      if (!res.ok) { setError(d.message || "Invalid code"); return; }
      const ld = d as LoginResponse; saveToken(ld.token);
      if (ld.companies?.length > 0) saveCompanyContext(ld.companies[0].id);
      window.location.replace("/dashboard");
    } catch { setError("Connection error."); } finally { setLoading(false); }
  }

  async function handleForgot(e: FormEvent) {
    e.preventDefault(); setError(null); setLoading(true);
    try {
      const res = await apiPost("/auth/forgot-password", { email: forgotEmail });
      if (res.ok) setForgotSent(true); else { const d = await res.json(); setError(d.message || "Failed"); }
    } catch { setError("Connection error."); } finally { setLoading(false); }
  }

  function resetStep() { setStep("credentials"); setMfaCode(""); setMfaToken(""); setRecoveryCode(""); setError(null); setMfaAttempts(0); }

  const inp: React.CSSProperties = { width: "100%", padding: "11px 14px 11px 38px", fontSize: "0.9rem", background: "#FFF", border: "1.5px solid #DEE2E6", borderRadius: 8, color: "#1A1A2E", outline: "none", transition: "border-color .2s,box-shadow .2s" };
  const lbl: React.CSSProperties = { display: "block", marginBottom: 6, fontSize: "0.78rem", fontWeight: 600, color: "#495057" };
  const btn: React.CSSProperties = { width: "100%", padding: "12px 16px", borderRadius: 8, fontSize: "0.9rem", fontWeight: 600, background: "#1B4332", color: "#FFF", border: "none", cursor: "pointer", display: "flex", alignItems: "center", justifyContent: "center", gap: 8, transition: "background .15s,transform .1s,box-shadow .15s", boxShadow: "0 1px 3px rgba(27,67,50,.2)" };
  const backBtn: React.CSSProperties = { width: "100%", padding: 10, borderRadius: 8, fontSize: "0.82rem", color: "#868E96", cursor: "pointer", marginTop: 8, background: "none", border: "none", display: "flex", alignItems: "center", justifyContent: "center", gap: 4 };

  if (appLoading) return (
    <div style={{ minHeight: "100vh", display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", background: "#1B4332" }}>
      <style>{`@keyframes pulse-ring{0%{transform:scale(.9);opacity:1}50%{transform:scale(1.15);opacity:.5}100%{transform:scale(.9);opacity:1}}@keyframes fade-in{from{opacity:0;transform:translateY(8px)}to{opacity:1;transform:translateY(0)}}`}</style>
      <div style={{ width: 64, height: 64, borderRadius: 14, background: "linear-gradient(135deg,#1B4332,#2D6A4F)", display: "flex", alignItems: "center", justifyContent: "center", fontWeight: 800, fontSize: "1.6rem", color: "#D4AF37", boxShadow: "0 0 0 0 rgba(212,175,55,.4)", animation: "pulse-ring 2s ease-in-out infinite" }}>H</div>
      <div style={{ marginTop: 20, fontSize: "0.9rem", color: "rgba(255,255,255,.6)", animation: "fade-in .6s ease .3s both" }}>Loading HAQLY ERP…</div>
    </div>
  );

  const features = [
    { icon: Receipt, t: "Tax Compliance", d: "Nigeria Tax Reform 2025" },
    { icon: Building2, t: "E-Invoicing", d: "NRS integration" },
    { icon: TrendingUp, t: "15 Industries", d: "Industry profiles" },
    { icon: Brain, t: "AI Powered", d: "6 intelligent agents" },
  ];
  const tiers = [
    { name: "Starter", price: "₦50K", period: "/mo" },
    { name: "Professional", price: "₦150K", period: "/mo" },
    { name: "Enterprise", price: "₦500K", period: "/mo" },
  ];

  return (
    <div style={{ minHeight: "100vh", display: "flex", fontFamily: '"Inter",-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif', background: "#FAFBFC" }}>
      <style>{`@keyframes spin{to{transform:rotate(360deg)}}@keyframes fu{from{opacity:0;transform:translateY(14px)}to{opacity:1;transform:translateY(0)}}.fu{animation:fu .45s ease both}.fu1{animation-delay:.06s}.fu2{animation-delay:.12s}.fu3{animation-delay:.18s}.fu4{animation-delay:.24s}.fu5{animation-delay:.30s}.fu6{animation-delay:.36s}@media(min-width:960px){.brand-panel{display:flex!important}}`}</style>

      <div className="brand-panel" style={{ flex: 1, display: "none", background: "linear-gradient(160deg,#1B4332 0%,#2D6A4F 40%,#1B4332 100%)", flexDirection: "column", justifyContent: "center", padding: "56px 64px", position: "relative", overflow: "hidden" }}>
        <div style={{ position: "absolute", inset: 0, background: "radial-gradient(ellipse at 70% 20%,rgba(212,175,55,.1) 0%,transparent 60%)" }} />
        <div style={{ position: "relative", zIndex: 1 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 44 }}>
            <div style={{ width: 42, height: 42, borderRadius: 10, background: "linear-gradient(135deg,#D4AF37,#C4A030)", display: "flex", alignItems: "center", justifyContent: "center", fontWeight: 800, fontSize: "1.15rem", color: "#1B4332" }}>H</div>
            <span style={{ fontSize: "1.05rem", fontWeight: 700, color: "#FFF" }}>HAQLY ERP</span>
          </div>
          <h1 style={{ fontSize: "2.3rem", fontWeight: 700, color: "#FFF", lineHeight: 1.15, marginBottom: 14, letterSpacing: "-0.03em", fontFamily: '"DM Serif Display",Georgia,serif' }}>
            Your Full Finance<br/>Department in<br/><span style={{ color: "#D4AF37" }}>Software Form</span>
          </h1>
          <p style={{ fontSize: "0.95rem", color: "rgba(255,255,255,.65)", lineHeight: 1.6, marginBottom: 44, maxWidth: 380 }}>
            Nigerian accounting, tax compliance &amp; business intelligence — built for how Nigeria works.
          </p>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 14 }}>
            {features.map((f, i) => (
              <div key={i} className={`fu fu${i + 1}`} style={{ display: "flex", gap: 10, padding: "12px 14px", borderRadius: 10, background: "rgba(255,255,255,.06)", border: "1px solid rgba(255,255,255,.08)" }}>
                <f.icon size={18} style={{ color: "#D4AF37", flexShrink: 0, marginTop: 2 }} />
                <div><div style={{ fontSize: "0.82rem", fontWeight: 600, color: "#FFF", marginBottom: 1 }}>{f.t}</div><div style={{ fontSize: "0.72rem", color: "rgba(255,255,255,.5)" }}>{f.d}</div></div>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", background: "#FAFBFC", padding: 24, minHeight: "100vh" }}>
        <div style={{ width: 400, maxWidth: "100%" }}>
          <div className="fu" style={{ marginBottom: 28 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 20 }}>
              <div style={{ width: 34, height: 34, borderRadius: 8, background: "linear-gradient(135deg,#1B4332,#2D6A4F)", display: "flex", alignItems: "center", justifyContent: "center", fontWeight: 800, fontSize: ".95rem", color: "#D4AF37" }}>H</div>
              <span style={{ fontSize: ".9rem", fontWeight: 700, color: "#1A1A2E" }}>HAQLY ERP</span>
            </div>

            {step === "credentials" && (
              <div style={{ display: "flex", background: "#F1F3F5", borderRadius: 8, padding: 3, marginBottom: 20 }}>
                {(["login", "register"] as AuthTab[]).map(t => (
                  <button key={t} onClick={() => { setTab(t); setError(null); setSuccess(null); }} style={{ flex: 1, padding: "9px 0", borderRadius: 6, fontSize: ".85rem", fontWeight: 600, border: "none", cursor: "pointer", transition: "all .2s", background: tab === t ? "#FFF" : "transparent", color: tab === t ? "#1B4332" : "#868E96", boxShadow: tab === t ? "0 1px 3px rgba(0,0,0,.08)" : "none" }}>
                    {t === "login" ? "Sign In" : "Register"}
                  </button>
                ))}
              </div>
            )}

            {step === "credentials" && tab === "login" && (
              <>
                <h2 style={{ fontSize: "1.5rem", fontWeight: 700, color: "#1A1A2E", letterSpacing: "-0.02em", marginBottom: 4 }}>Welcome back</h2>
                <p style={{ fontSize: ".88rem", color: "#868E96", marginBottom: 0 }}>Sign in to your account</p>
              </>
            )}
            {step === "credentials" && tab === "register" && (
              <>
                <h2 style={{ fontSize: "1.5rem", fontWeight: 700, color: "#1A1A2E", letterSpacing: "-0.02em", marginBottom: 4 }}>Create account</h2>
                <p style={{ fontSize: ".88rem", color: "#868E96", marginBottom: 0 }}>Start your free trial today</p>
              </>
            )}
            {step === "mfa" && <><h2 style={{ fontSize: "1.5rem", fontWeight: 700, color: "#1A1A2E", marginBottom: 4 }}>Two-factor auth</h2><p style={{ fontSize: ".88rem", color: "#868E96" }}>Enter the code from your authenticator</p></>}
            {step === "recovery" && <><h2 style={{ fontSize: "1.5rem", fontWeight: 700, color: "#1A1A2E", marginBottom: 4 }}>Recovery code</h2><p style={{ fontSize: ".88rem", color: "#868E96" }}>Use a recovery code to regain access</p></>}
            {step === "forgot_password" && <><h2 style={{ fontSize: "1.5rem", fontWeight: 700, color: "#1A1A2E", marginBottom: 4 }}>Reset password</h2><p style={{ fontSize: ".88rem", color: "#868E96" }}>We'll send you a reset link</p></>}
          </div>

          {error && <div className="fu" style={{ display: "flex", alignItems: "center", gap: 8, padding: "10px 14px", marginBottom: 18, background: "rgba(220,38,38,.06)", border: "1px solid rgba(220,38,38,.2)", borderRadius: 8, color: "#DC2626", fontSize: ".84rem" }}><AlertCircle size={15} style={{ flexShrink: 0 }} />{error}</div>}
          {success && <div className="fu" style={{ display: "flex", alignItems: "center", gap: 8, padding: "10px 14px", marginBottom: 18, background: "rgba(22,163,74,.06)", border: "1px solid rgba(22,163,74,.2)", borderRadius: 8, color: "#16A34A", fontSize: ".84rem" }}><Check size={15} style={{ flexShrink: 0 }} />{success}</div>}

          {step === "credentials" && tab === "login" && (
            <form onSubmit={handleLogin}>
              <div className="fu fu1" style={{ marginBottom: 16 }}>
                <label style={lbl}>Email</label>
                <div style={{ position: "relative" }}><Mail size={16} style={{ position: "absolute", left: 12, top: "50%", transform: "translateY(-50%)", color: "#ADB5BD" }} /><input type="email" value={email} onChange={e => setEmail(e.target.value)} placeholder="you@company.com" required style={inp} onFocus={e => { e.currentTarget.style.borderColor = "#1B4332"; e.currentTarget.style.boxShadow = "0 0 0 3px rgba(27,67,50,.1)"; }} onBlur={e => { e.currentTarget.style.borderColor = "#DEE2E6"; e.currentTarget.style.boxShadow = "none"; }} /></div>
              </div>
              <div className="fu fu2" style={{ marginBottom: 16 }}>
                <label style={lbl}>Password</label>
                <div style={{ position: "relative" }}><Lock size={16} style={{ position: "absolute", left: 12, top: "50%", transform: "translateY(-50%)", color: "#ADB5BD" }} /><input type={showPassword ? "text" : "password"} value={password} onChange={e => setPassword(e.target.value)} placeholder="Enter your password" required style={{ ...inp, paddingRight: 42 }} onFocus={e => { e.currentTarget.style.borderColor = "#1B4332"; e.currentTarget.style.boxShadow = "0 0 0 3px rgba(27,67,50,.1)"; }} onBlur={e => { e.currentTarget.style.borderColor = "#DEE2E6"; e.currentTarget.style.boxShadow = "none"; }} /><button type="button" onClick={() => setShowPassword(!showPassword)} style={{ position: "absolute", right: 10, top: "50%", transform: "translateY(-50%)", color: "#ADB5BD", cursor: "pointer", background: "none", border: "none", padding: 4 }}>{showPassword ? <EyeOff size={16} /> : <Eye size={16} />}</button></div>
              </div>
              <div className="fu fu3" style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 22 }}>
                <label style={{ display: "flex", alignItems: "center", gap: 7, fontSize: ".82rem", color: "#495057", cursor: "pointer" }}><input type="checkbox" checked={rememberMe} onChange={e => setRememberMe(e.target.checked)} style={{ accentColor: "#1B4332", width: 15, height: 15 }} />Remember me</label>
                <button type="button" onClick={() => { setStep("forgot_password"); setForgotEmail(email); setError(null); setForgotSent(false); }} style={{ fontSize: ".82rem", color: "#1B4332", fontWeight: 600, cursor: "pointer", background: "none", border: "none" }}>Forgot password?</button>
              </div>
              <button type="submit" disabled={loading} style={btn} onMouseEnter={e => !loading && (e.currentTarget.style.background = "#2D6A4F")} onMouseLeave={e => !loading && (e.currentTarget.style.background = "#1B4332")}>{loading ? <Loader2 size={16} style={{ animation: "spin .8s linear infinite" }} /> : "Sign in"}</button>
            </form>
          )}

          {step === "credentials" && tab === "register" && (
            <form onSubmit={handleRegister}>
              <div className="fu fu1" style={{ marginBottom: 14 }}>
                <label style={lbl}>Full Name</label>
                <div style={{ position: "relative" }}><UserPlus size={16} style={{ position: "absolute", left: 12, top: "50%", transform: "translateY(-50%)", color: "#ADB5BD" }} /><input type="text" value={regName} onChange={e => setRegName(e.target.value)} placeholder="Adebayo Okonkwo" required style={inp} onFocus={e => { e.currentTarget.style.borderColor = "#1B4332"; e.currentTarget.style.boxShadow = "0 0 0 3px rgba(27,67,50,.1)"; }} onBlur={e => { e.currentTarget.style.borderColor = "#DEE2E6"; e.currentTarget.style.boxShadow = "none"; }} /></div>
              </div>
              <div className="fu fu2" style={{ marginBottom: 14 }}>
                <label style={lbl}>Company Name</label>
                <div style={{ position: "relative" }}><Building2 size={16} style={{ position: "absolute", left: 12, top: "50%", transform: "translateY(-50%)", color: "#ADB5BD" }} /><input type="text" value={regCompany} onChange={e => setRegCompany(e.target.value)} placeholder="Your Company Ltd" required style={inp} onFocus={e => { e.currentTarget.style.borderColor = "#1B4332"; e.currentTarget.style.boxShadow = "0 0 0 3px rgba(27,67,50,.1)"; }} onBlur={e => { e.currentTarget.style.borderColor = "#DEE2E6"; e.currentTarget.style.boxShadow = "none"; }} /></div>
              </div>
              <div className="fu fu3" style={{ marginBottom: 14 }}>
                <label style={lbl}>Email</label>
                <div style={{ position: "relative" }}><Mail size={16} style={{ position: "absolute", left: 12, top: "50%", transform: "translateY(-50%)", color: "#ADB5BD" }} /><input type="email" value={regEmail} onChange={e => setRegEmail(e.target.value)} placeholder="you@company.com" required style={inp} onFocus={e => { e.currentTarget.style.borderColor = "#1B4332"; e.currentTarget.style.boxShadow = "0 0 0 3px rgba(27,67,50,.1)"; }} onBlur={e => { e.currentTarget.style.borderColor = "#DEE2E6"; e.currentTarget.style.boxShadow = "none"; }} /></div>
              </div>
              <div className="fu fu4" style={{ marginBottom: 14 }}>
                <label style={lbl}>Password</label>
                <div style={{ position: "relative" }}><Lock size={16} style={{ position: "absolute", left: 12, top: "50%", transform: "translateY(-50%)", color: "#ADB5BD" }} /><input type="password" value={regPassword} onChange={e => setRegPassword(e.target.value)} placeholder="Min 8 characters" required style={inp} onFocus={e => { e.currentTarget.style.borderColor = "#1B4332"; e.currentTarget.style.boxShadow = "0 0 0 3px rgba(27,67,50,.1)"; }} onBlur={e => { e.currentTarget.style.borderColor = "#DEE2E6"; e.currentTarget.style.boxShadow = "none"; }} /></div>
              </div>
              <div className="fu fu5" style={{ marginBottom: 14 }}>
                <label style={lbl}>Confirm Password</label>
                <div style={{ position: "relative" }}><Lock size={16} style={{ position: "absolute", left: 12, top: "50%", transform: "translateY(-50%)", color: "#ADB5BD" }} /><input type="password" value={regConfirm} onChange={e => setRegConfirm(e.target.value)} placeholder="Re-enter password" required style={inp} onFocus={e => { e.currentTarget.style.borderColor = "#1B4332"; e.currentTarget.style.boxShadow = "0 0 0 3px rgba(27,67,50,.1)"; }} onBlur={e => { e.currentTarget.style.borderColor = "#DEE2E6"; e.currentTarget.style.boxShadow = "none"; }} /></div>
              </div>
              <div className="fu fu6" style={{ marginBottom: 20 }}>
                <label style={{ display: "flex", alignItems: "flex-start", gap: 8, fontSize: ".8rem", color: "#495057", cursor: "pointer" }}>
                  <input type="checkbox" checked={regAgree} onChange={e => setRegAgree(e.target.checked)} style={{ accentColor: "#1B4332", width: 15, height: 15, marginTop: 2, flexShrink: 0 }} />
                  <span>I agree to the <span style={{ color: "#1B4332", fontWeight: 600 }}>Terms of Service</span> and <span style={{ color: "#1B4332", fontWeight: 600 }}>Privacy Policy</span></span>
                </label>
              </div>
              <button type="submit" disabled={loading} style={btn} onMouseEnter={e => !loading && (e.currentTarget.style.background = "#2D6A4F")} onMouseLeave={e => !loading && (e.currentTarget.style.background = "#1B4332")}>{loading ? <Loader2 size={16} style={{ animation: "spin .8s linear infinite" }} /> : "Create Account"}</button>
            </form>
          )}

          {step === "mfa" && (
            <form onSubmit={handleMfa}>
              <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 16, color: "#1B4332" }}><Shield size={18} /><span style={{ fontWeight: 600, fontSize: ".88rem" }}>Verification required</span></div>
              <div style={{ display: "flex", gap: 8, justifyContent: "center", marginBottom: 22 }}>
                {Array.from({ length: 6 }).map((_, i) => (<input key={i} ref={el => { mfaInputsRef.current[i] = el; }} type="text" inputMode="numeric" maxLength={1} value={mfaCode[i] || ""} onChange={e => handleMfaDigitChange(i, e.target.value)} onKeyDown={e => handleMfaKeyDown(i, e)} style={{ width: 46, height: 54, textAlign: "center", fontSize: "1.2rem", fontWeight: 600, background: "#FFF", border: `2px solid ${mfaCode[i] ? "#1B4332" : "#DEE2E6"}`, borderRadius: 8, color: "#1A1A2E", outline: "none", transition: "border-color .15s" }} />))}
              </div>
              <button type="submit" disabled={loading} style={btn}>{loading ? <Loader2 size={16} style={{ animation: "spin .8s linear infinite" }} /> : "Verify"}</button>
              <button type="button" onClick={resetStep} style={backBtn}><ArrowLeft size={14} /> Back to sign in</button>
            </form>
          )}

          {step === "recovery" && (
            <form onSubmit={handleRecovery}>
              <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 16, color: "#D4AF37" }}><KeyRound size={18} /><span style={{ fontWeight: 600, fontSize: ".88rem" }}>Recovery Code</span></div>
              <div style={{ marginBottom: 22 }}><input type="text" value={recoveryCode} onChange={e => setRecoveryCode(e.target.value)} placeholder="XXXXX-XXXXX" style={{ width: "100%", padding: "12px 14px", borderRadius: 8, fontSize: "1rem", letterSpacing: ".12em", textAlign: "center", fontWeight: 600, background: "#FFF", border: "1.5px solid #DEE2E6", color: "#1A1A2E", outline: "none", textTransform: "uppercase" }} /></div>
              <button type="submit" disabled={loading} style={btn}>{loading ? <Loader2 size={16} style={{ animation: "spin .8s linear infinite" }} /> : "Verify Code"}</button>
              <button type="button" onClick={resetStep} style={backBtn}><ArrowLeft size={14} /> Back to sign in</button>
            </form>
          )}

          {step === "forgot_password" && (
            <form onSubmit={handleForgot}>
              {forgotSent ? (
                <div style={{ textAlign: "center", padding: "20px 0" }}>
                  <div style={{ width: 50, height: 50, borderRadius: "50%", background: "rgba(22,163,74,.08)", display: "flex", alignItems: "center", justifyContent: "center", margin: "0 auto 14px" }}><Mail size={24} style={{ color: "#16A34A" }} /></div>
                  <p style={{ fontSize: ".92rem", fontWeight: 600, color: "#1A1A2E", marginBottom: 4 }}>Check your email</p>
                  <p style={{ fontSize: ".84rem", color: "#868E96" }}>Link sent to <strong>{forgotEmail}</strong></p>
                </div>
              ) : (<>
                <div style={{ marginBottom: 20 }}><label style={lbl}>Email</label><input type="email" value={forgotEmail} onChange={e => setForgotEmail(e.target.value)} placeholder="you@company.com" required style={{ ...inp, paddingLeft: 14 }} /></div>
                <button type="submit" disabled={loading} style={btn}>{loading ? <Loader2 size={16} style={{ animation: "spin .8s linear infinite" }} /> : "Send Reset Link"}</button>
              </>)}
              <button type="button" onClick={resetStep} style={backBtn}><ArrowLeft size={14} /> Back to sign in</button>
            </form>
          )}

          {step === "credentials" && (
            <div style={{ marginTop: 24, padding: "16px", background: "#F8F9FA", borderRadius: 8, border: "1px solid #E9ECEF" }}>
              <p style={{ fontSize: ".76rem", fontWeight: 600, color: "#495057", marginBottom: 10 }}>New to HAQLY ERP?</p>
              <div style={{ display: "flex", gap: 8, marginBottom: 8 }}>
                {tiers.map((t, i) => (
                  <div key={i} style={{ flex: 1, textAlign: "center", padding: "8px 4px", borderRadius: 6, background: "#FFF", border: "1px solid #E9ECEF" }}>
                    <div style={{ fontSize: ".7rem", fontWeight: 600, color: i === 1 ? "#1B4332" : "#495057", marginBottom: 2 }}>{t.name}</div>
                    <div style={{ fontSize: ".82rem", fontWeight: 700, color: i === 1 ? "#D4AF37" : "#1A1A2E" }}>{t.price}<span style={{ fontSize: ".65rem", fontWeight: 400, color: "#868E96" }}>{t.period}</span></div>
                  </div>
                ))}
              </div>
              <div style={{ textAlign: "center" }}><span style={{ fontSize: ".72rem", color: "#1B4332", fontWeight: 600, cursor: "pointer" }}>View all plans <ChevronRight size={11} style={{ verticalAlign: "middle" }} /></span></div>
            </div>
          )}

          <div style={{ textAlign: "center", marginTop: 20, paddingTop: 16, borderTop: "1px solid #E9ECEF" }}>
            <p style={{ fontSize: ".7rem", color: "#ADB5BD" }}>HAQLY ERP v0.1.0 · Secure · Nigeria Tax Reform 2025 Compliant</p>
          </div>
        </div>
      </div>
    </div>
  );
}
