const TOKEN_KEY = "haqly_token";
const COMPANY_KEY = "haqly_company";

export function saveToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token);
}

export function getToken(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(TOKEN_KEY);
}

export function clearToken(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(COMPANY_KEY);
}

export function saveCompanyContext(companyId: string): void {
  localStorage.setItem(COMPANY_KEY, companyId);
}

export function getCompanyContext(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(COMPANY_KEY);
}

export function isAuthenticated(): boolean {
  return !!getToken();
}
