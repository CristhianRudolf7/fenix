import { useEffect, useId, useRef, useState } from "react";
import { Loading } from "../components/loading";
import { download, type Content, type DiagramGraph } from "./types";
import "./diagram.css";

// Mermaid has global configuration: serialize render jobs, including multiple previews.
let queue: Promise<unknown> = Promise.resolve();
function render(
  source: string,
  id: string,
  structured: boolean,
): Promise<string> {
  const job = queue.then(async () => {
    const [{ default: mermaid }, { default: purify }] = await Promise.all([
      import("mermaid"),
      import("dompurify"),
    ]);
    mermaid.initialize({
      startOnLoad: false,
      maxTextSize: 128_000,
      maxEdges: 600,
      securityLevel: "strict",
      theme: "neutral",
      htmlLabels: false,
      flowchart: { htmlLabels: false },
      suppressErrorRendering: true,
    });
    const output = await mermaid.render(id, source);
    let svg = output.svg;
    if (structured) {
      // Mermaid's SVG-only labels leave its numeric entity escapes as literal
      // text. Decode text nodes once, never markup/attributes or arbitrary HTML.
      const doc = new DOMParser().parseFromString(svg, "image/svg+xml");
      for (const label of doc.querySelectorAll("text")) {
        const walker = doc.createTreeWalker(label, NodeFilter.SHOW_TEXT);
        while (walker.nextNode()) {
          walker.currentNode.textContent =
            walker.currentNode.textContent!.replace(
              /&#(10|13|34|35|38|60|62|92|96);/g,
              (_, code: string) => String.fromCharCode(Number(code)),
            );
        }
      }
      svg = new XMLSerializer().serializeToString(doc.documentElement);
    }
    return purify.sanitize(svg, {
      USE_PROFILES: { svg: true, svgFilters: true },
      FORBID_TAGS: ["foreignObject", "a", "script"],
    });
  });
  queue = job.catch(() => undefined);
  return job;
}
type Camera = { x: number; y: number; scale: number };
type Frame = {
  svg: string;
  source: string;
  graph?: DiagramGraph;
  width: number;
  height: number;
  revision?: number;
};
const clamp = (n: number) => Math.min(8, Math.max(0.02, n));

/** Read-only camera: pointer gestures never mutate the diagram or call the model. */
export function DiagramViewer({
  content,
  title,
  revision,
}: {
  content: Content;
  title: string;
  revision?: number;
}) {
  const id = `diagram${useId().replace(/[^a-zA-Z0-9]/g, "")}`;
  const host = useRef<HTMLDivElement>(null);
  const viewport = useRef<HTMLDivElement>(null);
  const pointers = useRef(new Map<number, { x: number; y: number }>());
  const fitMode = useRef(true);
  const [camera, setCamera] = useState<Camera>({ x: 0, y: 0, scale: 1 });
  const [frame, setFrame] = useState<Frame | null>(null);
  const [error, setError] = useState("");
  const [pending, setPending] = useState(true);
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState("");
  const [expanded, setExpanded] = useState(false);
  const [language, setLanguage] = useState(document.documentElement.lang);
  const tr = (en: string, es: string) => (language.startsWith("es") ? es : en);
  useEffect(() => {
    const observer = new MutationObserver(() =>
      setLanguage(document.documentElement.lang),
    );
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["lang"],
    });
    return () => observer.disconnect();
  }, []);
  useEffect(() => {
    let current = true;
    setPending(true);
    setError("");
    void render(content.source || "", id, Boolean(content.graph))
      .then((svg) => {
        if (!current) return;
        const element = new DOMParser().parseFromString(
          svg,
          "image/svg+xml",
        ).documentElement;
        const bounds = element
          .getAttribute("viewBox")
          ?.split(/[ ,]+/)
          .map(Number);
        const width =
          bounds?.[2] ||
          Number.parseFloat(element.getAttribute("width") || "") ||
          800;
        const height =
          bounds?.[3] ||
          Number.parseFloat(element.getAttribute("height") || "") ||
          500;
        setFrame({
          svg,
          source: content.source || "",
          graph: content.graph,
          width,
          height,
          revision,
        });
        setPending(false);
      })
      .catch((e: unknown) => {
        if (current) {
          setError(String(e));
          setPending(false);
        }
      });
    return () => {
      current = false;
    };
  }, [content.source, content.graph, revision, id]);
  function fit() {
    const el = viewport.current;
    if (!el || !frame) return;
    fitMode.current = true;
    const scale = clamp(
      Math.min(
        (el.clientWidth - 40) / frame.width,
        (el.clientHeight - 40) / frame.height,
      ),
    );
    setCamera({
      scale,
      x: (el.clientWidth - frame.width * scale) / 2,
      y: (el.clientHeight - frame.height * scale) / 2,
    });
  }
  useEffect(() => {
    if (!frame || !viewport.current) return;
    const resize = () => {
      if (fitMode.current) fit();
    };
    resize();
    const observer = new ResizeObserver(resize);
    observer.observe(viewport.current);
    return () => observer.disconnect();
  }, [frame]);
  const zoomAt = (factor: number, x: number, y: number) => {
    fitMode.current = false;
    setCamera((c) => {
      const scale = clamp(c.scale * factor),
        ratio = scale / c.scale;
      return { scale, x: x - (x - c.x) * ratio, y: y - (y - c.y) * ratio };
    });
  };
  const zoom = (factor: number) => {
    const el = viewport.current;
    if (el) zoomAt(factor, el.clientWidth / 2, el.clientHeight / 2);
  };
  useEffect(() => {
    const el = viewport.current;
    if (!el) return;
    const wheel = (e: WheelEvent) => {
      e.preventDefault();
      const rect = el.getBoundingClientRect();
      zoomAt(
        Math.exp(-Math.max(-100, Math.min(100, e.deltaY)) * 0.003),
        e.clientX - rect.left,
        e.clientY - rect.top,
      );
    };
    el.addEventListener("wheel", wheel, { passive: false });
    return () => el.removeEventListener("wheel", wheel);
  }, []);
  useEffect(() => {
    const change = () =>
      setExpanded(document.fullscreenElement === host.current);
    document.addEventListener("fullscreenchange", change);
    return () => document.removeEventListener("fullscreenchange", change);
  }, []);
  const node = frame?.graph?.nodes.find((n) => n.id === selected);
  const nodes =
    frame?.graph?.nodes.filter((n) =>
      `${n.label} ${n.detail || ""} ${n.id}`
        .toLocaleLowerCase()
        .includes(query.toLocaleLowerCase()),
    ) || [];
  const base = title.replace(/[^\p{L}\p{N}_-]/gu, "-") || "diagram";
  return (
    <div className="diagram-explorer" ref={host}>
      <div
        className="diagram-controls"
        role="toolbar"
        aria-label={tr("Diagram navigation", "Navegación del diagrama")}
      >
        <button
          onClick={() => zoom(1 / 1.25)}
          aria-label={tr("Zoom out", "Alejar")}
        >
          −
        </button>
        <output aria-label="Zoom">{Math.round(camera.scale * 100)}%</output>
        <button
          onClick={() => zoom(1.25)}
          aria-label={tr("Zoom in", "Acercar")}
        >
          ＋
        </button>
        <button onClick={fit}>{tr("Fit", "Ajustar")}</button>
        <button onClick={() => zoom(1 / camera.scale)}>100%</button>
        <button
          onClick={() => {
            const action = expanded
              ? document.exitFullscreen()
              : host.current?.requestFullscreen();
            void action?.catch(() =>
              setError(
                tr(
                  "Fullscreen is unavailable in this browser.",
                  "Este navegador no permite pantalla completa.",
                ),
              ),
            );
          }}
        >
          {expanded
            ? tr("Exit fullscreen", "Salir de pantalla completa")
            : tr("Fullscreen", "Pantalla completa")}
        </button>
        <button
          disabled={!frame}
          onClick={() =>
            frame && download(`${base}.svg`, frame.svg, "image/svg+xml")
          }
        >
          SVG
        </button>
      </div>
      <div
        className="diagram-canvas diagram-viewport"
        ref={viewport}
        tabIndex={0}
        role="region"
        aria-label={tr(
          "Read-only diagram. Drag to pan; use plus and minus to zoom.",
          "Diagrama de solo lectura. Arrastra para desplazar; usa más y menos para ampliar.",
        )}
        aria-busy={pending}
        onKeyDown={(e) => {
          if (e.key === "+" || e.key === "=") {
            e.preventDefault();
            zoom(1.25);
          } else if (e.key === "-") {
            e.preventDefault();
            zoom(1 / 1.25);
          } else if (e.key === "0" || e.key === "Home") {
            e.preventDefault();
            fit();
          } else if (
            ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(e.key)
          ) {
            e.preventDefault();
            fitMode.current = false;
            setCamera((c) => ({
              ...c,
              x:
                c.x +
                (e.key === "ArrowLeft" ? 40 : e.key === "ArrowRight" ? -40 : 0),
              y:
                c.y +
                (e.key === "ArrowUp" ? 40 : e.key === "ArrowDown" ? -40 : 0),
            }));
          }
        }}
        onPointerDown={(e) => {
          if (e.button !== 0) return;
          fitMode.current = false;
          e.currentTarget.focus();
          e.currentTarget.setPointerCapture(e.pointerId);
          pointers.current.set(e.pointerId, { x: e.clientX, y: e.clientY });
        }}
        onPointerMove={(e) => {
          const previous = pointers.current.get(e.pointerId);
          if (!previous) return;
          const other = [...pointers.current.entries()].find(
            ([id]) => id !== e.pointerId,
          )?.[1];
          if (other) {
            const before = Math.hypot(
              previous.x - other.x,
              previous.y - other.y,
            );
            const after = Math.hypot(e.clientX - other.x, e.clientY - other.y);
            const rect = e.currentTarget.getBoundingClientRect();
            if (before > 0)
              zoomAt(
                after / before,
                (other.x + e.clientX) / 2 - rect.left,
                (other.y + e.clientY) / 2 - rect.top,
              );
          } else
            setCamera((c) => ({
              ...c,
              x: c.x + e.clientX - previous.x,
              y: c.y + e.clientY - previous.y,
            }));
          pointers.current.set(e.pointerId, { x: e.clientX, y: e.clientY });
        }}
        onPointerUp={(e) => pointers.current.delete(e.pointerId)}
        onPointerCancel={(e) => pointers.current.delete(e.pointerId)}
        onLostPointerCapture={(e) => pointers.current.delete(e.pointerId)}
      >
        {frame && (
          <img
            draggable={false}
            alt={title}
            src={`data:image/svg+xml;charset=utf-8,${encodeURIComponent(frame.svg)}`}
            style={{
              width: frame.width,
              height: frame.height,
              transform: `translate(${camera.x}px, ${camera.y}px) scale(${camera.scale})`,
            }}
          />
        )}
        {pending && (
          <div className="diagram-progress">
            <Loading label={tr("Preparing diagram…", "Preparando diagrama…")} />
          </div>
        )}
      </div>
      {error && (
        <div role="alert" className="design-error">
          <p>
            {frame
              ? tr(
                  `Showing the last valid diagram (v${frame.revision ?? "—"}). Ask the agent to correct this update.`,
                  `Se muestra el último diagrama válido (v${frame.revision ?? "—"}). Pide al agente corregir esta actualización.`,
                )
              : tr(
                  "Ask the agent to correct this diagram.",
                  "Pide al agente corregir este diagrama.",
                )}
          </p>
          <details>
            <summary>{tr("Details", "Detalles")}</summary>
            <pre>{error}</pre>
          </details>
        </div>
      )}
      <p className="diagram-hint">
        {tr(
          "Scroll or pinch to zoom · Drag to explore · Request changes in chat",
          "Rueda o pellizca para ampliar · Arrastra para explorar · Pide cambios en el chat",
        )}
      </p>
      {frame?.graph && (
        <div className="diagram-outline">
          <input
            aria-label={tr(
              "Search diagram elements",
              "Buscar elementos del diagrama",
            )}
            placeholder={tr(
              "Find a step or component…",
              "Busca un paso o componente…",
            )}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <div
            className="diagram-node-list"
            aria-label={tr("Diagram elements", "Elementos del diagrama")}
          >
            {nodes.map((n) => (
              <button
                key={n.id}
                aria-pressed={selected === n.id}
                onClick={() => setSelected(n.id)}
              >
                {n.label}
              </button>
            ))}
            {!nodes.length && (
              <p>
                {tr("No matching elements.", "No se encontraron elementos.")}
              </p>
            )}
          </div>
          {node && (
            <article className="diagram-node-detail">
              <h3>{node.label}</h3>
              {node.detail && <p>{node.detail}</p>}
              {!!node.references?.length && (
                <>
                  <h4>{tr("References", "Referencias")}</h4>
                  <ul>
                    {node.references.map((r, i) => (
                      <li key={i}>{r}</li>
                    ))}
                  </ul>
                </>
              )}
              <ul>
                {frame.graph.edges
                  ?.filter((e) => e.from === node.id || e.to === node.id)
                  .map((e) => (
                    <li key={e.id}>
                      {frame.graph!.nodes.find((n) => n.id === e.from)?.label} →{" "}
                      {frame.graph!.nodes.find((n) => n.id === e.to)?.label}
                      {e.label ? `: ${e.label}` : ""}
                    </li>
                  ))}
              </ul>
            </article>
          )}
        </div>
      )}
    </div>
  );
}
