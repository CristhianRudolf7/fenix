export type Step = {
  id: string;
  label: string;
  kind: string;
  depends_on: string[];
  tool?: string | null;
  arguments: Record<string, unknown>;
  when?: { reference: string; operator: string; value?: unknown } | null;
  steps?: { id: string; tool: string; arguments: Record<string, unknown> }[];
  attempts?: number;
};
export type Flow = {
  id: string;
  name: string;
  description: string;
  version: number;
  project_id?: string | null;
  nodes: Step[];
  updated_at: string;
};
export type Run = {
  id: string;
  task_id: string;
  workflow: Flow;
  status: string;
  created_at: string;
  test_mode: boolean;
  error?: string;
  nodes: Record<
    string,
    {
      status: string;
      attempts: number;
      output: unknown;
      error?: string;
      started_at?: string;
      finished_at?: string;
    }
  >;
};
export type Schedule = {
  id?: string;
  name: string;
  content: string;
  mode: string;
  kind: string;
  at?: string | null;
  cron?: string | null;
  interval_seconds?: number | null;
  project_id?: string | null;
  workflow_id?: string | null;
  status?: string;
  next_run?: string | null;
  last_result?: string | null;
};
export const tr = (language: string) => (en: string, es: string) =>
  language === "es" ? es : en;
export function stepLabel(kind: string, language: string) {
  const labels: Record<string, [string, string]> = {
    connector: ["Connection", "Conexión"],
    browser: ["Browser", "Navegador"],
    script: ["Script", "Script"],
    prompt: ["Agent task", "Tarea del agente"],
    condition: ["Condition", "Condición"],
    wait: ["Your response", "Tu respuesta"],
    notify: ["Notification", "Notificación"],
    tool: ["Tool", "Herramienta"],
  };
  return labels[kind]?.[language === "es" ? 1 : 0] || kind;
}
export function stateLabel(status: string, language: string) {
  const words: Record<string, string> = {
    pending: "Pendiente",
    queued: "En cola",
    running: "En curso",
    waiting: "Espera tu respuesta",
    completed: "Completada",
    failed: "Necesita atención",
    skipped: "Omitida",
    interrupted: "Interrumpida",
    cancelled: "Cancelada",
    paused: "En pausa",
    active: "Activa",
    needs_confirmation: "Revisar antes de repetir",
  };
  return language === "es"
    ? words[status] || status
    : status.replaceAll("_", " ");
}
