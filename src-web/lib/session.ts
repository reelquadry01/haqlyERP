// Author: Quadri Atharu

const TOKEN_KEY = "haqly_session";
const COMPANY_KEY = "haqly_company";

async function storeTokenSecure(token: string): Promise<void> {
  if (typeof window !== "undefined" && (window as any).__TAURI__) {
    try {
      const { load } = await import("@tauri-apps/plugin-store");
      const store = await load(".credentials.json", { encryptionKey: "haqly-erp-store-key" } as any);
      await store.set(TOKEN_KEY, token);
      await store.save();
      return;
    } catch {
      // fallback
    }
  }
  sessionStorage.setItem(TOKEN_KEY, token);
}

async function retrieveTokenSecure(): Promise<string | null> {
  if (typeof window !== "undefined" && (window as any).__TAURI__) {
    try {
      const { load } = await import("@tauri-apps/plugin-store");
      const store = await load(".credentials.json", { encryptionKey: "haqly-erp-store-key" } as any);
      return (await store.get<string>(TOKEN_KEY)) ?? null;
    } catch {
      // fallback
    }
  }
  return sessionStorage.getItem(TOKEN_KEY);
}

async function clearTokenSecure(): Promise<void> {
  if (typeof window !== "undefined" && (window as any).__TAURI__) {
    try {
      const { load } = await import("@tauri-apps/plugin-store");
      const store = await load(".credentials.json", { encryptionKey: "haqly-erp-store-key" } as any);
      await store.delete(TOKEN_KEY);
      await store.delete(COMPANY_KEY);
      await store.save();
      return;
    } catch {
      // fallback
    }
  }
  sessionStorage.removeItem(TOKEN_KEY);
  sessionStorage.removeItem(COMPANY_KEY);
}

async function storeCompanySecure(companyId: string): Promise<void> {
  if (typeof window !== "undefined" && (window as any).__TAURI__) {
    try {
      const { load } = await import("@tauri-apps/plugin-store");
      const store = await load(".credentials.json", { encryptionKey: "haqly-erp-store-key" } as any);
      await store.set(COMPANY_KEY, companyId);
      await store.save();
      return;
    } catch {
      // fallback
    }
  }
  sessionStorage.setItem(COMPANY_KEY, companyId);
}

async function retrieveCompanySecure(): Promise<string | null> {
  if (typeof window !== "undefined" && (window as any).__TAURI__) {
    try {
      const { load } = await import("@tauri-apps/plugin-store");
      const store = await load(".credentials.json", { encryptionKey: "haqly-erp-store-key" } as any);
      return (await store.get<string>(COMPANY_KEY)) ?? null;
    } catch {
      // fallback
    }
  }
  return sessionStorage.getItem(COMPANY_KEY);
}

export function saveToken(token: string): void {
  storeTokenSecure(token).catch(() => {
    sessionStorage.setItem(TOKEN_KEY, token);
  });
}

export function getToken(): string | null {
  if (typeof window === "undefined") return null;
  return sessionStorage.getItem(TOKEN_KEY);
}

export async function getTokenAsync(): Promise<string | null> {
  return retrieveTokenSecure();
}

export function clearToken(): void {
  clearTokenSecure().catch(() => {
    sessionStorage.removeItem(TOKEN_KEY);
    sessionStorage.removeItem(COMPANY_KEY);
  });
}

export function saveCompanyContext(companyId: string): void {
  storeCompanySecure(companyId).catch(() => {
    sessionStorage.setItem(COMPANY_KEY, companyId);
  });
}

export function getCompanyContext(): string | null {
  if (typeof window === "undefined") return null;
  return sessionStorage.getItem(COMPANY_KEY);
}

export function isAuthenticated(): boolean {
  return !!getToken();
}
