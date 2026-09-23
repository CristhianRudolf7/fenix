import { useEffect, useRef, useState } from "react";
import { GlobalWorkerOptions, getDocument } from "pdfjs-dist";
import type { PDFDocumentProxy, RenderTask } from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { assetUrl } from "./types";
import "./pdf-viewer.css";

GlobalWorkerOptions.workerSrc = workerUrl;
const limit = 64 * 1024 * 1024;
const assets = import.meta.env.DEV
  ? "/node_modules/pdfjs-dist/"
  : "/assets/pdfjs/";

async function readPdf(url: string, signal: AbortSignal) {
  const response = await fetch(url, { signal, credentials: "same-origin" });
  if (!response.ok) throw new Error("PDF unavailable / PDF no disponible");
  if (Number(response.headers.get("Content-Length")) > limit)
    throw new Error("PDF exceeds 64 MiB / PDF supera 64 MiB");
  const reader = response.body?.getReader();
  if (!reader) throw new Error("PDF unavailable / PDF no disponible");
  const chunks: Uint8Array[] = [];
  let size = 0;
  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) break;
      size += value.length;
      if (size > limit)
        throw new Error("PDF exceeds 64 MiB / PDF supera 64 MiB");
      chunks.push(value);
    }
  } finally {
    await reader.cancel();
    reader.releaseLock();
  }
  const bytes = new Uint8Array(size);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.length;
  }
  return bytes;
}

export default function PdfViewer({
  assetId,
  sourceUrl,
  initialPage = 1,
  highlight,
  title,
  language = "en",
}: {
  assetId?: string;
  sourceUrl?: string;
  initialPage?: number;
  highlight?: { page: number; polygon: number[][] };
  title: string;
  language?: string;
}) {
  const tr = (en: string, es: string) => (language === "es" ? es : en);
  const canvas = useRef<HTMLCanvasElement>(null);
  const container = useRef<HTMLDivElement>(null);
  const [pdf, setPdf] = useState<PDFDocumentProxy | null>(null);
  const [page, setPage] = useState(1);
  const [width, setWidth] = useState(600);
  const [zoom, setZoom] = useState("fit");
  const [error, setError] = useState("");
  const [rendering, setRendering] = useState(false);
  const [password, setPassword] = useState("");
  const [unlock, setUnlock] = useState<((password: string) => void) | null>(
    null,
  );

  useEffect(() => {
    const abort = new AbortController();
    let loading: ReturnType<typeof getDocument> | undefined;
    void (async () => {
      try {
        const data = await readPdf(
          sourceUrl || assetUrl(assetId || ""),
          abort.signal,
        );
        if (abort.signal.aborted) return;
        loading = getDocument({
          data,
          enableXfa: false,
          cMapUrl: assets + "cmaps/",
          cMapPacked: true,
          standardFontDataUrl: assets + "standard_fonts/",
          wasmUrl: assets + "wasm/",
          iccUrl: assets + "iccs/",
          maxImageSize: 12_000_000,
          canvasMaxAreaInBytes: 48_000_000,
        });
        loading.onPassword = (update: (password: string) => void) => {
          if (!abort.signal.aborted) setUnlock(() => update);
        };
        const document = await loading.promise;
        if (abort.signal.aborted) return;
        if (document.numPages > 2000) {
          await loading.destroy();
          throw new Error("PDF exceeds 2,000 pages / PDF supera 2.000 páginas");
        }
        setUnlock(null);
        setPage(Math.max(1, Math.min(document.numPages, initialPage)));
        setPdf(document);
      } catch {
        if (!abort.signal.aborted) setError("open");
      }
    })();
    return () => {
      abort.abort();
      void loading?.destroy();
    };
  }, [assetId, sourceUrl]);

  useEffect(() => {
    if (pdf) setPage(Math.max(1, Math.min(pdf.numPages, initialPage)));
  }, [initialPage, pdf]);

  useEffect(() => {
    const target = container.current;
    if (!target) return;
    const observer = new ResizeObserver(([entry]) =>
      setWidth(Math.max(100, entry.contentRect.width)),
    );
    observer.observe(target);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (!pdf) return;
    let cancelled = false;
    let render: RenderTask | undefined;
    setRendering(true);
    setError("");
    void (async () => {
      try {
        const source = await pdf.getPage(page);
        if (cancelled || !canvas.current) return;
        const angle = source.rotate;
        const base = source.getViewport({ scale: 1, rotation: angle });
        const displayScale =
          zoom === "fit" ? width / base.width : (Number(zoom) * 96) / 72;
        const wanted = displayScale * Math.min(window.devicePixelRatio || 1, 2);
        const scale = Math.min(
          wanted,
          Math.sqrt(12_000_000 / (base.width * base.height)),
          16384 / Math.max(base.width, base.height),
        );
        const viewport = source.getViewport({ scale, rotation: angle });
        canvas.current.width = Math.ceil(viewport.width);
        canvas.current.height = Math.ceil(viewport.height);
        canvas.current.style.width = `${base.width * displayScale}px`;
        canvas.current.style.height = `${base.height * displayScale}px`;
        render = source.render({ canvas: canvas.current, viewport });
        await render.promise;
        if (
          !cancelled &&
          highlight?.page === page &&
          highlight.polygon.length >= 3
        ) {
          const points = highlight.polygon;
          if (
            points.every(
              (p) =>
                p.length === 2 &&
                p.every((v) => Number.isFinite(v) && v >= 0 && v <= 1),
            )
          ) {
            const ctx = canvas.current.getContext("2d");
            if (ctx) {
              ctx.save();
              ctx.fillStyle = "rgba(255, 198, 45, .28)";
              ctx.strokeStyle = "rgba(180, 120, 0, .8)";
              ctx.lineWidth = 2;
              ctx.beginPath();
              points.forEach((p, i) =>
                i
                  ? ctx.lineTo(p[0] * viewport.width, p[1] * viewport.height)
                  : ctx.moveTo(p[0] * viewport.width, p[1] * viewport.height),
              );
              ctx.closePath();
              ctx.fill();
              ctx.stroke();
              ctx.restore();
            }
          }
        }
      } catch {
        if (!cancelled) setError("page");
      } finally {
        if (!cancelled) setRendering(false);
      }
    })();
    return () => {
      cancelled = true;
      render?.cancel();
    };
  }, [pdf, page, zoom, width, highlight]);

  function changePage(value: number) {
    const next = Math.max(
      1,
      Math.min(
        pdf?.numPages || 1,
        Number.isFinite(value) ? Math.trunc(value) : page,
      ),
    );
    setPage(next);
    return next;
  }

  return (
    <section className="pdf-viewer" aria-label={`PDF: ${title}`}>
      {unlock && (
        <form
          className="pdf-unlock"
          onSubmit={(event) => {
            event.preventDefault();
            unlock(password);
            setPassword("");
            setUnlock(null);
          }}
        >
          <label>
            {tr("Password", "Contraseña")}
            <input
              type="password"
              autoComplete="off"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
            />
          </label>
          <button type="submit">{tr("Open", "Abrir")}</button>
        </form>
      )}
      {!pdf && !error && !unlock && (
        <p className="pdf-status" role="status">
          {tr("Loading PDF…", "Cargando PDF…")}
        </p>
      )}
      {pdf && (
        <div
          className="pdf-toolbar"
          role="group"
          aria-label={tr(
            "PDF navigation and zoom",
            "Navegación y zoom del PDF",
          )}
        >
          <div className="pdf-pagination">
            <button
              type="button"
              aria-label={tr("Previous page", "Página anterior")}
              title={tr("Previous page", "Página anterior")}
              disabled={page <= 1}
              onClick={() => changePage(page - 1)}
            >
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.7"
                aria-hidden="true"
              >
                <path d="m14 6-6 6 6 6" />
              </svg>
            </button>
            <input
              key={page}
              aria-label={tr("PDF page", "Página PDF")}
              type="number"
              inputMode="numeric"
              min={1}
              max={pdf.numPages}
              defaultValue={page}
              onBlur={(event) => {
                event.target.value = String(
                  changePage(Number(event.target.value) || page),
                );
              }}
              onKeyDown={(event) => {
                if (event.key === "Enter") event.currentTarget.blur();
              }}
            />
            <span className="pdf-page-count">/ {pdf.numPages}</span>
            <button
              type="button"
              aria-label={tr("Next page", "Página siguiente")}
              title={tr("Next page", "Página siguiente")}
              disabled={page >= pdf.numPages}
              onClick={() => changePage(page + 1)}
            >
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.7"
                aria-hidden="true"
              >
                <path d="m10 6 6 6-6 6" />
              </svg>
            </button>
          </div>
          <select
            aria-label={tr("PDF zoom", "Zoom del PDF")}
            value={zoom}
            onChange={(event) => setZoom(event.target.value)}
          >
            <option value="fit">{tr("Fit width", "Ajustar ancho")}</option>
            <option value="0.75">75%</option>
            <option value="1">100%</option>
            <option value="1.25">125%</option>
            <option value="1.5">150%</option>
            <option value="2">200%</option>
          </select>
        </div>
      )}
      {error && (
        <p className="pdf-status" role="alert">
          {error === "page"
            ? tr(
                "Could not display this page.",
                "No se pudo mostrar esta página.",
              )
            : tr(
                "Could not open this PDF. It may be damaged or unsupported.",
                "No se pudo abrir el PDF. Puede estar dañado o no ser compatible.",
              )}
        </p>
      )}
      <div
        className="pdf-canvas-container"
        ref={container}
        aria-busy={rendering}
      >
        <canvas
          ref={canvas}
          role="img"
          aria-label={`${title} · ${tr("Page", "Página")} ${page}`}
          hidden={!pdf || !!error}
        />
      </div>
    </section>
  );
}
