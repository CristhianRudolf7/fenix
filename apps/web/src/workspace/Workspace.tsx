import {
  useState,
  useEffect,
  useRef,
  useSyncExternalStore,
  lazy,
  Suspense,
} from "react";
import { createRoot } from "react-dom/client";
import { Loading } from "../components/loading";
import { Markdown } from "../components/markdown";
import { Viewer } from "../design/viewers";
import type { Visual } from "../design/types";
import {
  snapshot,
  subscribe,
  tr,
  activeTarget,
  openTarget,
  activateTab,
  closeTab,
  hideWorkspace,
  showExplorer,
  closeExplorer,
  registerAdapter,
  updateTab,
  type Target,
  type Tab,
} from "./store";
import "./workspace.css";
const Pdf = lazy(() => import("../design/pdf-viewer"));
async function api<T>(url: string): Promise<T> {
  const r = await fetch(url);
  if (r.status === 401) {
    location.reload();
    throw Error("Session expired");
  }
  if (!r.ok) {
    const body = await r.json().catch(() => ({}));
    throw Error(body.error || "Could not open this item");
  }
  return r.json();
}
export function fileUrl(endpoint: string, t: Target, offset = 0) {
  const q = new URLSearchParams({ path: t.path || "", offset: String(offset) });
  if (t.page) q.set("page", String(t.page));
  if (t.project_id) q.set("project_id", t.project_id);
  return `/api/workspace/${endpoint}?${q}`;
}
import { WorkflowPanel } from "../workflows/WorkflowPanel";
let initialized = false;
export function ensureWorkspace() {
  if (initialized) return;
  initialized = true;
  const node = document.createElement("div");
  node.id = "workspace-root";
  document.body.append(node);
  createRoot(node).render(<Workspace />);
}
let file: Tab | null = null;
const fileListeners = new Set<() => void>();
const subscribeFile = (fn: () => void) => {
  fileListeners.add(fn);
  return () => {
    fileListeners.delete(fn);
  };
};
registerAdapter("file", {
  show: (t) => {
    file = t;
    fileListeners.forEach((f) => f());
  },
  hide: () => {
    file = null;
    fileListeners.forEach((f) => f());
  },
});
function Workspace() {
  const s = useSyncExternalStore(subscribe, snapshot);
  const tabs = useRef<HTMLDivElement>(null);
  useEffect(() => {
    tabs.current
      ?.querySelector('[aria-selected="true"]')
      ?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, [s.active]);
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !s.explorer && !document.fullscreenElement)
        hideWorkspace();
      if (
        e.key === "Tab" &&
        s.visible &&
        !s.explorer &&
        matchMedia("(max-width: 900px)").matches
      ) {
        const nodes = Array.from(
          document.querySelectorAll<HTMLElement>(
            '.workspace-chrome button:not(:disabled), .design-panel button:not(:disabled), .design-panel a[href], .design-panel input, .design-panel select, .design-panel textarea, .design-panel [tabindex="0"]',
          ),
        ).filter((n) => n.getClientRects().length && n.tabIndex >= 0);
        if (e.shiftKey && document.activeElement === nodes[0]) {
          e.preventDefault();
          nodes.at(-1)?.focus();
        } else if (!e.shiftKey && document.activeElement === nodes.at(-1)) {
          e.preventDefault();
          nodes[0]?.focus();
        }
      }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [s.explorer, s.visible]);
  return (
    <>
      {!s.visible && (
        <button className="workspace-launcher" onClick={showExplorer}>
          ▱ {tr("Files", "Archivos")}
        </button>
      )}
      {s.visible && (
        <header className="workspace-chrome">
          <div className="workspace-heading">
            <strong>{tr("Workspace", "Espacio de trabajo")}</strong>
            <div>
              <button onClick={showExplorer}>
                ▱ {tr("Files", "Archivos")}
              </button>
              <button
                aria-label={tr("Close workspace", "Cerrar espacio de trabajo")}
                onClick={hideWorkspace}
              >
                ×
              </button>
            </div>
          </div>
          <div
            className="workspace-tabs"
            role="tablist"
            aria-label={tr("Open files", "Archivos abiertos")}
            ref={tabs}
          >
            {s.tabs.map((t, i) => (
              <div
                key={t.key}
                className={
                  t.key === s.active
                    ? "workspace-tab selected"
                    : "workspace-tab"
                }
              >
                <button
                  role="tab"
                  aria-selected={t.key === s.active}
                  tabIndex={t.key === s.active ? 0 : -1}
                  title={`${t.title || t.path || t.id} · ${t.project_id || "General"}`}
                  onKeyDown={(e) => {
                    if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
                      e.preventDefault();
                      activateTab(
                        s.tabs[
                          (i +
                            (e.key === "ArrowRight" ? 1 : s.tabs.length - 1)) %
                            s.tabs.length
                        ].key,
                      );
                    }
                    if (e.key === "ArrowRight" || e.key === "ArrowLeft")
                      requestAnimationFrame(() =>
                        tabs.current
                          ?.querySelector<HTMLElement>('[aria-selected="true"]')
                          ?.focus(),
                      );
                    if (e.key === "Delete") closeTab(t.key);
                  }}
                  onClick={() => activateTab(t.key)}
                >
                  {t.title || t.path?.split("/").at(-1) || t.id}
                </button>
                <button
                  aria-label={`${tr("Close", "Cerrar")} ${t.title || t.path || t.id}`}
                  onClick={() => closeTab(t.key)}
                >
                  ×
                </button>
              </div>
            ))}
          </div>
        </header>
      )}
      <FilePanel />
      <WorkflowPanel />
      {s.explorer && <Explorer />}
    </>
  );
}
type Entry = {
  name: string;
  path?: string;
  directory?: boolean;
  bytes?: number;
  modified?: number;
  target?: Target;
};
const locations = new Map<string, string>();
function Explorer() {
  const s = useSyncExternalStore(subscribe, snapshot);
  const active = activeTarget();
  const initialProject = active ? active.project_id || "" : s.project || "";
  const [project, setProject] = useState(initialProject),
    [category, setCategory] = useState("files"),
    [path, setPath] = useState(
      locations.get(`${s.session}:${initialProject}:files`) || "",
    ),
    [query, setQuery] = useState(""),
    [entries, setEntries] = useState<Entry[]>([]),
    [projects, setProjects] = useState<{ id: string; name: string }[]>([]),
    [selected, setSelected] = useState<Entry | null>(null),
    [next, setNext] = useState<number | null>(null),
    [offset, setOffset] = useState(0),
    [error, setError] = useState(""),
    [busy, setBusy] = useState(true);
  const dialog = useRef<HTMLDivElement>(null),
    input = useRef<HTMLInputElement>(null);
  const favoriteKey = `fenix-workspace-favorites:${s.session}`;
  const [favorites, setFavorites] = useState<Target[]>(() => {
    try {
      return JSON.parse(sessionStorage.getItem(favoriteKey) || "[]");
    } catch {
      return [];
    }
  });
  useEffect(() => {
    const previous = document.activeElement as HTMLElement;
    input.current?.focus();
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopImmediatePropagation();
        closeExplorer();
      }
      if (e.key === "Tab") {
        const nodes = Array.from(
          dialog.current?.querySelectorAll<HTMLElement>(
            'button:not(:disabled),input,select,[tabindex="0"]',
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
    document.addEventListener("keydown", key, true);
    return () => {
      document.removeEventListener("keydown", key, true);
      previous?.isConnected && previous.focus();
    };
  }, []);
  useEffect(() => {
    void api<{ id: string; name: string }[]>("/api/projects")
      .then(setProjects)
      .catch((e) => setError(String(e)));
  }, []);
  useEffect(() => {
    let valid = true;
    setBusy(true);
    setError("");
    setSelected(null);
    const timer = setTimeout(() => {
      void (async () => {
        const p = project ? `&project_id=${encodeURIComponent(project)}` : "";
        let rows: Entry[] = [];
        let more: number | null = null;
        if (category === "files") {
          const data = await api<{
            entries: Entry[];
            next_offset: number | null;
          }>(
            fileUrl(
              "files",
              { kind: "file", path, project_id: project },
              offset,
            ) + `&query=${encodeURIComponent(query)}`,
          );
          rows = data.entries;
          more = data.next_offset;
        } else if (category === "library") {
          const d = await api<{
            documents: {
              id: string;
              title: string;
              project_id: string | null;
              bytes: number;
              updated_at: string;
            }[];
            next_offset: number | null;
          }>(`/api/documents?offset=${offset}${p}`);
          rows = d.documents.map((v) => ({
            name: v.title,
            bytes: v.bytes,
            modified: Date.parse(v.updated_at) / 1000,
            target: {
              kind: "library",
              id: v.id,
              title: v.title,
              project_id: v.project_id,
            },
          }));
          more = d.next_offset;
        } else if (category === "design") {
          const d = await api<{
            designs: {
              id: string;
              title: string;
              project_id: string | null;
              updated_at: string;
            }[];
            next_offset: number | null;
          }>(`/api/design/artifacts?offset=${offset}${p}`);
          rows = d.designs
            .filter((v) => (v.project_id || "") === project)
            .map((v) => ({
              name: v.title,
              modified: Date.parse(v.updated_at) / 1000,
              target: {
                kind: "design",
                id: v.id,
                title: v.title,
                project_id: v.project_id,
              },
            }));
          more = d.next_offset;
        } else
          rows = (category === "favorites" ? favorites : s.tabs)
            .filter((v) => (v.project_id || "") === project)
            .map((v) => ({ name: v.title || v.path || v.id || "", target: v }));
        if (category !== "files" && query)
          rows = rows.filter((v) =>
            v.name.toLowerCase().includes(query.toLowerCase()),
          );
        if (valid) {
          setEntries((old) => (offset ? [...old, ...rows] : rows));
          setNext(more);
        }
      })()
        .catch((e) => {
          if (valid) setError(e.message);
        })
        .finally(() => {
          if (valid) setBusy(false);
        });
    }, 150);
    return () => {
      valid = false;
      clearTimeout(timer);
    };
  }, [project, category, path, query, offset, favorites]);
  function navigate(destination: string) {
    locations.set(`${s.session}:${project}:files`, destination);
    setPath(destination);
    setOffset(0);
  }
  function open(e: Entry) {
    if (e.directory) {
      navigate(e.path || "");
      return;
    }
    openTarget(
      e.target || {
        kind: "file",
        path: e.path,
        title: e.name,
        project_id: project || null,
      },
    );
  }
  const chosen =
    selected && !selected.directory
      ? selected.target || {
          kind: "file" as const,
          path: selected.path,
          title: selected.name,
          project_id: project || null,
        }
      : null;
  return (
    <div
      className="workspace-modal-backdrop"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) closeExplorer();
      }}
    >
      <div
        className="workspace-explorer"
        role="dialog"
        aria-modal="true"
        aria-labelledby="explorer-title"
        ref={dialog}
      >
        <header>
          <div>
            <h2 id="explorer-title">
              {tr("Project files", "Archivos del proyecto")}
            </h2>
            <select
              aria-label={tr("Explorer project", "Proyecto del explorador")}
              value={project}
              onChange={(e) => {
                setProject(e.target.value);
                setPath(
                  locations.get(`${s.session}:${e.target.value}:files`) || "",
                );
                setOffset(0);
                setQuery("");
              }}
            >
              <option value="">General</option>
              {projects.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </div>
          <button
            onClick={closeExplorer}
            aria-label={tr("Close explorer", "Cerrar explorador")}
          >
            ×
          </button>
        </header>
        <div className="workspace-explorer-toolbar">
          <button
            disabled={category !== "files" || !path}
            aria-label={tr("Parent folder", "Carpeta superior")}
            onClick={() => navigate(path.split("/").slice(0, -1).join("/"))}
          >
            ↑
          </button>
          <nav aria-label={tr("Folder path", "Ruta de carpeta")}>
            <button onClick={() => navigate("")}>
              {tr("Project", "Proyecto")}
            </button>
            {category === "files" &&
              path
                .split("/")
                .filter(Boolean)
                .map((p, i) => (
                  <span key={i}>
                    {" "}
                    /{" "}
                    <button
                      onClick={() =>
                        navigate(
                          path
                            .split("/")
                            .slice(0, i + 1)
                            .join("/"),
                        )
                      }
                    >
                      {p}
                    </button>
                  </span>
                ))}
          </nav>
          <input
            ref={input}
            aria-label={tr("Find files", "Buscar archivos")}
            placeholder={tr("Search this folder…", "Buscar en esta carpeta…")}
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setOffset(0);
            }}
          />
        </div>
        <div className="workspace-explorer-body">
          <nav aria-label={tr("Locations", "Ubicaciones")}>
            {[
              ["files", "Project files", "Archivos del proyecto"],
              ["library", "Library", "Biblioteca"],
              ["design", "Designs", "Diseños"],
              ["recent", "Open files", "Abiertos"],
              ["favorites", "Favorites", "Favoritos"],
            ].map(([id, en, es]) => (
              <button
                key={id}
                aria-pressed={category === id}
                onClick={() => {
                  setCategory(id);
                  setOffset(0);
                  setQuery("");
                }}
              >
                {tr(en, es)}
              </button>
            ))}
          </nav>
          <div className="workspace-file-list">
            {error && <p role="alert">{error}</p>}
            {busy && !offset ? (
              <Loading />
            ) : (
              <>
                <div className="workspace-file-columns">
                  <span>{tr("Name", "Nombre")}</span>
                  <span>{tr("Modified", "Modificado")}</span>
                  <span>{tr("Size", "Tamaño")}</span>
                </div>
                {entries.map((e, i) => (
                  <button
                    className={`workspace-file-row ${selected === e ? "selected" : ""}`}
                    key={`${e.path || e.target?.id}:${i}`}
                    aria-pressed={selected === e}
                    onClick={() => setSelected(e)}
                    onDoubleClick={() => open(e)}
                    onKeyDown={(ev) => {
                      if (ev.key === "Enter") open(e);
                    }}
                  >
                    <span>
                      {e.directory ? "▱" : "▤"} {e.name}
                    </span>
                    <small>
                      {e.modified
                        ? new Date(e.modified * 1000).toLocaleDateString()
                        : "—"}
                    </small>
                    <small>
                      {e.directory
                        ? "—"
                        : e.bytes
                          ? `${Math.max(1, Math.round(e.bytes / 1024))} KB`
                          : "—"}
                    </small>
                  </button>
                ))}
                {!entries.length && (
                  <p className="muted">
                    {tr(
                      "No files in this location.",
                      "No hay archivos en esta ubicación.",
                    )}
                  </p>
                )}
                {next !== null && (
                  <button disabled={busy} onClick={() => setOffset(next!)}>
                    {tr("Load more", "Cargar más")}
                  </button>
                )}
              </>
            )}
          </div>
        </div>
        <footer>
          <div>
            <strong>
              {selected?.name || tr("Choose a file", "Elige un archivo")}
            </strong>
            <small>
              {tr("Opens in the right panel", "Se abrirá en el panel derecho")}
            </small>
          </div>
          {chosen && (
            <button
              onClick={() => {
                const key = JSON.stringify([
                  chosen.kind,
                  chosen.id,
                  chosen.path,
                  chosen.project_id,
                ]);
                const has = favorites.some(
                  (f) =>
                    JSON.stringify([f.kind, f.id, f.path, f.project_id]) ===
                    key,
                );
                const next = has
                  ? favorites.filter(
                      (f) =>
                        JSON.stringify([f.kind, f.id, f.path, f.project_id]) !==
                        key,
                    )
                  : [...favorites, chosen];
                setFavorites(next);
                sessionStorage.setItem(favoriteKey, JSON.stringify(next));
              }}
            >
              ☆ {tr("Favorite", "Favorito")}
            </button>
          )}
          <button onClick={closeExplorer}>{tr("Cancel", "Cancelar")}</button>
          <button
            className="primary"
            disabled={!selected || busy}
            onClick={() => selected && open(selected)}
          >
            {selected?.directory
              ? tr("Open folder", "Abrir carpeta")
              : tr("Open file", "Abrir archivo")}
          </button>
        </footer>
      </div>
    </div>
  );
}
type Preview = {
  title: string;
  kind: string;
  format: string;
  text?: string;
  sha256: string;
  extraction?: {
    blocks: Record<string, unknown>[];
    next_offset?: number | null;
    total_blocks: number;
  };
};
function FilePanel() {
  const tab = useSyncExternalStore(subscribeFile, () => file);
  const s = useSyncExternalStore(subscribe, snapshot);
  const [data, setData] = useState<Preview | null>(null),
    [error, setError] = useState(""),
    [offset, setOffset] = useState(0),
    [page, setPage] = useState(1);
  useEffect(() => {
    setOffset(0);
    setPage(tab?.page || 1);
  }, [tab]);
  useEffect(() => {
    if (!tab) return;
    let valid = true;
    setData(null);
    setError("");
    void api<Preview>(fileUrl("preview", { ...tab, page }, offset))
      .then((v) => {
        if (valid) {
          setData(v);
          updateTab(tab.key, { title: v.title });
        }
      })
      .catch((e) => {
        if (valid) setError(e.message);
      });
    return () => {
      valid = false;
    };
  }, [tab, offset, page]);
  if (!tab || !s.visible) return null;
  const url = fileUrl("content", tab);
  const office = data?.extraction;
  const slidePart =
    data?.format === "pptx" ? `ppt/slides/slide${page}.xml` : null;
  const blocks = slidePart
    ? office?.blocks.filter((b) => b.part === slidePart)
    : office?.blocks;
  return (
    <aside
      className="design-panel workspace-file-panel"
      aria-label={tr("File preview", "Vista del archivo")}
    >
      <header className="design-panel-head">
        <div>
          <h2>{tab.title || tab.path}</h2>
          <small>{tab.path}</small>
        </div>
        <a href={url} download={tab.title || tab.path?.split("/").at(-1)}>
          {tr("Download", "Descargar")}
        </a>
      </header>
      <div className="workspace-file-body">
        {error && <p role="alert">{error}</p>}
        {!data && !error && <Loading />}
        {data && (
          <>
            {data.kind === "pdf" && (
              <Suspense fallback={<Loading />}>
                <Pdf
                  title={data.title}
                  sourceUrl={url + `&v=${data.sha256}`}
                  initialPage={tab.page || 1}
                />
              </Suspense>
            )}
            {data.kind === "image" && (
              <img alt={data.title} src={url + `&v=${data.sha256}`} />
            )}{" "}
            {data.kind === "video" && (
              <video controls src={url + `&v=${data.sha256}`} />
            )}{" "}
            {data.kind === "markdown" && <Markdown>{data.text || ""}</Markdown>}
            {data.kind === "text" && <pre>{data.text}</pre>}
            {data.kind === "web" && (
              <Viewer
                visual={
                  {
                    id: tab.key,
                    title: data.title,
                    kind: "web",
                    content: { html: data.text },
                  } as Visual
                }
              />
            )}{" "}
            {data.kind === "download" && (
              <p>
                {tr(
                  "Preview unavailable for this format. Download the original.",
                  "Este formato no tiene vista previa. Puedes descargar el original.",
                )}
              </p>
            )}
            {office && (
              <>
                <p className="muted">
                  {tr(
                    "Extracted Office content; layout is not reproduced and formulas are not recalculated.",
                    "Contenido extraído de Office: no reproduce el diseño visual ni recalcula las fórmulas.",
                  )}
                </p>
                {slidePart && (
                  <label>
                    {tr("Slide", "Diapositiva")}{" "}
                    <input
                      type="number"
                      min={1}
                      max={2000}
                      value={page}
                      onChange={(e) => {
                        setOffset(0);
                        setPage(
                          Math.min(
                            2000,
                            Math.max(1, Number(e.target.value) || 1),
                          ),
                        );
                      }}
                    />
                  </label>
                )}
                {blocks?.map((b, i) => (
                  <article className="workspace-office-block" key={i}>
                    <small>
                      {String(b.part || "")} {String(b.cell || "")}
                    </small>
                    <p>{String(b.text ?? b.value ?? "")}</p>
                    {b.cached_value_only === true && (
                      <small>
                        {tr(
                          "Cached formula value",
                          "Valor almacenado de fórmula",
                        )}
                      </small>
                    )}
                  </article>
                ))}
                {slidePart && !blocks?.length && (
                  <p>
                    {tr(
                      "No text was extracted from this slide.",
                      "No se extrajo texto de esta diapositiva.",
                    )}
                  </p>
                )}
                {office.next_offset != null && (
                  <button onClick={() => setOffset(office.next_offset!)}>
                    {tr("More content", "Más contenido")}
                  </button>
                )}
                {offset > 0 && (
                  <button onClick={() => setOffset(0)}>
                    {tr("Start", "Inicio")}
                  </button>
                )}
              </>
            )}
          </>
        )}
      </div>
    </aside>
  );
}
