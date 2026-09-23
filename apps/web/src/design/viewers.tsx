import { DiagramViewer } from "./diagram";
import { Markdown } from "../components/markdown";
import { Loading } from "../components/loading";
import { lazy, Suspense, useMemo, useState, useEffect } from "react";
import { ComposedSlide, ComposedChart } from "./template-compositions";
import type { Content, Visual } from "./types";
import { assetUrl } from "./types";
import {
  TemplateSlide,
  TemplateFrame,
  TemplateDocument,
} from "./template-visuals";

const PdfViewer = lazy(() => import("./pdf-viewer"));

function WebPreview({ content, title }: { content: Content; title: string }) {
  const src = useMemo(() => {
    const doc = new DOMParser().parseFromString(
      content.html || "",
      "text/html",
    );
    doc
      .querySelectorAll(
        "script,iframe,object,embed,base,meta[http-equiv],link,form",
      )
      .forEach((node) => {
        if (node.tagName === "FORM") node.replaceWith(...node.childNodes);
        else node.remove();
      });
    doc.querySelectorAll("[href]").forEach((n) => {
      if (!n.getAttribute("href")?.startsWith("#")) n.removeAttribute("href");
    });
    const policy = doc.createElement("meta");
    policy.httpEquiv = "Content-Security-Policy";
    policy.content =
      "default-src 'none'; style-src 'unsafe-inline'; img-src data: blob:; font-src data:; form-action 'none'; base-uri 'none'; object-src 'none'";
    doc.head.prepend(policy);
    const brand = content.brand;
    if (brand) {
      const style = doc.createElement("style");
      const color = (v: string) => (/^#[0-9a-f]{6}$/i.test(v) ? v : "#171717");
      const font = ["sans-serif", "serif", "monospace"].includes(brand.font)
        ? brand.font
        : "sans-serif";
      style.textContent = `:root{--brand-accent:${color(brand.accent)};--brand-bg:${color(brand.background)};--brand-text:${color(brand.text)};--brand-font:${font}}`;
      doc.head.append(style);
    }
    return "<!doctype html>" + doc.documentElement.outerHTML;
  }, [content]);
  return (
    <iframe
      className="design-frame"
      sandbox=""
      referrerPolicy="no-referrer"
      srcDoc={src}
      title={title}
    />
  );
}
function Chart({ content: c }: { content: Content }) {
  const labels = c.labels || [];
  const values = c.values || [];
  const min = Math.min(0, ...values),
    max = Math.max(1, ...values),
    span = max - min;
  const y = (v: number) => 260 - ((v - min) / span) * 220;
  const x = (i: number) => 55 + ((i + 0.5) * 570) / Math.max(values.length, 1);
  const colors = [
    "#235942",
    "#668f76",
    "#abbea5",
    "#7b85ab",
    "#a37d53",
    "#555f70",
  ];
  let angle = -Math.PI / 2;
  const sum = values.reduce((a, b) => a + b, 0);
  return (
    <div className="chart-view">
      {c.composition ? (
        <ComposedChart content={c} />
      ) : (
        <svg
          viewBox="0 0 680 320"
          role="img"
          aria-label={c.caption || "Chart / Gráfica"}
        >
          {c.chart_type === "pie" ? (
            values.map((v, i) => {
              const a = angle;
              angle += (v / sum) * 2 * Math.PI;
              const path = `M330 150 L${330 + 115 * Math.cos(a)} ${150 + 115 * Math.sin(a)} A115 115 0 ${angle - a > Math.PI ? 1 : 0} 1 ${330 + 115 * Math.cos(angle)} ${150 + 115 * Math.sin(angle)} Z`;
              return v === sum ? (
                <circle
                  key={i}
                  cx="330"
                  cy="150"
                  r="115"
                  fill={colors[i % colors.length]}
                >
                  <title>
                    {labels[i]}: {v}
                  </title>
                </circle>
              ) : (
                <path key={i} d={path} fill={colors[i % colors.length]}>
                  <title>
                    {labels[i]}: {v}
                  </title>
                </path>
              );
            })
          ) : (
            <>
              {[0, 0.25, 0.5, 0.75, 1].map((t) => (
                <g key={t}>
                  <line
                    x1="50"
                    y1={y(min + span * t)}
                    x2="640"
                    y2={y(min + span * t)}
                    stroke="#ddd"
                  />
                  <text
                    x="44"
                    y={y(min + span * t) + 4}
                    textAnchor="end"
                    fontSize="11"
                    fill="currentColor"
                  >
                    {Number((min + span * t).toPrecision(3))}
                  </text>
                </g>
              ))}
              {c.chart_type === "line" && (
                <polyline
                  points={values.map((v, i) => `${x(i)},${y(v)}`).join(" ")}
                  fill="none"
                  stroke="#235942"
                  strokeWidth="3"
                />
              )}
              {values.map((v, i) => (
                <g key={i}>
                  {c.chart_type === "bar" ? (
                    <rect
                      x={x(i) - 180 / values.length}
                      y={Math.min(y(v), y(0))}
                      width={360 / values.length}
                      height={Math.max(1, Math.abs(y(v) - y(0)))}
                      fill="#467357"
                    >
                      <title>
                        {labels[i]}: {v}
                      </title>
                    </rect>
                  ) : (
                    <circle cx={x(i)} cy={y(v)} r="4" fill="#235942">
                      <title>
                        {labels[i]}: {v}
                      </title>
                    </circle>
                  )}
                  {(values.length < 15 ||
                    i % Math.ceil(values.length / 12) === 0) && (
                    <text
                      x={x(i)}
                      y="290"
                      textAnchor="middle"
                      fontSize="11"
                      fill="currentColor"
                    >
                      {labels[i].slice(0, 14)}
                    </text>
                  )}
                </g>
              ))}
            </>
          )}
        </svg>
      )}
      <Table
        content={{
          columns: ["Label / Etiqueta", c.unit || "Value / Valor"],
          rows: labels.map((label, i) => [label, values[i]]),
        }}
      />
    </div>
  );
}
function Table({ content }: { content: Content }) {
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(0);
  const [sort, setSort] = useState<{ column: number; asc: boolean } | null>(
    null,
  );
  const rows = useMemo(() => {
    let r = (content.rows || []).filter(
      (r) =>
        !query ||
        r.some((v) =>
          String(v ?? "")
            .toLowerCase()
            .includes(query.toLowerCase()),
        ),
    );
    if (sort)
      r = [...r].sort((a, b) => {
        const x = a[sort.column],
          y = b[sort.column];
        return (
          (typeof x === "number" && typeof y === "number"
            ? x - y
            : String(x ?? "").localeCompare(String(y ?? ""), undefined, {
                numeric: true,
              })) * (sort.asc ? 1 : -1)
        );
      });
    return r;
  }, [content, query, sort]);
  return (
    <div className="data-grid">
      <input
        aria-label="Filter rows / Filtrar filas"
        placeholder="Filter rows / Filtrar filas"
        value={query}
        onChange={(e) => {
          setQuery(e.target.value);
          setPage(0);
        }}
      />
      <div className="table-scroll">
        <table>
          <thead>
            <tr>
              {content.columns?.map((c, i) => (
                <th key={i}>
                  <button
                    onClick={() => {
                      setSort({
                        column: i,
                        asc: sort?.column === i ? !sort.asc : true,
                      });
                      setPage(0);
                    }}
                  >
                    {c}
                    {sort?.column === i ? (sort.asc ? " ↑" : " ↓") : ""}
                  </button>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.slice(page * 50, (page + 1) * 50).map((r, i) => (
              <tr key={i}>
                {r.map((v, j) => (
                  <td key={j}>{v === null ? "—" : String(v)}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="design-row">
        <button disabled={!page} onClick={() => setPage(page - 1)}>
          ←
        </button>
        <span>
          {rows.length ? page * 50 + 1 : 0}–
          {Math.min((page + 1) * 50, rows.length)} / {rows.length}
        </span>
        <button
          disabled={(page + 1) * 50 >= rows.length}
          onClick={() => setPage(page + 1)}
        >
          →
        </button>
      </div>
    </div>
  );
}
function Slides({
  content,
  initialPage = 1,
}: {
  content: Content;
  initialPage?: number;
}) {
  const [index, setIndex] = useState(initialPage - 1);
  useEffect(
    () =>
      setIndex(
        Math.max(
          0,
          Math.min(initialPage - 1, (content.slides?.length || 1) - 1),
        ),
      ),
    [initialPage, content],
  );
  const slides = content.slides || [];
  const slide = slides[Math.min(index, slides.length - 1)];
  if (!slide) return <p>No slides / Sin diapositivas</p>;
  return (
    <div
      className="slides-view"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "ArrowRight")
          setIndex(Math.min(index + 1, slides.length - 1));
        if (e.key === "ArrowLeft") setIndex(Math.max(0, index - 1));
      }}
    >
      {content.composition ? (
        <ComposedSlide
          content={content}
          index={Math.min(index, slides.length - 1)}
        />
      ) : content.theme ? (
        <TemplateSlide
          content={content}
          index={Math.min(index, slides.length - 1)}
        />
      ) : (
        <div
          className="slide"
          style={{
            background: content.brand?.background,
            color: content.brand?.text,
            fontFamily: content.brand?.font,
          }}
        >
          <small>
            {index + 1} / {slides.length}
          </small>
          <h2>{slide.title}</h2>
          <p>{slide.body}</p>
        </div>
      )}
      <div className="design-row">
        <button disabled={!index} onClick={() => setIndex(index - 1)}>
          ←
        </button>
        {slides.map((s, i) => (
          <button
            key={i}
            aria-label={s.title}
            aria-pressed={i === index}
            onClick={() => setIndex(i)}
          >
            {i + 1}
          </button>
        ))}
        <button
          disabled={index === slides.length - 1}
          onClick={() => setIndex(index + 1)}
        >
          →
        </button>
      </div>
      {slide.notes && (
        <details>
          <summary>Notes / Notas</summary>
          <p>{slide.notes}</p>
        </details>
      )}
    </div>
  );
}
export function Viewer({
  visual,
  language = "en",
  initialPage,
}: {
  initialPage?: number;
  visual: Visual;
  language?: string;
}) {
  const c = visual.content;
  if (!c) return <Loading />;
  let view;
  switch (visual.kind) {
    case "web":
      view = <WebPreview content={c} title={visual.title} />;
      break;
    case "diagram":
      view = (
        <DiagramViewer
          content={c}
          title={visual.title}
          revision={visual.revision}
        />
      );
      break;
    case "presentation":
      view = <Slides content={c} initialPage={initialPage} />;
      break;
    case "chart":
      view = <Chart content={c} />;
      break;
    case "spreadsheet":
      view = <Table content={c} />;
      break;
    case "document":
      view = c.asset_id ? (
        <Suspense fallback={<Loading />}>
          <PdfViewer
            key={`${c.asset_id}:${visual.revision}`}
            assetId={c.asset_id}
            initialPage={initialPage}
            title={visual.title}
            language={language}
          />
        </Suspense>
      ) : c.theme ? (
        <TemplateDocument content={c} title={visual.title} />
      ) : (
        <article className="document-preview">
          <h2>{visual.title}</h2>
          <Markdown>{c.text || ""}</Markdown>
        </article>
      );
      break;
    case "image":
      view = (
        <img
          className="image-preview"
          src={
            c.asset_id
              ? assetUrl(c.asset_id)
              : `data:image/svg+xml;charset=utf-8,${encodeURIComponent(c.svg || "")}`
          }
          alt={visual.title}
        />
      );
      break;
    case "video":
      view = (
        <video
          className="video-preview"
          controls
          preload="metadata"
          src={
            c.asset_id
              ? assetUrl(c.asset_id)
              : `/api/design/samples/${encodeURIComponent(c.sample || "")}`
          }
          onError={(e) => {
            e.currentTarget.setAttribute(
              "aria-label",
              "Unsupported video codec / Códec no compatible",
            );
          }}
        />
      );
      break;
  }
  return (
    <>
      {c.caption && !c.theme && <p className="design-note">{c.caption}</p>}
      {c.extraction?.next_offset != null && (
        <p role="status">
          Partial preview / Vista parcial: {c.extraction.next_offset} /{" "}
          {c.extraction.total}
        </p>
      )}
      {c.theme && ["chart", "spreadsheet", "diagram"].includes(visual.kind) ? (
        <TemplateFrame content={c} title={visual.title} kind={visual.kind}>
          {view}
        </TemplateFrame>
      ) : (
        view
      )}
    </>
  );
}
