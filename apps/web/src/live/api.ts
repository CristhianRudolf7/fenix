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
  const value = await response.json();
  if (!response.ok)
    throw new Error(serviceError(value.error || `HTTP ${response.status}`));
  return value as T;
}
export interface LiveConfiguration {
  provider: string;
  model: string;
  base_url: string | null;
  reasoning_effort?: string | null;
}
export interface LiveStatus {
  configured: boolean;
  settings: LiveConfiguration;
}
export interface TurnReply {
  reply: string;
  delegated: boolean;
  dispatch?: unknown;
}
