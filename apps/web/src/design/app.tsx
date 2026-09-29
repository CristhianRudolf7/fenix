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
  useLayoutEffect,
  useRef,
  useState,
  useSyncExternalStore,
  Component,
  type ReactNode,
} from "react";
import { createPortal } from "react-dom";
import { createRoot, type Root } from "react-dom/client";
import {
  api,
  assetUrl,
  download,
  labels,
  type Visual,
  type ArtifactPage,
  type Offer,
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
  offers: Offer[];
  // Template proposals preview in a centered modal; the right panel is for finished results.
  modal: boolean;
  heading: string;
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
  modal: false,
  heading: "",
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
let trayRoot: Root | undefined;
let panelRoot: Root | undefined;
let selectionSequence = 0;
let active = false;
let polling = false;
let previousFocus: HTMLElement | null = null;
const fail = (e: unknown) =>
  update({ error: e instanceof Error ? e.message : String(e), busy: false });
// Finished results open in the chat's right panel; template proposals never do.
function select(v: Visual, isTemplate = false) {
  if (isTemplate || state.modal) {
    openDesignModal(v, isTemplate);
    return;
  }
  openTarget({
    kind: "design",
    id: v.id,
    title: title(v),
    project_id: v.project_id,
  });
}
function openDesignModal(v: Visual, isTemplate = false) {
  ensurePanel();
  hideWorkspace();
  if (!state.modal) previousFocus = document.activeElement as HTMLElement;
  update({ modal: true, heading: title(v) || "" });
  void loadDesign(v, isTemplate);
}
/** Preview one catalog template in the centered modal, over the chat. */
export function openTemplatePreview(id: string) {
  openDesignModal({ id, title: "" } as Visual, true);
}
/** Template id from a legacy `/design?template=` link, or null for other links. */
export function legacyTemplate(url: URL) {
  if (url.origin !== location.origin) return null;
  if (url.pathname.replace(/\/+$/, "") !== "/design") return null;
  return url.searchParams.get("template");
}
// Saved choices in chat history link to their template; preview it in place.
document.addEventListener("click", (event) => {
  const link = (event.target as Element).closest?.("a[href]");
  const { button, ctrlKey, metaKey, shiftKey } = event as MouseEvent;
  if (!link || button !== 0 || ctrlKey || metaKey || shiftKey) return;
  const id = legacyTemplate(new URL(link.getAttribute("href")!, location.href));
  if (!id) return;
  event.preventDefault();
  openTemplatePreview(id);
});
registerAdapter("design", {
  show: (tab) => {
    ensurePanel();
    update({ modal: false });
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
  // Hiding the chat workspace must not dismiss a template preview modal.
  hide: () => {
    if (!state.modal) close();
  },
});
async function loadDesign(v: Visual, isTemplate = false, tab?: Tab) {
  window.dispatchEvent(new Event("fenix:design-preview"));
  const seq = ++selectionSequence;
  if (!state.modal) previousFocus = document.activeElement as HTMLElement;
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
        heading: title(full),
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
  update({
    selected: null,
    error: "",
    busy: false,
    editing: false,
    modal: false,
  });
  if (previousFocus?.isConnected) previousFocus.focus();
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
  if (state.modal) close();
  trayRoot?.unmount();
  trayRoot = undefined;
}
export function renderChatDesigns(language: string) {
  ensurePanel();
  active = true;
  update({ language });
  const node = document.getElementById("design-results");
  if (node) {
    trayRoot = createRoot(node);
    trayRoot.render(
      <>
        <ChatResults />
        <OfferCarousels />
      </>,
    );
  }
  void refreshDesigns();
}
function ChatResults() {
  const s = useStateStore();
  const [dismissed, setDismissed] = useState<string[]>([]);
  const [storageKey, setStorageKey] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    void fetch("/api/auth/session")
      .then(async (response) => {
        if (!response.ok) return;
        const account = await response.json();
        if (!active) return;
        const key = `fenix:hidden-chat-files:${account.user_id || "local"}`;
        try {
          const saved = JSON.parse(localStorage.getItem(key) || "[]");
          if (Array.isArray(saved))
            setDismissed(saved.filter((id) => typeof id === "string"));
        } catch {
          /* Storage can be unavailable in private browsing. */
        }
        setStorageKey(key);
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, []);
  function dismiss(id: string) {
    setDismissed((previous) => {
      const next = [...new Set([...previous, id])];
      if (storageKey) {
        try {
          localStorage.setItem(storageKey, JSON.stringify(next));
        } catch {
          /* Keep this session usable without storage. */
        }
      }
      return next;
    });
  }
  return (
    <div
      className="chat-design-results"
      aria-label={tr("Visual results", "Resultados visuales")}
    >
      {s.artifacts
        .filter((v) => !dismissed.includes(v.id))
        .map((v) => (
          <div className="chat-file-chip" key={v.id}>
            <button onClick={() => void select(v)} title={title(v)}>
              <span aria-hidden="true">◇</span> {title(v)}{" "}
              <small>v{v.revision}</small>
            </button>
            <button
              className="chat-file-dismiss"
              aria-label={`${tr("Hide", "Ocultar")} ${title(v)}`}
              title={tr("Hide from chat", "Ocultar del chat")}
              onClick={() => dismiss(v.id)}
            >
              <Icon name="close" />
            </button>
          </div>
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
  const [revision, setRevision] = useState(1);
  const [latest, setLatest] = useState(1);
  useEffect(() => {
    document.documentElement.classList.toggle(
      "design-preview-open",
      !!v && !s.modal,
    );
    if (v) {
      setMode("preview");
      setSource(JSON.stringify(v.content, null, 2));
      setRevision(v.revision || 1);
      setLatest(v.revision || 1);
      if (!s.modal) document.getElementById("workspace-back")?.focus();
    }
  }, [v?.id, s.modal]);
  useEffect(() => {
    if (v?.content) setSource(JSON.stringify(v.content, null, 2));
    if (v?.revision && v.revision > latest) {
      setLatest(v.revision);
      setRevision(v.revision);
    }
  }, [v]);
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (!v || s.modal) return;
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
  }, [v, s.modal]);
  if (!v) {
    if (!s.busy && !s.error) return null;
    const pending = (
      <div className="design-preview-content">
        {s.error ? <p role="alert">{s.error}</p> : <Loading />}
      </div>
    );
    return s.modal ? (
      <DesignModal>{pending}</DesignModal>
    ) : (
      <aside
        className="design-panel"
        aria-label={tr("Design preview", "Vista del diseño")}
      >
        {pending}
      </aside>
    );
  }
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
  // The chat sends the choice so the agent continues with the user's request.
  const chooseTemplate = () => {
    const template = { id: v.id, title: title(v) };
    close();
    window.dispatchEvent(
      new CustomEvent("fenix:design-choose", { detail: { template } }),
    );
  };
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
  const toolbar = !isPdf && (
    <div className="design-toolbar">
      {!s.modal && (
        <button
          className="subtle icon-button"
          onClick={exportResult}
          aria-label={tr("Download", "Descargar")}
          title={tr("Download", "Descargar")}
        >
          <Icon name="download" />
        </button>
      )}
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
  );
  const body = (
    <>
      {s.error && (
        <p role="alert" className="design-error">
          {s.error}
        </p>
      )}
      {!s.modal && v.kind === "web" && (
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
        <footer className="design-panel-foot design-choice-foot">
          <p>
            {tr(
              "Fénix will adapt this design to your request. Sample text and data are illustrative.",
              "Fénix adaptará este diseño a tu solicitud. Los textos y datos de muestra son ilustrativos.",
            )}
          </p>
          <button
            className="primary"
            disabled={s.busy}
            onClick={chooseTemplate}
          >
            {tr("Choose this design", "Elegir este diseño")}
          </button>
        </footer>
      )}
    </>
  );
  if (s.modal)
    return (
      <DesignModal kind={v.kind} pdf={isPdf} actions={toolbar}>
        {body}
      </DesignModal>
    );
  return (
    <aside
      className={`design-panel${isPdf ? " design-panel-pdf" : ""}`}
      role="dialog"
      aria-label={tr("Design preview", "Vista del diseño")}
    >
      {toolbar}
      {body}
    </aside>
  );
}
const focusable =
  'button:not(:disabled), a[href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), iframe, [tabindex]:not([tabindex="-1"])';
function DesignModal({
  kind,
  pdf,
  actions,
  children,
}: {
  kind?: Visual["kind"];
  pdf?: boolean;
  actions?: ReactNode;
  children: ReactNode;
}) {
  const s = useStateStore();
  const dialog = useRef<HTMLDivElement>(null);
  const closeButton = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    const root = document.documentElement;
    root.classList.add("design-modal-open");
    closeButton.current?.focus({ preventScroll: true });
    const key = (e: KeyboardEvent) => {
      if (e.defaultPrevented) return;
      if (e.key === "Escape" && !document.fullscreenElement) {
        e.preventDefault();
        close();
        return;
      }
      if (e.key !== "Tab" || !dialog.current) return;
      const nodes = [
        ...dialog.current.querySelectorAll<HTMLElement>(focusable),
      ].filter((n) => n.tabIndex >= 0 && n.getClientRects().length);
      const first = nodes[0],
        last = nodes.at(-1);
      const current = document.activeElement;
      if (!dialog.current.contains(current)) {
        e.preventDefault();
        first?.focus();
      } else if (e.shiftKey && current === first) {
        e.preventDefault();
        last?.focus();
      } else if (!e.shiftKey && current === last) {
        e.preventDefault();
        first?.focus();
      }
    };
    // Focus can leave through an embedded frame; bring it back into the dialog.
    const contain = (e: FocusEvent) => {
      if (dialog.current && !dialog.current.contains(e.target as Node))
        closeButton.current?.focus({ preventScroll: true });
    };
    document.addEventListener("keydown", key);
    document.addEventListener("focusin", contain);
    return () => {
      document.removeEventListener("keydown", key);
      document.removeEventListener("focusin", contain);
      root.classList.remove("design-modal-open");
    };
  }, []);
  return (
    <div
      className="design-modal-backdrop"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) close();
      }}
    >
      <div
        ref={dialog}
        role="dialog"
        aria-modal="true"
        aria-labelledby="design-modal-title"
        aria-busy={s.busy}
        className={`design-panel design-modal${pdf ? " design-panel-pdf" : ""}`}
      >
        <header className="design-panel-head design-modal-head">
          <div>
            <small>
              {kind
                ? labels[kind][s.language === "es" ? 1 : 0]
                : tr("Design", "Diseño")}
              {s.isTemplate && s.selected
                ? ` · ${tr("Design proposal", "Propuesta de diseño")}`
                : ""}
            </small>
            <h2 id="design-modal-title">
              {s.heading || tr("Design preview", "Vista del diseño")}
            </h2>
          </div>
          <div className="design-modal-actions">
            {actions}
            <button
              ref={closeButton}
              className="subtle icon-button"
              aria-label={tr("Close preview", "Cerrar vista")}
              title={tr("Close (Esc)", "Cerrar (Esc)")}
              onClick={close}
            >
              <Icon name="close" />
            </button>
          </div>
        </header>
        {children}
      </div>
    </div>
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
type Slot = { offer: Offer; node: HTMLElement };
const time = (value?: string) => {
  const parsed = Date.parse(value || "");
  return Number.isNaN(parsed) ? 0 : parsed;
};
/** Place each proposal set inside the assistant reply that presented it. */
function offerSlots(offers: Offer[]): Slot[] {
  const list = document.getElementById("message-list");
  if (!list) return [];
  const messages = [...list.querySelectorAll<HTMLElement>("article.message")];
  const times = messages.map((m) => time(m.dataset.createdAt));
  const slots: Slot[] = [];
  // Oldest first, so several sets in one reply keep their order.
  for (const offer of [...offers].reverse()) {
    let node = list.querySelector<HTMLElement>(
      `[data-offer-slot="${CSS.escape(offer.id)}"]`,
    );
    if (!node) {
      const created = time(offer.created_at);
      const next = messages[times.findIndex((t) => t >= created)];
      const host =
        messages
          .filter(
            (m) =>
              m.classList.contains("assistant") &&
              !!offer.task_id &&
              m.dataset.taskId === offer.task_id,
          )
          .at(-1) ||
        (next?.classList.contains("assistant") && times[0] <= created
          ? next
          : undefined);
      // A newer set than every loaded message belongs to the turn in progress;
      // an older one without its reply loaded stays out of this history page.
      if (!host && (!messages.length || created < Math.max(...times))) continue;
      node = document.createElement("div");
      node.dataset.offerSlot = offer.id;
      node.className = "design-offer-slot";
      if (host) {
        const body = host.querySelector(".message-body") || host;
        body.insertBefore(node, body.querySelector(":scope > .message-footer"));
      } else {
        node.classList.add("is-pending");
        list.append(node);
      }
    }
    slots.push({ offer, node });
  }
  return slots;
}
type Favorites = {
  ids: string[] | null;
  pending: string[];
  error: { offer: string; message: string } | null;
  toggle: (v: Visual, offer: string) => void;
};
function useFavorites(enabled: boolean): Favorites {
  const [ids, setIds] = useState<string[] | null>(null);
  const [pending, setPending] = useState<string[]>([]);
  const [error, setError] = useState<Favorites["error"]>(null);
  const current = useRef<string[]>([]);
  const busy = useRef(new Set<string>());
  current.current = ids || [];
  useEffect(() => {
    if (!enabled || ids) return;
    let live = true;
    // Without the saved list the star state would be a guess, so stars stay hidden.
    void api<{ id: string }[]>("/favorites")
      .then((rows) => live && setIds(rows.map((row) => row.id)))
      .catch(() => {});
    return () => {
      live = false;
    };
  }, [enabled, ids]);
  useEffect(() => {
    if (!error) return;
    const timer = setTimeout(() => setError(null), 6000);
    return () => clearTimeout(timer);
  }, [error]);
  async function toggle(v: Visual, offer: string) {
    if (busy.current.has(v.id)) return;
    const enabled = !current.current.includes(v.id);
    const apply = (on: boolean) =>
      setIds((list) =>
        on
          ? [...new Set([...(list || []), v.id])]
          : (list || []).filter((id) => id !== v.id),
      );
    busy.current.add(v.id);
    setPending([...busy.current]);
    setError(null);
    apply(enabled);
    try {
      await api(`/favorites/${encodeURIComponent(v.id)}`, { enabled }, "PUT");
    } catch (e) {
      apply(!enabled);
      setError({
        offer,
        message: `${tr("Could not update favorite", "No se pudo actualizar el favorito")}: ${title(v)}. ${e instanceof Error ? e.message : String(e)}`,
      });
    } finally {
      busy.current.delete(v.id);
      setPending([...busy.current]);
    }
  }
  return { ids, pending, error, toggle };
}
/** Proposal carousels live in the conversation, next to the reply that offered them. */
function OfferCarousels() {
  const s = useStateStore();
  const [slots, setSlots] = useState<Slot[]>([]);
  const [drawn, setDrawn] = useState(0);
  const follow = useRef(false);
  const offers = useRef(s.offers);
  offers.current = s.offers;
  const signature = s.offers.map((o) => o.id).join();
  useEffect(() => {
    const redraw = () => setDrawn((n) => n + 1);
    window.addEventListener("fenix:messages-drawn", redraw);
    return () => window.removeEventListener("fenix:messages-drawn", redraw);
  }, []);
  useLayoutEffect(() => {
    const scroll = document.getElementById("chat-scroll");
    follow.current =
      !!scroll &&
      scroll.scrollHeight - scroll.scrollTop - scroll.clientHeight < 100;
    setSlots(offerSlots(offers.current));
  }, [signature, drawn]);
  useLayoutEffect(() => {
    const scroll = document.getElementById("chat-scroll");
    if (follow.current && scroll) scroll.scrollTop = scroll.scrollHeight;
    follow.current = false;
  }, [slots]);
  const favorites = useFavorites(slots.length > 0);
  return slots.map(({ offer, node }) =>
    createPortal(
      <OfferCarousel offer={offer} favorites={favorites} />,
      node,
      offer.id,
    ),
  );
}
function OfferCarousel({
  offer,
  favorites,
}: {
  offer: Offer;
  favorites: Favorites;
}) {
  const s = useStateStore();
  const track = useRef<HTMLUListElement>(null);
  const [edges, setEdges] = useState({ start: true, end: true });
  const measure = () => {
    const n = track.current;
    if (!n) return;
    setEdges({
      start: n.scrollLeft <= 2,
      end: n.scrollLeft + n.clientWidth >= n.scrollWidth - 2,
    });
  };
  useEffect(() => {
    measure();
    const observer = new ResizeObserver(measure);
    if (track.current) observer.observe(track.current);
    return () => observer.disconnect();
  }, [offer.templates.length]);
  const move = (direction: 1 | -1) =>
    track.current?.scrollBy({
      left: direction * track.current.clientWidth * 0.8,
      behavior: matchMedia("(prefers-reduced-motion: reduce)").matches
        ? "auto"
        : "smooth",
    });
  const heading = `design-offer-${offer.id}`;
  const error = favorites.error?.offer === offer.id ? favorites.error : null;
  return (
    <section className="design-offer" aria-labelledby={heading}>
      <header className="design-offer-head">
        <div>
          <h3 id={heading}>{tr("Design proposals", "Propuestas de diseño")}</h3>
          <p>
            {tr(
              "Open one to preview and choose it, or reply with its number.",
              "Abre una para verla y elegirla, o responde con su número.",
            )}
          </p>
        </div>
        {!(edges.start && edges.end) && (
          <div className="design-offer-nav">
            <button
              type="button"
              className="subtle icon-button design-offer-previous"
              aria-label={tr("Previous proposals", "Propuestas anteriores")}
              disabled={edges.start}
              onClick={() => move(-1)}
            >
              <Icon name="chevron" />
            </button>
            <button
              type="button"
              className="subtle icon-button"
              aria-label={tr("More proposals", "Más propuestas")}
              disabled={edges.end}
              onClick={() => move(1)}
            >
              <Icon name="chevron" />
            </button>
          </div>
        )}
      </header>
      <ul
        ref={track}
        className="design-offer-track"
        onScroll={measure}
        aria-label={tr("Design proposals", "Propuestas de diseño")}
      >
        {offer.templates.map((v, i) => {
          const favorite = !!favorites.ids?.includes(v.id);
          const pending = favorites.pending.includes(v.id);
          return (
            <li className="design-offer-card" key={v.id}>
              <button
                type="button"
                className="design-offer-open"
                aria-label={`${i + 1}. ${title(v)} · ${tr("Open preview", "Abrir vista previa")}`}
                onClick={() => openDesignModal(v, true)}
              >
                <Thumbnail v={v} />
                <span className="design-offer-meta">
                  <small>
                    {i + 1} · {labels[v.kind][s.language === "es" ? 1 : 0]}
                  </small>
                  <strong>{title(v)}</strong>
                </span>
              </button>
              {favorites.ids && (
                <button
                  type="button"
                  className="design-star"
                  aria-label={`${tr("Favorite", "Favorito")}: ${title(v)}`}
                  aria-pressed={favorite}
                  aria-disabled={pending}
                  aria-busy={pending}
                  onClick={() => favorites.toggle(v, offer.id)}
                >
                  <span aria-hidden="true">{favorite ? "★" : "☆"}</span>
                </button>
              )}
            </li>
          );
        })}
      </ul>
      <p className="design-offer-status" role="status" aria-live="polite">
        {error?.message}
      </p>
    </section>
  );
}
