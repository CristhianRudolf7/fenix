import {
  openTarget,
  registerAdapter,
  hideWorkspace,
  updateTab,
  type Tab,
} from "../workspace/store";
import { Icon } from "../components/icons";
import { Loading } from "../components/loading";
import {
  useEffect,
  useState,
  useSyncExternalStore,
  Component,
  type ReactNode,
} from "react";
import { createRoot, type Root } from "react-dom/client";
import {
  api,
  assetUrl,
  download,
  kinds,
  labels,
  type Visual,
  type TemplatePage,
  type ArtifactPage,
  type Brand,
} from "./types";
import { Viewer } from "./viewers";
import "./design.css";

type State = {
  language: string;
  selected: Visual | null;
  isTemplate: boolean;
  artifacts: Visual[];
  error: string;
  busy: boolean;
  editing: boolean;
  offers: { id: string; templates: Visual[] }[];
};
let state: State = {
  language: "es",
  selected: null,
  isTemplate: false,
  artifacts: [],
  error: "",
  busy: false,
  editing: false,
  offers: [],
};
const listeners = new Set<() => void>();
const subscribe = (fn: () => void) => {
  listeners.add(fn);
  return () => {
    listeners.delete(fn);
  };
};
const update = (v: Partial<State>) => {
  state = { ...state, ...v };
  listeners.forEach((fn) => fn());
};
const useStateStore = () => useSyncExternalStore(subscribe, () => state);
const tr = (en: string, es: string) => (state.language === "es" ? es : en);
const title = (v: Visual) =>
  state.language === "es" ? v.title_es || v.title : v.title;
let pageRoot: Root | undefined;
let trayRoot: Root | undefined;
let panelRoot: Root | undefined;
let selectionSequence = 0;
let active = false;
let refreshed = false;
let polling = false;
let previousFocus: HTMLElement | null = null;
const fail = (e: unknown) =>
  update({ error: e instanceof Error ? e.message : String(e), busy: false });
function select(v: Visual, isTemplate = false) {
  openTarget({
    kind: "design",
    id: v.id,
    title: title(v),
    project_id: v.project_id,
    template: isTemplate,
  });
}
registerAdapter("design", {
  show: (tab) => {
    ensurePanel();
    void loadDesign(
      {
        id: tab.id!,
        title: tab.title || tab.id!,
        project_id: tab.project_id,
      } as Visual,
      !!tab.template,
      tab,
    );
  },
  hide: close,
});
async function loadDesign(v: Visual, isTemplate = false, tab?: Tab) {
  window.dispatchEvent(new Event("fenix:design-preview"));
  const seq = ++selectionSequence;
  previousFocus = document.activeElement as HTMLElement;
  update({
    selected: state.selected?.id === v.id ? state.selected : null,
    busy: true,
    error: "",
  });
  try {
    const full = await api<Visual>(
      `${isTemplate ? "/templates" : "/artifacts"}/${encodeURIComponent(v.id)}`,
    );
    if (seq === selectionSequence) {
      if (tab)
        updateTab(tab.key, { title: title(full), project_id: full.project_id });
      update({
        selected: {
          ...full,
          ...(isTemplate ? { project_id: v.project_id } : {}),
          preview_page: tab?.page,
        } as Visual,
        isTemplate,
        busy: false,
        editing: false,
      });
    }
  } catch (e) {
    if (seq === selectionSequence) fail(e);
  }
}
export function closeDesignPreview() {
  close();
}
function close() {
  selectionSequence++;
  update({ selected: null, error: "", busy: false, editing: false });
  previousFocus?.focus();
}
export async function refreshDesigns() {
  if (!active || polling) return;
  polling = true;
  try {
    const [result, offers] = await Promise.all([
      api<ArtifactPage>("/artifacts"),
      api<State["offers"]>("/offers"),
    ]);
    if (!active) return;
    update({ artifacts: result.designs, offers });
    // Keep an already-open view current when another client updates its revision.
    // New targets are opened by durable events, not by scanning history.
    const current = state.selected;
    const changed =
      current &&
      !state.isTemplate &&
      !state.busy &&
      result.designs.find(
        (d) =>
          d.id === current.id && (d.revision || 0) > (current.revision || 0),
      );
    if (changed) select(changed);
    refreshed = true;
  } catch (e) {
    fail(e);
  } finally {
    polling = false;
  }
}
function ensurePanel() {
  if (panelRoot) return;
  const node = document.createElement("div");
  node.id = "design-panel-root";
  document.body.append(node);
  panelRoot = createRoot(node);
  panelRoot.render(<Panel />);
}
export function stopDesignViews() {
  active = false;
  pageRoot?.unmount();
  trayRoot?.unmount();
  pageRoot = undefined;
  trayRoot = undefined;
}
export function renderDesign(language: string) {
  ensurePanel();
  active = true;
  update({ language });
  const node = document.getElementById("page")!;
  pageRoot = createRoot(node);
  pageRoot.render(<DesignPage />);
  void refreshDesigns();
  const template = new URLSearchParams(window.location.search).get("template");
  if (template) openTarget({ kind: "design", id: template, template: true });
}
export function renderChatDesigns(language: string) {
  ensurePanel();
  active = true;
  update({ language });
  const node = document.getElementById("design-results");
  if (node) {
    trayRoot = createRoot(node);
    trayRoot.render(<ChatResults />);
  }
  void refreshDesigns();
}
function ChatResults() {
  const s = useStateStore();
  return (
    <div
      className="chat-design-results"
      aria-label={tr("Visual results", "Resultados visuales")}
    >
      {s.offers.slice(0, 1).flatMap((o) =>
        o.templates.map((v, i) => (
          <button
            className="chat-template-choice"
            key={`${o.id}-${v.id}`}
            onClick={() => void select(v, true)}
          >
            <Thumbnail v={v} />
            {i + 1}. {title(v)}
          </button>
        )),
      )}
      {s.artifacts.slice(0, 8).map((v) => (
        <button
          key={v.id}
          onClick={() => void select(v)}
          title={`${labels[v.kind][s.language === "es" ? 1 : 0]} · ${v.project_id || tr("General", "General")}`}
        >
          <span aria-hidden="true">◇</span> {v.title}{" "}
          <small>v{v.revision}</small>
        </button>
      ))}
    </div>
  );
}
class PreviewBoundary extends Component<
  { children: ReactNode },
  { error: boolean }
> {
  state = { error: false };
  static getDerivedStateFromError() {
    return { error: true };
  }
  render() {
    return this.state.error ? (
      <p role="alert">
        Preview unavailable. Open the source to inspect it. / Vista no
        disponible; revisa el contenido.
      </p>
    ) : (
      this.props.children
    );
  }
}
function Panel() {
  const s = useStateStore();
  const v = s.selected;
  const isPdf = v?.kind === "document" && !!v.content?.asset_id;
  const [mode, setMode] = useState("preview");
  const [source, setSource] = useState("");
  const [device, setDevice] = useState("desktop");
  const [project, setProject] = useState("");
  const [projects, setProjects] = useState<{ id: string; name: string }[]>([]);
  const [revision, setRevision] = useState(1);
  const [latest, setLatest] = useState(1);
  useEffect(() => {
    document.documentElement.classList.toggle("design-preview-open", !!v);
    if (v) {
      setMode("preview");
      setProject(v.project_id || "");
      setSource(JSON.stringify(v.content, null, 2));
      setRevision(v.revision || 1);
      setLatest(v.revision || 1);
      document.getElementById("workspace-back")?.focus();
    }
  }, [v?.id]);
  useEffect(() => {
    if (v?.content) setSource(JSON.stringify(v.content, null, 2));
    if (v?.revision && v.revision > latest) {
      setLatest(v.revision);
      setRevision(v.revision);
    }
  }, [v]);
  useEffect(() => {
    if (v)
      void fetch("/api/projects")
        .then((r) => r.json())
        .then(setProjects)
        .catch(fail);
  }, [v?.id]);
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (!v) return;
      if (e.key === "Escape" && !document.fullscreenElement) hideWorkspace();
      if (
        !document.documentElement.classList.contains("workspace-open") &&
        e.key === "Tab" &&
        matchMedia("(max-width: 900px)").matches
      ) {
        const nodes = [
          ...document.querySelectorAll<HTMLElement>(
            '.design-panel button:not(:disabled), .design-panel input, .design-panel select, .design-panel textarea, .design-panel a[href], .design-panel [tabindex="0"]',
          ),
        ].filter((n) => n.getClientRects().length);
        const first = nodes[0],
          last = nodes.at(-1);
        if (e.shiftKey && document.activeElement === first) {
          e.preventDefault();
          last?.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first?.focus();
        }
      }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [v]);
  if (!v)
    return s.busy || s.error ? (
      <aside
        className="design-panel"
        aria-label={tr("Design preview", "Vista del diseño")}
      >
        <div className="design-preview-content">
          {s.error ? <p role="alert">{s.error}</p> : <Loading />}
        </div>
      </aside>
    ) : null;
  const run = async (fn: () => Promise<void>) => {
    const seq = selectionSequence;
    update({ busy: true, error: "" });
    try {
      await fn();
      if (seq === selectionSequence) update({ busy: false });
    } catch (e) {
      if (seq === selectionSequence) fail(e);
    }
  };
  const commit = async (content: Visual["content"]) => {
    const seq = selectionSequence;
    const saved = await api<Visual>(
      `/artifacts/${v.id}`,
      {
        title: v.title,
        kind: v.kind,
        content,
        project_id: v.project_id,
        template_id: v.template_id,
        expected_revision: latest,
      },
      "PUT",
    );
    if (seq !== selectionSequence) return;
    setLatest(saved.revision!);
    setRevision(saved.revision!);
    update({ selected: saved });
    await refreshDesigns();
  };
  const useTemplate = () =>
    run(async () => {
      const seq = selectionSequence;
      const saved = await api<Visual>(`/templates/${v.id}/use`, {
        project_id: project || null,
      });
      if (seq !== selectionSequence) return;
      select(saved, false);
      await refreshDesigns();
    });
  const ask = () => {
    const text = tr(
      `Work on design “${v.title}” (design_id: ${v.id}, project_id: ${v.project_id || "general"}, revision: ${revision}, latest_revision: ${latest}): `,
      `Trabaja sobre el diseño «${v.title}» (design_id: ${v.id}, project_id: ${v.project_id || "general"}, revisión: ${revision}, última_revisión: ${latest}): `,
    );
    window.dispatchEvent(
      new CustomEvent("fenix:design-chat", { detail: { text } }),
    );
  };
  function exportResult() {
    const c = v!.content!;
    const base = v!.title.replace(/[^\p{L}\p{N}_-]/gu, "-");
    if (c.asset_id) {
      const a = document.createElement("a");
      a.href = assetUrl(c.asset_id);
      a.download = c.filename || base;
      a.click();
      return;
    }
    if (v!.kind === "web") download(base + ".html", c.html!, "text/html");
    else if (v!.kind === "diagram")
      download(
        base + ".diagram.json",
        JSON.stringify({ title: v!.title, kind: v!.kind, content: c }, null, 2),
        "application/json",
      );
    else if (v!.kind === "image")
      download(base + ".svg", c.svg!, "image/svg+xml");
    else if (v!.kind === "document") download(base + ".txt", c.text!);
    else if (v!.kind === "spreadsheet") {
      const cell = (x: unknown) =>
        '"' + String(x ?? "").replaceAll('"', '""') + '"';
      download(
        base + ".csv",
        [c.columns!, ...c.rows!].map((r) => r.map(cell).join(",")).join("\r\n"),
        "text/csv",
      );
    } else if (c.sample) {
      const a = document.createElement("a");
      a.href = `/api/design/samples/${c.sample}`;
      a.download = base + ".webm";
      a.click();
    } else
      download(
        base + ".json",
        JSON.stringify({ title: v!.title, kind: v!.kind, content: c }, null, 2),
        "application/json",
      );
  }
  return (
    <aside
      className={`design-panel${isPdf ? " design-panel-pdf" : ""}`}
      role="dialog"
      aria-label={tr("Design preview", "Vista del diseño")}
    >
      {!isPdf && (
        <div className="design-toolbar">
          <button
            className="subtle icon-button"
            onClick={exportResult}
            aria-label={tr("Download", "Descargar")}
            title={tr("Download", "Descargar")}
          >
            <Icon name="download" />
          </button>
          {v.kind === "web" && (
            <select
              aria-label={tr("Preview size", "Tamaño de vista")}
              value={device}
              onChange={(e) => setDevice(e.target.value)}
            >
              <option value="desktop">Desktop</option>
              <option value="mobile">Mobile · 390px</option>
            </select>
          )}
        </div>
      )}
      {s.error && (
        <p role="alert" className="design-error">
          {s.error}
        </p>
      )}
      {v.kind === "web" && (
        <p className="design-note">
          {tr(
            "Isolated visual preview. Scripts, forms and external requests are disabled.",
            "Vista visual aislada. Scripts, formularios y solicitudes externas desactivados.",
          )}
        </p>
      )}
      <div
        className={`design-preview-scroll ${device === "mobile" && v.kind === "web" ? "device-mobile" : ""}`}
      >
        <div className="design-preview-content">
          <PreviewBoundary
            key={v.kind === "diagram" ? v.id : `${v.id}-${v.revision}`}
          >
            <Viewer
              key={v.kind === "diagram" ? v.id : `${v.id}-${v.revision}`}
              visual={v}
              initialPage={
                (v as Visual & { preview_page?: number }).preview_page
              }
              language={s.language}
            />
          </PreviewBoundary>
        </div>
      </div>
      {s.isTemplate && (
        <footer className="design-panel-foot">
          <>
            <select
              aria-label={tr("Destination project", "Proyecto de destino")}
              value={project}
              onChange={(e) => setProject(e.target.value)}
            >
              <option value="">
                {tr("General conversation", "Conversación general")}
              </option>
              {projects.map((p) => (
                <option value={p.id} key={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
            <button
              className="primary"
              disabled={s.busy}
              onClick={() => void useTemplate()}
            >
              {tr("Use this template", "Usar esta plantilla")}
            </button>
          </>
        </footer>
      )}
    </aside>
  );
}
export function Thumbnail({ v }: { v: Visual }) {
  const [missing, setMissing] = useState(false);
  return !missing && (!v.revision || v.revision === 1) ? (
    <img
      className="design-thumbnail"
      loading="lazy"
      src={`/assets/design-thumbnails/${v.template_id || v.id}.png`}
      alt=""
      onError={() => setMissing(true)}
    />
  ) : (
    <div className={`design-thumbnail thumb-${v.kind}`} aria-hidden="true">
      <svg viewBox="0 0 320 170">
        <rect width="320" height="170" fill="#f4f4f0" />
        {v.kind === "diagram" ? (
          <>
            <path d="M70 85H250M160 45V125" stroke="#788e7c" strokeWidth="3" />
            {[
              [40, 65],
              [135, 20],
              [135, 115],
              [230, 65],
            ].map(([x, y], i) => (
              <rect
                key={i}
                x={x}
                y={y}
                width="50"
                height="30"
                rx="5"
                fill={i % 2 ? "#b7cab7" : "#294e3a"}
              />
            ))}
          </>
        ) : v.kind === "chart" ? (
          <>
            {[60, 95, 75, 120, 100].map((h, i) => (
              <rect
                key={i}
                x={45 + i * 48}
                y={145 - h}
                width="26"
                height={h}
                rx="3"
                fill={i % 2 ? "#b7cab7" : "#294e3a"}
              />
            ))}
          </>
        ) : v.kind === "spreadsheet" ? (
          <>
            {Array.from({ length: 5 }, (_, i) => (
              <path
                key={i}
                d={`M30 ${35 + i * 24}H290 M${30 + i * 65} 35V140`}
                stroke="#b3bbae"
              />
            ))}
            <rect x="30" y="20" width="260" height="22" fill="#294e3a" />
          </>
        ) : (
          <>
            <rect x="28" y="25" width="264" height="120" rx="6" fill="white" />
            <rect x="42" y="40" width="90" height="7" rx="3" fill="#294e3a" />
            <rect x="42" y="65" width="145" height="12" rx="3" fill="#889f89" />
            <rect x="42" y="86" width="110" height="5" fill="#ddd" />
            <rect x="42" y="98" width="125" height="5" fill="#ddd" />
            <rect x="206" y="60" width="64" height="63" rx="4" fill="#dce5d6" />
            {v.kind === "video" && (
              <path d="M229 75L251 91L229 106Z" fill="#294e3a" />
            )}
          </>
        )}
      </svg>
    </div>
  );
}
function DesignPage() {
  const s = useStateStore();
  const [tab, setTab] = useState("templates");
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState("");
  const [rows, setRows] = useState<Visual[]>([]);
  const [next, setNext] = useState<number | null>(null);
  const [favorites, setFavorites] = useState<string[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [project, setProject] = useState("");
  const [projects, setProjects] = useState<{ id: string; name: string }[]>([]);
  useEffect(() => {
    void api<{ id: string }[]>("/favorites")
      .then((v) => setFavorites(v.map((x) => x.id)))
      .catch((e) => setError(String(e)));
    void fetch("/api/projects")
      .then((r) => r.json())
      .then(setProjects)
      .catch((e) => setError(String(e)));
  }, []);
  useEffect(() => {
    const controller = new AbortController();
    setRows([]);
    setLoading(true);
    setError("");
    if (tab === "brand") {
      setLoading(false);
      return;
    }
    const timer = setTimeout(() => {
      void (async () => {
        if (tab === "mine") {
          const p = await api<ArtifactPage>(
            `/artifacts${project ? `?project_id=${project}` : ""}`,
            undefined,
            "GET",
            controller.signal,
          );
          setRows(p.designs);
          setNext(p.next_offset);
        } else if (tab === "favorites") {
          const p = await Promise.all(
            favorites.map((id) =>
              api<Visual>(
                `/templates/${id}`,
                undefined,
                "GET",
                controller.signal,
              ),
            ),
          );
          if (!controller.signal.aborted) {
            setRows(p);
            setNext(null);
          }
        } else {
          const p = await api<TemplatePage>(
            `/templates?query=${encodeURIComponent(query)}${kind ? `&kind=${kind}` : ""}`,
            undefined,
            "GET",
            controller.signal,
          );
          setRows(p.templates);
          setNext(p.next_offset);
        }
        if (!controller.signal.aborted) setLoading(false);
      })().catch((e) => {
        if (!controller.signal.aborted) {
          setError(String(e));
          setLoading(false);
        }
      });
    }, 150);
    return () => {
      clearTimeout(timer);
      controller.abort();
    };
  }, [tab, query, kind, favorites, project]);
  async function more() {
    setLoading(true);
    try {
      if (tab === "mine") {
        const p = await api<ArtifactPage>(
          `/artifacts?offset=${next}${project ? `&project_id=${project}` : ""}`,
        );
        setRows([...rows, ...p.designs]);
        setNext(p.next_offset);
      } else {
        const p = await api<TemplatePage>(
          `/templates?offset=${next}&query=${encodeURIComponent(query)}${kind ? `&kind=${kind}` : ""}`,
        );
        setRows([...rows, ...p.templates]);
        setNext(p.next_offset);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }
  async function favorite(v: Visual) {
    try {
      const enabled = !favorites.includes(v.id);
      await api(`/favorites/${v.id}`, { enabled }, "PUT");
      setFavorites(
        enabled ? [...favorites, v.id] : favorites.filter((id) => id !== v.id),
      );
    } catch (e) {
      setError(String(e));
    }
  }
  async function upload(file: File) {
    setLoading(true);
    setError("");
    try {
      if (file.size > 64 * 1024 * 1024)
        throw new Error(
          tr("Maximum file size: 64 MiB", "Tamaño máximo: 64 MiB"),
        );
      const data = await new Promise<string>((resolve, reject) => {
        const r = new FileReader();
        r.onload = () => resolve(String(r.result).split(",")[1]);
        r.onerror = () => reject(r.error);
        r.readAsDataURL(file);
      });
      const saved = await api<Visual>("/import", {
        filename: file.name,
        data,
        project_id: project || null,
      });
      await refreshDesigns();
      await select(saved);
      setTab("mine");
      setRows([saved, ...rows.filter((r) => r.revision)]);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }
  const visible = rows.filter(
    (v) =>
      (tab === "templates" ||
        !query ||
        `${v.title} ${v.title_es || ""}`
          .toLowerCase()
          .includes(query.toLowerCase())) &&
      (!kind || v.kind === kind),
  );
  return (
    <section className="design-page">
      <div className="design-heading">
        <div>
          <small>FÉNIX DESIGN</small>
          <h1>{tr("Give your ideas a shape.", "Dale forma a tus ideas.")}</h1>
          <p>
            {tr(
              "Explore 72 original starting points. Create, compare and continue in your conversation.",
              "Explora 72 puntos de partida propios. Crea, compara y continúa en tu conversación.",
            )}
          </p>
        </div>
        <label className="design-upload">
          ＋ {tr("Import file", "Importar archivo")}
          <input
            type="file"
            disabled={loading}
            accept=".html,.htm,.mmd,.mermaid,.md,.txt,.json,.csv,.docx,.xlsx,.pptx,.pdf,.svg,.png,.jpg,.jpeg,.webp,.gif,.mp4,.webm"
            onChange={(e) => {
              const f = e.target.files?.[0];
              if (f) void upload(f);
              e.target.value = "";
            }}
          />
        </label>
      </div>
      <nav className="design-tabs" aria-label="Design">
        {[
          ["templates", "Templates", "Plantillas"],
          ["favorites", "Favorites", "Favoritos"],
          ["mine", "My designs", "Mis diseños"],
          ["brand", "Brand identity", "Identidad visual"],
        ].map(([id, en, es]) => (
          <button
            key={id}
            aria-current={tab === id ? "page" : undefined}
            onClick={() => {
              setTab(id);
              setQuery("");
              setKind("");
            }}
          >
            {tr(en, es)}
            {id === "favorites" && favorites.length > 0
              ? ` (${favorites.length})`
              : ""}
          </button>
        ))}
      </nav>
      {tab === "brand" ? (
        <BrandEditor />
      ) : (
        <>
          <div className="design-filters">
            <input
              aria-label={tr("Search designs", "Buscar diseños")}
              placeholder={tr(
                "Search a purpose, style or format…",
                "Busca un propósito, estilo o formato…",
              )}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
            <select
              aria-label={tr("Design category", "Categoría de diseño")}
              value={kind}
              onChange={(e) => setKind(e.target.value)}
            >
              <option value="">
                {tr("All formats", "Todos los formatos")}
              </option>
              {kinds.map((k) => (
                <option value={k} key={k}>
                  {labels[k][s.language === "es" ? 1 : 0]}
                </option>
              ))}
            </select>
            <select
              aria-label={tr(
                "Project for imports",
                "Proyecto para importaciones",
              )}
              value={project}
              onChange={(e) => setProject(e.target.value)}
            >
              <option value="">
                {tr("General / all projects", "General / todos los proyectos")}
              </option>
              {projects.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </div>
          {error && (
            <p role="alert" className="design-error">
              {error}
            </p>
          )}
          <div className="design-grid" aria-busy={loading}>
            {visible.map((v) => (
              <article className="design-card" key={v.id}>
                <button
                  className="design-card-open"
                  onClick={() => void select(v, tab !== "mine")}
                >
                  <Thumbnail v={v} />
                  <div className="design-card-title">
                    <small>{labels[v.kind][s.language === "es" ? 1 : 0]}</small>
                    <h2>{title(v)}</h2>
                    <span>
                      {v.revision
                        ? `v${v.revision} · ${projects.find((p) => p.id === v.project_id)?.name || tr("General", "General")}`
                        : tr(
                            "Original · Editable example",
                            "Original · Ejemplo editable",
                          )}
                    </span>
                  </div>
                </button>
                {tab !== "mine" && (
                  <button
                    className="design-star"
                    aria-label={`${tr("Favorite", "Favorito")}: ${title(v)}`}
                    aria-pressed={favorites.includes(v.id)}
                    onClick={() => void favorite(v)}
                  >
                    {favorites.includes(v.id) ? "★" : "☆"}
                  </button>
                )}
              </article>
            ))}
          </div>
          {!loading && !visible.length && (
            <div className="design-empty">
              <h2>
                {tr(
                  "A little space for your next idea.",
                  "Un espacio para tu próxima idea.",
                )}
              </h2>
              <p>
                {tab === "mine"
                  ? tr(
                      "Choose a template, import a file or ask the agent to create a visual result.",
                      "Elige una plantilla, importa un archivo o pide al agente que cree un resultado visual.",
                    )
                  : tr(
                      "Try another search or save a template as a favorite.",
                      "Prueba otra búsqueda o guarda una plantilla como favorita.",
                    )}
              </p>
            </div>
          )}
          {loading && (
            <Loading
              label={tr("Preparing your designs…", "Preparando tus diseños…")}
            />
          )}
          {next !== null && (
            <button
              className="design-load"
              disabled={loading}
              onClick={() => void more()}
            >
              {tr("Load more", "Ver más")}
            </button>
          )}
        </>
      )}
    </section>
  );
}
function BrandEditor() {
  const [brand, setBrand] = useState<Brand | null>(null);
  const [status, setStatus] = useState("");
  useEffect(() => {
    void api<Brand>("/brand")
      .then(setBrand)
      .catch((e) => setStatus(String(e)));
  }, []);
  if (!brand) return status ? <p role="alert">{status}</p> : <Loading />;
  return (
    <form
      className="brand-editor"
      onSubmit={(e) => {
        e.preventDefault();
        void api<Brand>("/brand", brand, "PUT")
          .then((v) => {
            setBrand(v);
            setStatus(
              tr(
                "Identity saved. Apply it from a saved design.",
                "Identidad guardada. Aplícala desde un diseño guardado.",
              ),
            );
          })
          .catch((e) => setStatus(String(e)));
      }}
    >
      <div>
        <h2>
          {tr("A consistent visual voice.", "Una identidad visual coherente.")}
        </h2>
        <p>
          {tr(
            "Save your colors and typography. Apply them explicitly to web templates and presentations; existing designs keep their own versions.",
            "Guarda colores y tipografía. Aplícalos a plantillas web y presentaciones; los diseños existentes conservan sus propias versiones.",
          )}
        </p>
        <label>
          {tr("Logo (general workspace)", "Logo (espacio general)")}
          <input
            type="file"
            accept="image/png,image/jpeg,image/webp,image/svg+xml"
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (!file) return;
              if (file.size > 4 * 1024 * 1024) {
                setStatus("Logo: maximum 4 MiB");
                return;
              }
              const reader = new FileReader();
              reader.onload = () => {
                void api<Visual>("/import", {
                  filename: file.name,
                  data: String(reader.result).split(",")[1],
                  project_id: null,
                })
                  .then((v) => {
                    setBrand({ ...brand, logo_asset_id: v.content!.asset_id });
                    setStatus(
                      tr(
                        "Logo ready. Save your identity.",
                        "Logo listo. Guarda tu identidad.",
                      ),
                    );
                  })
                  .catch((e) => setStatus(String(e)));
              };
              reader.onerror = () =>
                setStatus("Unable to read logo / No se pudo leer el logo");
              reader.readAsDataURL(file);
            }}
          />
        </label>
        {brand.logo_asset_id && (
          <button
            type="button"
            onClick={() => setBrand({ ...brand, logo_asset_id: null })}
          >
            {tr("Remove logo from identity", "Retirar logo de la identidad")}
          </button>
        )}
        <label>
          {tr("Brand name", "Nombre de marca")}
          <input
            maxLength={100}
            value={brand.name}
            onChange={(e) => setBrand({ ...brand, name: e.target.value })}
          />
        </label>
        {(["accent", "background", "text"] as const).map((key) => (
          <label key={key}>
            {tr(
              key,
              { accent: "Acento", background: "Fondo", text: "Texto" }[key],
            )}
            <input
              type="color"
              value={brand[key]}
              onChange={(e) => setBrand({ ...brand, [key]: e.target.value })}
            />
          </label>
        ))}
        <label>
          {tr("Typography", "Tipografía")}
          <select
            value={brand.font}
            onChange={(e) => setBrand({ ...brand, font: e.target.value })}
          >
            <option value="sans-serif">Sans serif</option>
            <option value="serif">Serif</option>
            <option value="monospace">Monospace</option>
          </select>
        </label>
        <button className="primary">
          {tr("Save identity", "Guardar identidad")}
        </button>
        <p role="status">{status}</p>
      </div>
      <div
        className="brand-sample"
        style={{
          background: brand.background,
          color: brand.text,
          fontFamily: brand.font,
        }}
      >
        {brand.logo_asset_id && (
          <img
            className="brand-logo"
            alt={brand.name || "Logo"}
            src={assetUrl(brand.logo_asset_id)}
          />
        )}
        <small>{brand.name || "YOUR BRAND"}</small>
        <h2>{tr("Ideas deserve clarity.", "Las ideas merecen claridad.")}</h2>
        <p>
          {tr(
            "A preview of your selected colors and typography.",
            "Una muestra de tus colores y tipografía.",
          )}
        </p>
        <span style={{ background: brand.accent, color: "white" }}>
          {tr("Get started", "Comenzar")}
        </span>
      </div>
    </form>
  );
}
