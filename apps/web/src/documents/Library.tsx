import {
  openTarget,
  registerAdapter,
  hideWorkspace,
  updateTab,
  type Tab,
} from "../workspace/store";
import {
  lazy,
  Suspense,
  useEffect,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import { createRoot, type Root } from "react-dom/client";
import { Markdown } from "../components/markdown";
import { Loading } from "../components/loading";
import { Icon } from "../components/icons";
import { closeDesignPreview } from "../design/app";
import "./library.css";
const PdfViewer = lazy(() => import("../design/pdf-viewer"));
type Doc = {
  id: string;
  project_id: string | null;
  title: string;
  filename: string;
  format: string;
  collection: string;
  tags: string[];
  enabled: boolean;
  revision: number;
  status: string;
  embedding_status: string;
  warnings: string[];
  error: string | null;
  bytes: number;
  updated_at: string;
  sha256: string;
};
type Chunk = {
  id: string;
  text: string;
  ordinal: number;
  location: {
    page?: number;
    section?: string;
    line_start?: number;
    line_end?: number;
    cell?: string;
    part?: string;
    polygon?: number[][];
  };
};
type Detail = {
  document: Doc;
  chunks: Chunk[];
  revisions: Doc[];
  next_offset: number | null;
};
type Settings = {
  model: string;
  enabled: boolean;
  configured: boolean;
  execution_available: boolean;
};
type Hit = Chunk & {
  document_id: string;
  revision: number;
  title: string;
  citation: string;
  branches: string[];
};
type Project = { id: string; name: string };
let language = "es";
const tr = (en: string, es: string) => (language === "es" ? es : en);
const message = (e: unknown) => (e instanceof Error ? e.message : String(e));
export async function libraryApi<T>(
  path: string,
  method = "GET",
  body?: unknown,
): Promise<T> {
  const r = await fetch(`/api/documents${path}`, {
    method,
    headers: { "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (r.status === 401) {
    location.reload();
    throw new Error("Session expired");
  }
  const data = await r.json();
  if (!r.ok) throw new Error(data.error || `HTTP ${r.status}`);
  return data as T;
}
const accept = ".pdf,.docx,.xlsx,.pptx,.txt,.md,.csv,.json,.png,.jpg,.jpeg";
async function upload(
  file: File,
  project: string | null,
  collection = "",
  previous?: Doc,
): Promise<Doc> {
  if (!file.size || file.size > 64 * 1024 * 1024)
    throw new Error(
      tr("Choose a file up to 64 MiB", "Elige un archivo de hasta 64 MiB"),
    );
  const data = await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(new Error("File unavailable"));
    reader.onload = () => resolve(String(reader.result).split(",")[1]);
    reader.readAsDataURL(file);
  });
  return libraryApi<Doc>("/import", "POST", {
    filename: file.name,
    data,
    project_id: project,
    collection,
    ...(previous
      ? {
          id: previous.id,
          expected_revision: previous.revision,
          title: previous.title,
          tags: previous.tags,
        }
      : {}),
  });
}
let sources: Doc[] = [];
const sourceListeners = new Set<() => void>();
const sourceSubscribe = (fn: () => void) => {
  sourceListeners.add(fn);
  return () => {
    sourceListeners.delete(fn);
  };
};
function setSources(value: Doc[]) {
  sources = value;
  sourceListeners.forEach((fn) => fn());
}
export function selectedDocumentIds() {
  return sources.map((d) => d.id);
}
export function clearDocumentSelection() {
  setSources([]);
}
function choose(doc: Doc) {
  if (sources.some((d) => d.id === doc.id)) {
    setSources(sources.filter((d) => d.id !== doc.id));
    return;
  }
  if (sources.length && sources[0].project_id !== doc.project_id)
    throw new Error(
      tr(
        "Select documents from the same project",
        "Selecciona documentos del mismo proyecto",
      ),
    );
  if (sources.length >= 20)
    throw new Error(
      tr("Select up to 20 documents", "Selecciona hasta 20 documentos"),
    );
  setSources([...sources, doc]);
}
function status(doc: Doc) {
  const names: Record<string, string> = {
    queued: tr("Queued", "En cola"),
    processing: tr("Reading", "Leyendo"),
    ready: tr("Ready", "Disponible"),
    partial: tr("Review extraction", "Revisar extracción"),
    failed: tr("Needs attention", "Requiere atención"),
  };
  return names[doc.status] || doc.status;
}
function Badge({ doc }: { doc: Doc }) {
  return (
    <span className={`library-badge ${doc.status}`}>
      <i />
      {status(doc)}
    </span>
  );
}
function locationLabel(c: Chunk) {
  return [
    c.location.page && `${tr("Page", "Página")} ${c.location.page}`,
    c.location.section,
    c.location.part,
    c.location.cell,
    c.location.line_start &&
      `${tr("Lines", "Líneas")} ${c.location.line_start}–${c.location.line_end}`,
  ]
    .filter(Boolean)
    .join(" · ");
}
let pageRoot: Root | undefined,
  chatRoot: Root | undefined,
  panelRoot: Root | undefined;
let openSource: {
  id: string;
  revision?: number;
  chunk?: string;
  page?: number;
} | null = null;
const panelListeners = new Set<() => void>();
const panelSubscribe = (fn: () => void) => {
  panelListeners.add(fn);
  return () => {
    panelListeners.delete(fn);
  };
};
function closeSource() {
  if (!openSource) return;
  openSource = null;
  document.documentElement.classList.remove("library-preview-open");
  panelListeners.forEach((fn) => fn());
}
export function openDocument(id: string, revision?: number, chunk?: string) {
  openTarget({
    kind: "library",
    id,
    revision,
    chunk,
    title: tr("Document", "Documento"),
  });
}
registerAdapter("library", {
  show: (tab) => showDocument(tab),
  hide: closeSource,
});
function showDocument(tab: Tab) {
  const { id, revision, chunk, page } = tab;
  closeDesignPreview();
  if (!panelRoot) {
    const node = document.createElement("div");
    document.body.append(node);
    panelRoot = createRoot(node);
    panelRoot.render(<SourcePanel />);
  }
  openSource = { id: id!, revision, chunk, page };
  document.documentElement.classList.add("library-preview-open");
  panelListeners.forEach((fn) => fn());
}
window.addEventListener("fenix:design-preview", closeSource);
document.addEventListener("click", (event) => {
  const a = (event.target as Element).closest?.("a[href]");
  if (!a) return;
  const url = new URL(a.getAttribute("href")!, location.href);
  if (
    url.origin !== location.origin ||
    url.pathname !== "/documents" ||
    !url.searchParams.get("document")
  )
    return;
  event.preventDefault();
  openDocument(
    url.searchParams.get("document")!,
    Number(url.searchParams.get("revision")) || undefined,
    url.searchParams.get("chunk") || undefined,
  );
});
export function stopLibrary() {
  pageRoot?.unmount();
  chatRoot?.unmount();
  pageRoot = undefined;
  chatRoot = undefined;
}
export function renderLibrary(lang: string) {
  language = lang;
  const page = document.getElementById("page")!;
  page.replaceChildren();
  pageRoot = createRoot(page);
  pageRoot.render(<Library />);
  const q = new URLSearchParams(location.search);
  if (q.get("document"))
    openDocument(
      q.get("document")!,
      Number(q.get("revision")) || undefined,
      q.get("chunk") || undefined,
    );
}
export function renderChatLibrary(lang: string) {
  language = lang;
  const node = document.createElement("div");
  node.className = "library-chat-root";
  document.getElementById("chat-form")?.prepend(node);
  chatRoot = createRoot(node);
  chatRoot.render(<ChatSources />);
}
function ChatSources() {
  const selected = useSyncExternalStore(sourceSubscribe, () => sources);
  const [expanded, setExpanded] = useState(false),
    [docs, setDocs] = useState<Doc[]>([]),
    [project, setProject] = useState(selected[0]?.project_id || ""),
    [projects, setProjects] = useState<Project[]>([]),
    [error, setError] = useState(""),
    [busy, setBusy] = useState(false);
  useEffect(() => {
    if (!expanded) return;
    let active = true;
    void Promise.all([
      libraryApi<{ documents: Doc[] }>(
        project ? `?project_id=${encodeURIComponent(project)}` : "",
      ),
      fetch("/api/projects").then((r) => r.json()),
    ])
      .then(([r, p]) => {
        if (active) {
          setDocs(r.documents.filter((d) => d.enabled));
          setProjects(p);
        }
      })
      .catch((e) => {
        if (active) setError(message(e));
      });
    return () => {
      active = false;
    };
  }, [expanded, project]);
  return (
    <div className="library-chat-sources">
      <div className="library-source-chips">
        <button
          type="button"
          className="subtle"
          onClick={() => setExpanded(!expanded)}
          aria-expanded={expanded}
        >
          <Icon name="documents" />
          {tr("Documents", "Documentos")}
        </button>
        {selected.map((d) => (
          <span className="library-chip" key={d.id}>
            <button
              type="button"
              onClick={() => openDocument(d.id, d.revision)}
            >
              {d.title}
            </button>
            <button
              type="button"
              aria-label={`${tr("Remove", "Quitar")} ${d.title}`}
              onClick={() => choose(d)}
            >
              ×
            </button>
          </span>
        ))}
      </div>
      {expanded && (
        <div className="library-picker">
          <label>
            {tr("Project", "Proyecto")}
            <select
              value={project}
              onChange={(e) => {
                setProject(e.target.value);
                setError("");
              }}
            >
              <option value="">{tr("General", "General")}</option>
              {projects.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </label>
          <label className="design-upload">
            {busy
              ? tr("Uploading…", "Subiendo…")
              : tr("Attach files", "Adjuntar archivos")}
            <input
              aria-label={tr("Attach documents", "Adjuntar documentos")}
              type="file"
              multiple
              accept={accept}
              disabled={busy}
              onChange={async (e) => {
                const files = Array.from(e.target.files || []);
                e.target.value = "";
                setBusy(true);
                try {
                  for (const f of files) {
                    const d = await upload(f, project || null);
                    choose(d);
                    setDocs((old) => [d, ...old]);
                  }
                } catch (e) {
                  setError(message(e));
                } finally {
                  setBusy(false);
                }
              }}
            />
          </label>
          <div className="library-picker-list">
            {docs.map((d) => (
              <label key={d.id}>
                <input
                  type="checkbox"
                  checked={selected.some((x) => x.id === d.id)}
                  onChange={() => {
                    try {
                      choose(d);
                      setError("");
                    } catch (e) {
                      setError(message(e));
                    }
                  }}
                />
                <span>{d.title}</span>
                <Badge doc={d} />
              </label>
            ))}
            {!docs.length && (
              <p className="muted">
                {tr(
                  "Upload files or choose them from your library.",
                  "Sube archivos o selecciónalos desde tu biblioteca.",
                )}
              </p>
            )}
          </div>
        </div>
      )}
      {error && (
        <p role="alert" className="library-error">
          {error}
        </p>
      )}
    </div>
  );
}
function Library() {
  const [docs, setDocs] = useState<Doc[]>([]),
    [projects, setProjects] = useState<Project[]>([]),
    [project, setProject] = useState(""),
    [collection, setCollection] = useState(""),
    [query, setQuery] = useState(""),
    [hits, setHits] = useState<Hit[] | null>(null),
    [error, setError] = useState(""),
    [busy, setBusy] = useState(false),
    [settings, setSettings] = useState<Settings | null>(null),
    [next, setNext] = useState<number | null>(null),
    [format, setFormat] = useState(""),
    [editing, setEditing] = useState<Doc | null>(null),
    [name, setName] = useState(""),
    [tags, setTags] = useState(""),
    [newCollection, setNewCollection] = useState(""),
    [key, setKey] = useState("");
  const alive = useRef(true);
  const scope = useRef(project);
  scope.current = project;
  const refreshGeneration = useRef(0);
  async function refresh(offset = 0, polling = false) {
    const generation = ++refreshGeneration.current;
    const p = new URLSearchParams({ offset: String(offset) });
    if (project !== "*") {
      if (project) p.set("project_id", project);
    } else p.set("all", "true");
    const r = await libraryApi<{
      documents: Doc[];
      next_offset: number | null;
    }>(`?${p}`);
    if (
      alive.current &&
      scope.current === project &&
      generation === refreshGeneration.current
    ) {
      setDocs((old) => {
        if (offset)
          return [
            ...old,
            ...r.documents.filter((d) => !old.some((o) => o.id === d.id)),
          ];
        if (polling && old.length > 50)
          return old.map((d) => r.documents.find((n) => n.id === d.id) || d);
        return r.documents;
      });
      if (!polling || docs.length <= 50) setNext(r.next_offset);
    }
  }
  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
    };
  }, []);
  useEffect(() => {
    let active = true;
    void fetch("/api/projects")
      .then((r) => r.json())
      .then((v) => {
        if (active) setProjects(v);
      })
      .catch((e) => setError(message(e)));
    void libraryApi<Settings>("/settings")
      .then((v) => {
        if (active) setSettings(v);
      })
      .catch((e) => setError(message(e)));
    return () => {
      active = false;
    };
  }, []);
  useEffect(() => {
    setHits(null);
    setError("");
    void refresh().catch((e) => setError(message(e)));
  }, [project]);
  useEffect(() => {
    if (
      !docs.some(
        (d) =>
          ["queued", "processing"].includes(d.status) ||
          (d.embedding_status === "pending" &&
            settings?.enabled &&
            settings?.configured),
      )
    )
      return;
    const timer = setInterval(() => {
      if (document.visibilityState === "visible")
        void refresh(0, true).catch((e) => setError(message(e)));
    }, 3500);
    return () => clearInterval(timer);
  }, [docs, project, settings]);
  const visible = docs.filter(
    (d) =>
      (!collection || d.collection === collection) &&
      (!format || d.format === format),
  );
  async function action(fn: () => Promise<void>) {
    setBusy(true);
    setError("");
    try {
      await fn();
    } catch (e) {
      setError(message(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <main className="library-page">
      <header className="library-heading">
        <div>
          <div className="eyebrow">
            {tr("YOUR SOURCES, CONNECTED", "TUS FUENTES, CONECTADAS")}
          </div>
          <h1>{tr("Document library", "Biblioteca de documentos")}</h1>
          <p>
            {tr(
              "Keep your sources together. Ask, compare and follow every answer back to its evidence.",
              "Reúne tus fuentes. Pregunta, compara y encuentra la evidencia de cada respuesta.",
            )}
          </p>
        </div>
        <label className="design-upload primary">
          <Icon name="documents" />
          {busy
            ? tr("Working…", "Procesando…")
            : tr("Upload documents", "Subir documentos")}
          <input
            type="file"
            multiple
            accept={accept}
            aria-label={tr("Upload documents", "Subir documentos")}
            disabled={busy || project === "*"}
            onChange={(e) => {
              const files = Array.from(e.target.files || []);
              e.target.value = "";
              void action(async () => {
                for (const f of files)
                  await upload(f, project || null, collection);
                await refresh();
              });
            }}
          />
        </label>
      </header>
      <div className="library-toolbar">
        <label>
          {tr("Project", "Proyecto")}
          <select
            disabled={busy}
            aria-label={tr("Library project", "Proyecto de biblioteca")}
            value={project}
            onChange={(e) => setProject(e.target.value)}
          >
            <option value="">General</option>
            <option value="*">
              {tr("All projects · browse", "Todos los proyectos · explorar")}
            </option>
            {projects.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>
        </label>
        <label>
          {tr("Collection", "Colección")}
          <select
            value={collection}
            onChange={(e) => setCollection(e.target.value)}
          >
            <option value="">
              {tr("All collections", "Todas las colecciones")}
            </option>
            {[...new Set(docs.map((d) => d.collection).filter(Boolean))].map(
              (c) => (
                <option key={c}>{c}</option>
              ),
            )}
          </select>
        </label>
        <label>
          {tr("Format", "Formato")}
          <select value={format} onChange={(e) => setFormat(e.target.value)}>
            <option value="">{tr("All formats", "Todos los formatos")}</option>
            {[...new Set(docs.map((d) => d.format))].map((f) => (
              <option key={f}>{f}</option>
            ))}
          </select>
        </label>
      </div>
      <form
        className="library-search"
        onSubmit={(e) => {
          e.preventDefault();
          void action(async () => {
            const r = await libraryApi<{
              results: Hit[];
              degraded_reason: string | null;
            }>("/search", "POST", {
              query,
              project_id: project || null,
              collection: collection || null,
              limit: 20,
            });
            setHits(r.results);
            if (r.degraded_reason)
              setError(
                tr("Lexical results only: ", "Solo resultados léxicos: ") +
                  r.degraded_reason,
              );
          });
        }}
      >
        <Icon name="memory" />
        <input
          aria-label={tr(
            "Search document content",
            "Buscar contenido de documentos",
          )}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={tr(
            "Search an idea, a name, a condition…",
            "Busca una idea, un nombre, una condición…",
          )}
          required
          maxLength={2400}
        />
        <button type="submit" disabled={busy || project === "*"}>
          {tr("Search", "Buscar")}
        </button>
        {hits && (
          <button type="button" onClick={() => setHits(null)}>
            {tr("Clear", "Limpiar")}
          </button>
        )}
      </form>
      {project === "*" && (
        <p className="muted">
          {tr(
            "Choose a project to upload or search its contents.",
            "Elige un proyecto para subir archivos o buscar en su contenido.",
          )}
        </p>
      )}
      {error && (
        <p role="alert" className="library-error">
          {error}
        </p>
      )}
      {hits !== null ? (
        <section
          className="library-hits"
          aria-label={tr("Search results", "Resultados de búsqueda")}
        >
          {hits.map((h) => (
            <button
              className="library-hit"
              key={h.id}
              onClick={() => openDocument(h.document_id, h.revision, h.id)}
            >
              <strong>{h.title}</strong>
              <small>
                {locationLabel(h)} · v{h.revision}
              </small>
              <p>{h.text}</p>
              <span>{h.branches.join(" + ")}</span>
            </button>
          ))}
          {!hits.length && (
            <p>
              {tr(
                "No matching evidence. Try another expression or check processing status.",
                "No encontramos evidencia. Prueba otra expresión o revisa el estado de procesamiento.",
              )}
            </p>
          )}
        </section>
      ) : (
        <section
          className="library-list"
          aria-label={tr("Documents", "Documentos")}
        >
          {!visible.length && (
            <div className="library-empty">
              <Icon name="documents" />
              <h2>
                {tr(
                  "A home for your knowledge",
                  "Un lugar para tu conocimiento",
                )}
              </h2>
              <p>
                {tr(
                  "Add your first document and continue the conversation with sources at hand.",
                  "Añade tu primer documento y continúa la conversación con tus fuentes a mano.",
                )}
              </p>
            </div>
          )}
          {visible.map((d) => (
            <article className="library-card" key={d.id}>
              <span className="library-format">{d.format.toUpperCase()}</span>
              <div className="library-card-body">
                <button
                  className="library-title"
                  onClick={() => openDocument(d.id)}
                >
                  {d.title}
                </button>
                <p>
                  {d.collection || tr("Unsorted", "Sin colección")} · v
                  {d.revision} · {(d.bytes / 1024).toFixed(1)} KB
                </p>
                <div className="library-card-tags">
                  {d.tags.map((t) => (
                    <span key={t}>{t}</span>
                  ))}
                  <Badge doc={d} />
                  {!d.enabled && (
                    <span>
                      {tr("Excluded from search", "Fuera de búsqueda")}
                    </span>
                  )}
                </div>
                <small>
                  {d.embedding_status === "ready"
                    ? tr(
                        "Semantic search ready",
                        "Búsqueda semántica disponible",
                      )
                    : d.embedding_status === "failed"
                      ? tr(
                          "Embeddings need attention",
                          "Embeddings requieren atención",
                        )
                      : tr("Embeddings pending", "Embeddings pendientes")}
                </small>
              </div>
              <div className="library-card-actions">
                <button
                  disabled={!d.enabled}
                  onClick={() => {
                    setSources([d]);
                    window.dispatchEvent(
                      new CustomEvent("fenix:design-chat", {
                        detail: {
                          text:
                            tr(
                              "Help me with this document: ",
                              "Ayúdame con este documento: ",
                            ) + d.title,
                        },
                      }),
                    );
                  }}
                >
                  {tr("Ask Fénix", "Preguntar a Fénix")}
                </button>
                <button
                  onClick={() => {
                    setEditing(d);
                    setName(d.title);
                    setNewCollection(d.collection);
                    setTags(d.tags.join(", "));
                  }}
                >
                  {tr("Manage", "Gestionar")}
                </button>
              </div>
            </article>
          ))}
          {next !== null && (
            <button
              disabled={busy}
              onClick={() => void action(() => refresh(next))}
            >
              {tr("Load more", "Cargar más")}
            </button>
          )}
        </section>
      )}
      {editing && (
        <section className="library-manage card">
          <h2>{tr("Manage document", "Gestionar documento")}</h2>
          <label>
            {tr("Name", "Nombre")}
            <input
              value={name}
              maxLength={240}
              onChange={(e) => setName(e.target.value)}
            />
          </label>
          <label>
            {tr("Collection", "Colección")}
            <input
              value={newCollection}
              maxLength={120}
              onChange={(e) => setNewCollection(e.target.value)}
            />
          </label>
          <label>
            {tr("Tags, separated by commas", "Etiquetas, separadas por comas")}
            <input value={tags} onChange={(e) => setTags(e.target.value)} />
          </label>
          <div className="library-actions">
            <button
              disabled={busy}
              onClick={() =>
                void action(async () => {
                  await libraryApi(`/${editing.id}`, "PUT", {
                    expected_revision: editing.revision,
                    title: name,
                    collection: newCollection,
                    tags: tags
                      .split(",")
                      .map((t) => t.trim())
                      .filter(Boolean),
                    enabled: editing.enabled,
                  });
                  setEditing(null);
                  await refresh();
                })
              }
            >
              {tr("Save", "Guardar")}
            </button>
            <button
              onClick={() =>
                void action(async () => {
                  await libraryApi(`/${editing.id}`, "PUT", {
                    ...editing,
                    expected_revision: editing.revision,
                    enabled: !editing.enabled,
                  });
                  setEditing(null);
                  await refresh();
                })
              }
            >
              {editing.enabled
                ? tr("Exclude from search", "Excluir de búsqueda")
                : tr("Include in search", "Incluir en búsqueda")}
            </button>
            <button
              disabled={busy}
              onClick={() =>
                void action(async () => {
                  await libraryApi(`/${editing.id}/retry`, "POST", {
                    expected_revision: editing.revision,
                  });
                  setEditing(null);
                  await refresh();
                })
              }
            >
              {tr("Retry processing", "Reintentar procesamiento")}
            </button>
            <label className="design-upload">
              {tr("Upload new version", "Subir nueva versión")}
              <input
                type="file"
                accept={accept}
                disabled={busy}
                onChange={(e) => {
                  const f = e.target.files?.[0];
                  if (f)
                    void action(async () => {
                      await upload(
                        f,
                        editing.project_id,
                        editing.collection,
                        editing,
                      );
                      setEditing(null);
                      await refresh();
                    });
                }}
              />
            </label>
            <button
              disabled={busy}
              onClick={() => {
                if (
                  confirm(
                    tr(
                      "Delete this document, every version and its search index? Existing citations will no longer open.",
                      "¿Eliminar el documento, todas sus versiones y su índice? Las citas existentes ya no abrirán.",
                    ),
                  )
                )
                  void action(async () => {
                    await libraryApi(`/${editing.id}`, "DELETE", {
                      expected_revision: editing.revision,
                    });
                    setSources(sources.filter((d) => d.id !== editing.id));
                    setEditing(null);
                    closeSource();
                    await refresh();
                  });
              }}
            >
              {tr("Delete", "Eliminar")}
            </button>
            <button onClick={() => setEditing(null)}>
              {tr("Close", "Cerrar")}
            </button>
          </div>
        </section>
      )}
      {settings && (
        <details className="library-settings">
          <summary>
            {tr("Document processing", "Procesamiento de documentos")} ·{" "}
            {settings.model}
          </summary>
          <p>
            {tr(
              "OpenAI receives the extracted text for embeddings. Originals and indexes remain in your Fénix account. Conversation answers have separate model costs.",
              "OpenAI recibe el texto extraído para crear embeddings. Los originales y los índices quedan en tu cuenta de Fénix. Las respuestas del chat tienen un coste aparte.",
            )}
          </p>
          <p>
            {settings.configured
              ? tr(
                  "Using your configured OpenAI key.",
                  "Usando tu clave de OpenAI configurada.",
                )
              : tr(
                  "An OpenAI key is needed for semantic search. Lexical search is available.",
                  "Falta una clave de OpenAI para búsqueda semántica. La búsqueda léxica está disponible.",
                )}
          </p>
          <label>
            <input
              type="checkbox"
              checked={settings.enabled}
              onChange={(e) =>
                void action(async () =>
                  setSettings(
                    await libraryApi("/settings", "PUT", {
                      enabled: e.target.checked,
                    }),
                  ),
                )
              }
            />
            {tr("Enable embeddings", "Activar embeddings")}
          </label>
          <label>
            {tr(
              "Optional OpenAI key override",
              "Clave OpenAI alternativa (opcional)",
            )}
            <input
              type="password"
              autoComplete="new-password"
              value={key}
              onChange={(e) => setKey(e.target.value)}
            />
          </label>
          <button
            disabled={!key || busy}
            onClick={() =>
              void action(async () => {
                setSettings(
                  await libraryApi("/settings", "PUT", {
                    enabled: true,
                    api_key: key,
                  }),
                );
                setKey("");
              })
            }
          >
            {tr("Save key", "Guardar clave")}
          </button>
          {!settings.execution_available && (
            <p>
              {tr(
                "PDF/scans require the isolated OCR execution image. Text and Office documents can be indexed now.",
                "PDF y escaneos requieren la imagen de ejecución OCR aislada. Ya puedes indexar texto y documentos Office.",
              )}
            </p>
          )}
        </details>
      )}
    </main>
  );
}
function SourcePanel() {
  const selected = useSyncExternalStore(panelSubscribe, () => openSource);
  const [detail, setDetail] = useState<Detail | null>(null),
    [error, setError] = useState(""),
    [tab, setTab] = useState("source"),
    [busy, setBusy] = useState(false);
  const close = useRef<HTMLButtonElement>(null),
    container = useRef<HTMLElement>(null);
  useEffect(() => {
    if (!selected) return;
    const previous = document.activeElement as HTMLElement | null;
    close.current?.focus();
    return () => previous?.focus();
  }, [selected?.id]);
  useEffect(() => {
    if (!selected) return;
    let active = true;
    setError("");
    setDetail(null);
    setBusy(true);
    setTab("source");
    const q = new URLSearchParams();
    if (selected.revision) q.set("revision", String(selected.revision));
    if (selected.chunk) q.set("chunk", selected.chunk);
    void libraryApi<Detail>(`/${selected.id}?${q}`)
      .then((d) => {
        if (active) {
          setDetail(d);
          updateTab(`library:${d.document.id}:`, {
            title: d.document.title,
            project_id: d.document.project_id,
          });
        }
      })
      .catch((e) => {
        if (active) setError(message(e));
      })
      .finally(() => {
        if (active) setBusy(false);
      });
    return () => {
      active = false;
    };
  }, [selected]);
  useEffect(() => {
    if (!selected) return;
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") hideWorkspace();
      if (
        !document.documentElement.classList.contains("workspace-open") &&
        e.key === "Tab" &&
        matchMedia("(max-width:900px)").matches
      ) {
        const nodes = Array.from(
          container.current?.querySelectorAll<HTMLElement>(
            "button:not(:disabled),a[href],select,input",
          ) || [],
        ).filter((n) => n.getClientRects().length);
        if (e.shiftKey && document.activeElement === nodes[0]) {
          e.preventDefault();
          nodes.at(-1)?.focus();
        } else if (!e.shiftKey && document.activeElement === nodes.at(-1)) {
          e.preventDefault();
          nodes[0]?.focus();
        }
      }
    };
    document.addEventListener("keydown", key);
    return () => document.removeEventListener("keydown", key);
  }, [selected]);
  useEffect(() => {
    if (
      !selected ||
      !detail ||
      !["queued", "processing"].includes(detail.document.status)
    )
      return;
    let active = true;
    const timer = setInterval(() => {
      if (document.visibilityState !== "visible") return;
      void libraryApi<Detail>(
        `/${detail.document.id}?revision=${detail.document.revision}`,
      )
        .then((value) => {
          if (active) setDetail(value);
        })
        .catch((error) => {
          if (active) setError(message(error));
        });
    }, 3500);
    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [
    selected,
    detail?.document.status,
    detail?.document.id,
    detail?.document.revision,
  ]);
  useEffect(() => {
    if (detail && selected?.chunk)
      requestAnimationFrame(() =>
        document
          .getElementById(`source-${selected.chunk}`)
          ?.scrollIntoView({ block: "nearest" }),
      );
  }, [detail, selected?.chunk]);
  if (!selected) return null;
  const d = detail?.document,
    cited = detail?.chunks.find((c) => c.id === selected.chunk);
  const original = d
    ? `/api/documents/${d.id}/original?revision=${d.revision}`
    : "";
  const highlight =
    cited?.location.page && cited.location.polygon
      ? { page: cited.location.page, polygon: cited.location.polygon }
      : undefined;
  return (
    <aside
      ref={container}
      className="design-panel library-panel"
      aria-label={tr("Document source", "Fuente del documento")}
    >
      <header className="design-panel-head">
        <div>
          <small>{tr("DOCUMENT SOURCE", "FUENTE DOCUMENTAL")}</small>
          <h2>{d?.title || tr("Opening source…", "Abriendo fuente…")}</h2>
          {d && (
            <span>
              v{d.revision} · <Badge doc={d} />
            </span>
          )}
        </div>
        <button
          ref={close}
          onClick={hideWorkspace}
          aria-label={tr(
            "Back to chat / close source",
            "Volver al chat / cerrar fuente",
          )}
        >
          ×
        </button>
      </header>
      {d && (
        <div className="library-panel-toolbar">
          <button
            onClick={() => setTab("source")}
            aria-pressed={tab === "source"}
          >
            {tr("Source", "Original")}
          </button>
          <button onClick={() => setTab("text")} aria-pressed={tab === "text"}>
            {tr("Extracted text", "Texto extraído")}
          </button>
          <select
            aria-label={tr("Document version", "Versión del documento")}
            value={d.revision}
            onChange={(e) => openDocument(d.id, Number(e.target.value))}
          >
            {detail?.revisions.map((v) => (
              <option key={v.revision} value={v.revision}>
                v{v.revision} ·{" "}
                {new Date(v.updated_at).toLocaleDateString(language)}
              </option>
            ))}
          </select>
          <a href={original} download={d.filename}>
            {tr("Download", "Descargar")}
          </a>
        </div>
      )}
      <div className="library-panel-body">
        {busy && (
          <Loading
            label={tr("Opening your evidence…", "Abriendo tu evidencia…")}
          />
        )}{" "}
        {error && (
          <p role="alert" className="library-error">
            {error}
          </p>
        )}
        {d?.error && (
          <p role="status" className="library-error">
            {d.error}
          </p>
        )}
        {d?.warnings.length ? (
          <details className="library-warnings">
            <summary>
              {tr("Extraction notes", "Notas de extracción")} (
              {d.warnings.length})
            </summary>
            {d.warnings.map((w, i) => (
              <p key={i}>{w}</p>
            ))}
          </details>
        ) : null}
        {d && tab === "source" && d.format === "pdf" && (
          <Suspense
            fallback={<Loading label={tr("Loading PDF…", "Cargando PDF…")} />}
          >
            <PdfViewer
              sourceUrl={original}
              title={d.title}
              initialPage={selected.page || cited?.location.page || 1}
              highlight={highlight}
              language={language}
            />
          </Suspense>
        )}
        {d && tab === "source" && ["png", "jpg", "jpeg"].includes(d.format) && (
          <img
            className="library-original-image"
            src={original}
            alt={d.title}
          />
        )}
        {d && ["docx", "xlsx", "pptx"].includes(d.format) && (
          <p className="muted">
            {tr(
              "Extracted content with original part/cell references; visual formatting and formula recalculation are not reproduced.",
              "Contenido extraído con referencias a partes/celdas; no reproduce el formato visual ni recalcula fórmulas.",
            )}
          </p>
        )}
        {detail?.chunks.map((c) => (
          <article
            key={c.id}
            id={`source-${c.id}`}
            className={`library-fragment ${selected.chunk === c.id ? "cited" : ""}`}
          >
            <div className="library-fragment-head">
              <small>
                {locationLabel(c) ||
                  tr("Source fragment", "Fragmento de origen")}
              </small>
              <button
                onClick={() => {
                  void navigator.clipboard
                    .writeText(
                      `${location.origin}/documents?document=${d!.id}&revision=${d!.revision}&chunk=${c.id}`,
                    )
                    .catch((e) => setError(message(e)));
                }}
              >
                {tr("Copy citation", "Copiar cita")}
              </button>
            </div>
            <Markdown>{c.text}</Markdown>
          </article>
        ))}
        {d &&
          detail?.next_offset !== null &&
          detail?.next_offset !== undefined && (
            <button
              disabled={busy}
              onClick={async () => {
                setBusy(true);
                try {
                  const more = await libraryApi<Detail>(
                    `/${d.id}?revision=${d.revision}&offset=${detail.next_offset}`,
                  );
                  if (selected !== openSource) return;
                  setDetail({
                    ...more,
                    chunks: [...detail.chunks, ...more.chunks],
                  });
                } catch (e) {
                  if (selected === openSource) setError(message(e));
                } finally {
                  if (selected === openSource) setBusy(false);
                }
              }}
            >
              {tr("Read more", "Leer más")}
            </button>
          )}
        {d && !detail?.chunks.length && (
          <p>
            {tr(
              "No extracted text yet. Check processing status in the library.",
              "Aún no hay texto extraído. Revisa el procesamiento en la biblioteca.",
            )}
          </p>
        )}
      </div>
      <footer className="library-panel-footer">
        {tr(
          "Source content is evidence. Ask Fénix in chat to interpret or edit it.",
          "El contenido es evidencia. Pide a Fénix en el chat que lo interprete o edite.",
        )}
      </footer>
    </aside>
  );
}
