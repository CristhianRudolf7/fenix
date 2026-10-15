import { serviceError } from "../components/service-error";
export async function api<T>(
  path: string,
  body?: unknown,
  method = body === undefined ? "GET" : "POST",
  signal?: AbortSignal,
): Promise<T> {
  const response = await fetch(`/api${path}`, {
    method,
    headers: { "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
    signal,
  });
  if (response.status === 401) {
    location.reload();
    throw new Error("Session expired");
  }
  const value = await response.json();
  if (!response.ok)
    throw new Error(serviceError(value.error || `HTTP ${response.status}`));
  return value as T;
}
