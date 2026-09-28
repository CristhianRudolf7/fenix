import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { createRoot } from "react-dom/client";
import { Thumbnail } from "./app";
import {
  api,
  kinds,
  labels,
  type Kind,
  type TemplatePage,
  type Visual,
} from "./types";
import { creationKind } from "./creation-intent";
import {
  closeTab,
  openTarget,
  registerAdapter,
  snapshot,
  subscribe as subscribeWorkspace,
  targetKey,
  tr,
} from "../workspace/store";
import { Loading } from "../components/loading";
import "./creation.css";

export type DesignChoice =
  { mode: "template"; template_id: string } | { mode: "custom" };
export type CreationRequest = {
  text: string;
  document_ids: string[];
  reply_to_task_id: string | null;
  design_choice?: DesignChoice;
};
type Draft = {
  request: CreationRequest;
  kind: Kind | "";
  selected: Visual | null;
  submit: (request: CreationRequest) => Promise<void>;
  session: string;
};
let state: {
  draft: Draft | null;
  visible: boolean;
  busy: boolean;
  error: string;
} = { draft: null, visible: false, busy: false, error: "" };
const listeners = new Set<() => void>();
const subscribe = (fn: () => void) => {
  listeners.add(fn);
  return () => {
    listeners.delete(fn);
  };
};
const update = (patch: Partial<typeof state>) => {
  state = { ...state, ...patch };
  listeners.forEach((fn) => fn());
};
const target = { kind: "design-choice" as const, id: "new-design" };
export function finishDesignCreation(text: string) {
  if (state.draft?.request.text.trim() !== text.trim()) return;
  update({ draft: null, visible: false, busy: false, error: "" });
  closeTab(targetKey(target));
}
let initialized = false;
function ensurePicker() {
  if (initialized) return;
  initialized = true;
  const node = document.createElement("div");
  node.id = "design-creation-root";
  document.body.append(node);
  createRoot(node).render(<CreationPicker />);
}
registerAdapter("design-choice", {
  show: () => {
    ensurePicker();
    update({ visible: true });
  },
  hide: () => update({ visible: false }),
});
subscribeWorkspace(() => {
  if (state.draft && state.draft.session !== snapshot().session)
    update({ draft: null, visible: false, busy: false, error: "" });
});

/** Keep the original prompt and source/reply context until the owner starts generation. */
export function openDesignCreation(
  request: CreationRequest,
  submit: Draft["submit"],
): void {
  const kind = creationKind(request.text);
  if (state.busy) return;
  const previous = state.draft;
  const same =
    previous?.request.text === request.text &&
    previous.session === snapshot().session;
  update({
    draft: {
      request,
      kind: same ? previous.kind : kind || "",
      selected: same ? previous.selected : null,
      submit,
      session: snapshot().session,
    },
    error: "",
  });
  openTarget({ ...target, title: tr("Choose a design", "Elegir diseño") });
}

function CreationPicker() {
  const s = useSyncExternalStore(subscribe, () => state);
  const workspace = useSyncExternalStore(subscribeWorkspace, snapshot);
  const [query, setQuery] = useState("");
  const [rows, setRows] = useState<Visual[]>([]);
  const [loading, setLoading] = useState(false);
  const [catalogError, setCatalogError] = useState("");
  const [retry, setRetry] = useState(0);
  const heading = useRef<HTMLHeadingElement>(null);
  const draft = s.draft;
  useEffect(() => {
    if (s.visible) heading.current?.focus();
  }, [s.visible]);
  useEffect(() => {
    if (!s.visible || !draft) return;
    const controller = new AbortController();
    setRows([]);
    setCatalogError("");
    setLoading(true);
    void (async () => {
      let offset: number | null = 0;
      const templates: Visual[] = [];
      do {
        const params: URLSearchParams = new URLSearchParams({
          offset: String(offset),
        });
        if (draft.kind) params.set("kind", draft.kind);
        const page: TemplatePage = await api<TemplatePage>(
          `/templates?${params}`,
          undefined,
          "GET",
          controller.signal,
        );
        templates.push(...page.templates);
        offset = page.next_offset;
      } while (offset !== null && !controller.signal.aborted);
      if (!controller.signal.aborted) setRows(templates);
    })()
      .catch((error: unknown) => {
        if (!controller.signal.aborted)
          setCatalogError(
            error instanceof Error ? error.message : String(error),
          );
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => controller.abort();
  }, [s.visible, draft?.kind, retry]);
  if (!s.visible || !draft) return null;
  const label = (v: Visual) =>
    workspace.language === "es" ? v.title_es || v.title : v.title;
  const normalized = (text: string) =>
    text
      .normalize("NFD")
      .replace(/[\u0300-\u036f]/g, "")
      .toLowerCase();
  const shown = rows.filter((v) =>
    normalized(`${v.title} ${v.title_es} ${v.tags?.join(" ")}`).includes(
      normalized(query),
    ),
  );
  const patchDraft = (patch: Partial<Draft>) =>
    update({ draft: { ...draft, ...patch }, error: "" });
  const generate = async (choice: DesignChoice) => {
    if (state.busy || !draft.request.text.trim()) return;
    const current = state.draft;
    update({ busy: true, error: "" });
    try {
      await draft.submit({
        ...draft.request,
        text: draft.request.text.trim(),
        design_choice: choice,
      });
      if (state.draft === current) {
        update({ draft: null, visible: false, busy: false });
        closeTab(targetKey(target));
      }
    } catch (error) {
      if (state.draft === current)
        update({
          busy: false,
          error: error instanceof Error ? error.message : String(error),
        });
    }
  };
  return (
    <aside
      className="design-panel design-creation-panel"
      role="dialog"
      aria-label={tr("Choose a design", "Elegir diseño")}
      aria-busy={s.busy}
    >
      <header className="design-creation-heading">
        <span className="design-creation-eyebrow">FÉNIX DESIGN</span>
        <h2 ref={heading} tabIndex={-1}>
          {tr(
            "Give your idea a starting point.",
            "Dale un punto de partida a tu idea.",
          )}
        </h2>
        <p>
          {tr(
            "Choose a template, or describe your own design. Generation starts when you continue.",
            "Elige una plantilla o describe tu propio diseño. La generación empieza cuando continúas.",
          )}
        </p>
      </header>
      <div className="design-creation-body">
        <label className="design-creation-brief">
          {tr("Your description", "Tu descripción")}
          <textarea
            rows={3}
            maxLength={32000}
            value={draft.request.text}
            disabled={s.busy}
            onChange={(e) => {
              const text = e.target.value;
              patchDraft({ request: { ...draft.request, text } });
              window.dispatchEvent(
                new CustomEvent("fenix:design-draft", { detail: { text } }),
              );
            }}
          />
        </label>
        <div className="design-creation-filters">
          <select
            aria-label={tr("Template category", "Categoría de plantilla")}
            value={draft.kind}
            disabled={s.busy}
            onChange={(e) => {
              patchDraft({ kind: e.target.value as Kind | "", selected: null });
              setQuery("");
            }}
          >
            <option value="">{tr("All designs", "Todos los diseños")}</option>
            {kinds.map((kind) => (
              <option key={kind} value={kind}>
                {labels[kind][workspace.language === "es" ? 1 : 0]}
              </option>
            ))}
          </select>
          <input
            type="search"
            aria-label={tr("Search templates", "Buscar plantillas")}
            placeholder={tr("Find a style…", "Busca un estilo…")}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </div>
        {loading ? (
          <Loading />
        ) : (
          <p className="design-creation-count" role="status">
            {shown.length} {tr("designs available", "diseños disponibles")}
          </p>
        )}
        {catalogError && (
          <div role="alert">
            <p>
              {tr(
                "Templates could not be loaded. You can retry or continue with your description.",
                "No se pudieron cargar las plantillas. Puedes reintentar o continuar con tu descripción.",
              )}
            </p>
            <button type="button" onClick={() => setRetry(retry + 1)}>
              {tr("Retry", "Reintentar")}
            </button>
          </div>
        )}
        <div
          className="design-creation-grid"
          aria-label={tr("Available templates", "Plantillas disponibles")}
        >
          {shown.map((v) => (
            <button
              type="button"
              className="design-creation-card"
              key={v.id}
              aria-pressed={draft.selected?.id === v.id}
              aria-label={`${tr("Select", "Seleccionar")} ${label(v)}`}
              disabled={s.busy}
              onClick={() => patchDraft({ selected: v })}
            >
              <Thumbnail v={v} />
              <span>
                <small>
                  {labels[v.kind][workspace.language === "es" ? 1 : 0]}
                </small>
                <strong>{label(v)}</strong>
                {draft.selected?.id === v.id && <b aria-hidden="true">✓</b>}
              </span>
            </button>
          ))}
        </div>
        {!loading && !catalogError && !shown.length && (
          <p>
            {tr(
              "No matching templates. Try another category or continue with your own description.",
              "No hay plantillas que coincidan. Prueba otra categoría o continúa con tu propia descripción.",
            )}
          </p>
        )}
      </div>
      <footer className="design-creation-footer">
        {s.error && <p role="alert">{s.error}</p>}
        {draft.selected && (
          <p>
            {tr("Selected:", "Seleccionada:")}{" "}
            <strong>{label(draft.selected)}</strong>
          </p>
        )}
        <div>
          <button
            type="button"
            disabled={s.busy || !draft.request.text.trim()}
            onClick={() => void generate({ mode: "custom" })}
          >
            {tr("Generate without a template", "Generar sin plantilla")}
          </button>
          <button
            type="button"
            className="primary"
            disabled={s.busy || !draft.selected || !draft.request.text.trim()}
            onClick={() => {
              if (draft.selected)
                void generate({
                  mode: "template",
                  template_id: draft.selected.id,
                });
            }}
          >
            {s.busy
              ? tr("Sending…", "Enviando…")
              : tr("Generate with this template", "Generar con esta plantilla")}
          </button>
        </div>
      </footer>
    </aside>
  );
}
