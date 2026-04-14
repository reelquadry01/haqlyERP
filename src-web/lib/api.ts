const BASE_URL = "http://localhost:8100/api/v1";
import { clearToken } from "./session";

function authHeaders(token: string | null): Record<string, string> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    Accept: "application/json",
  };
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }
  return headers;
}

export function handleAuthError(response: Response): void {
  if (response.status === 401) {
    if (typeof window !== "undefined") {
      clearToken();
      window.location.replace("/login");
    }
  }
}

export async function apiGet(path: string, token: string | null = null): Promise<Response> {
  const response = await fetch(`${BASE_URL}${path}`, {
    method: "GET",
    headers: authHeaders(token),
  });
  handleAuthError(response);
  return response;
}

export async function apiPost(
  path: string,
  body: unknown,
  token: string | null = null
): Promise<Response> {
  const response = await fetch(`${BASE_URL}${path}`, {
    method: "POST",
    headers: authHeaders(token),
    body: JSON.stringify(body),
  });
  handleAuthError(response);
  return response;
}

export async function apiPatch(
  path: string,
  body: unknown,
  token: string | null = null
): Promise<Response> {
  const response = await fetch(`${BASE_URL}${path}`, {
    method: "PATCH",
    headers: authHeaders(token),
    body: JSON.stringify(body),
  });
  handleAuthError(response);
  return response;
}

export async function apiDelete(
  path: string,
  token: string | null = null
): Promise<Response> {
  const response = await fetch(`${BASE_URL}${path}`, {
    method: "DELETE",
    headers: authHeaders(token),
  });
  handleAuthError(response);
  return response;
}
