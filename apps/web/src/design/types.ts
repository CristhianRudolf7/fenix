import { serviceError } from "../components/service-error";
export type Kind =
  | "web"
  | "diagram"
  | "presentation"
  | "chart"
  | "document"
  | "spreadsheet"
  | "image"
  | "video";
export type Scalar = string | number | boolean | null;
export interface DiagramGraph {
  version: 1;
  direction?: "LR" | "RL" | "TB" | "BT";
  nodes: {
    id: string;
    label: string;
    shape?: "rectangle" | "rounded" | "diamond" | "circle" | "database";
    group?: string;
    detail?: string;
    references?: string[];
    fill?: string;
    stroke?: string;
    color?: string;
  }[];
  edges?: {
    id: string;
    from: string;
    to: string;
    label?: string;
    style?: "solid" | "dashed";
  }[];
  groups?: { id: string; label: string }[];
}
export interface Content {
  graph?: DiagramGraph;
  format?: string;
  composition?: string;
  theme?:
    | "midnight"
    | "cobalt"
    | "coral"
    | "forest"
    | "plum"
    | "sand"
    | "mint"
    | "ink";
  eyebrow?: string;
  display_title?: string;
  html?: string;
  source?: string;
  text?: string;
  svg?: string;
  slides?: {
    title: string;
    body: string;
    notes?: string;
    layout?: "cover" | "split" | "steps" | "statement";
    eyebrow?: string;
  }[];
  chart_type?: "bar" | "line" | "pie";
  labels?: string[];
  values?: number[];
  unit?: string;
  columns?: string[];
  rows?: Scalar[][];
  caption?: string;
  asset_id?: string;
  filename?: string;
  sample?: string;
  brand?: Brand;
  extraction?: { total: number; next_offset: number | null };
}
export interface Brand {
  logo_asset_id?: string | null;
  name: string;
  accent: string;
  background: string;
  text: string;
  font: string;
}
export interface Visual {
  id: string;
  title: string;
  title_es?: string;
  kind: Kind;
  content?: Content;
  revision?: number;
  project_id?: string | null;
  task_id?: string | null;
  template_id?: string | null;
  created_at?: string;
  updated_at?: string;
  tags?: string[];
  source?: string;
}
export interface TemplatePage {
  templates: Visual[];
  total: number;
  next_offset: number | null;
}
export interface ArtifactPage {
  designs: Visual[];
  next_offset: number | null;
}
/** Template proposals the agent showed in chat with `design_present`. */
export interface Offer {
  id: string;
  task_id?: string | null;
  project_id?: string | null;
  templates: Visual[];
  created_at?: string;
}
/** A validated direction the server appends to the user's message. */
export type DesignChoice =
  { mode: "template"; template_id: string } | { mode: "custom" };
export const labels: Record<Kind, [string, string]> = {
  web: ["Web interfaces", "Interfaces web"],
  diagram: ["Diagrams", "Diagramas"],
  presentation: ["Presentations", "Presentaciones"],
  chart: ["Data & statistics", "Datos y estadísticas"],
  document: ["Documents", "Documentos"],
  spreadsheet: ["Spreadsheets", "Hojas de cálculo"],
  image: ["Images", "Imágenes"],
  video: ["Video", "Vídeo"],
};
export async function api<T>(
  path: string,
  body?: unknown,
  method = body === undefined ? "GET" : "POST",
  signal?: AbortSignal,
): Promise<T> {
  const response = await fetch(`/api/design${path}`, {
    method,
    headers: { "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
    signal,
  });
  const data = await response.json();
  if (!response.ok)
    throw new Error(serviceError(data.error || `HTTP ${response.status}`));
  return data as T;
}
export const assetUrl = (id: string) =>
  `/api/design/assets/${encodeURIComponent(id)}`;
export function download(name: string, data: string, mime = "text/plain") {
  const url = URL.createObjectURL(new Blob([data], { type: mime }));
  const link = document.createElement("a");
  link.href = url;
  link.download = name;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
