import type { CSSProperties, ReactNode } from "react";
import type { Content } from "./types";
import "./template-visuals.css";

export const templateThemes = {
  midnight: ["#181b30", "#f6f3ff", "#b9a0ff", "#292d49"],
  cobalt: ["#1535cd", "#ffffff", "#d4ff74", "#2949df"],
  coral: ["#ffe6da", "#3c2425", "#c34930", "#f9c9b7"],
  forest: ["#173d33", "#f2f0dc", "#d5e8a0", "#285044"],
  plum: ["#482747", "#fff0e7", "#ffab8e", "#613858"],
  sand: ["#f0e8d8", "#352b23", "#a44e2c", "#e4d8bf"],
  mint: ["#dff2e7", "#153d35", "#246b55", "#c6e3d1"],
  ink: ["#f4f2ed", "#222630", "#465cbb", "#e3e4eb"],
} as const;

export function templateStyle(content: Content): CSSProperties {
  const colors = templateThemes[content.theme || "ink"] || templateThemes.ink;
  return {
    "--template-bg": content.brand?.background || colors[0],
    "--template-fg": content.brand?.text || colors[1],
    "--template-accent": content.brand?.accent || colors[2],
    "--template-soft": colors[3],
    background: content.brand?.background,
    color: content.brand?.text,
    fontFamily: content.brand?.font,
  } as CSSProperties;
}

/** Original vector artwork. It carries no invented business metrics. */
export function TemplateArtwork({ variant = 0 }: { variant?: number }) {
  return (
    <svg className="template-artwork" viewBox="0 0 400 400" aria-hidden="true">
      {variant % 3 === 0 ? (
        <>
          {[150, 115, 80, 45].map((r) => (
            <circle
              key={r}
              cx="200"
              cy="200"
              r={r}
              fill="none"
              stroke="currentColor"
              strokeWidth="24"
            />
          ))}
          <path
            d="M0 200H400M200 0V400"
            stroke="var(--template-bg)"
            strokeWidth="12"
          />
          <circle cx="306" cy="94" r="28" fill="var(--template-fg)" />
        </>
      ) : variant % 3 === 1 ? (
        <>
          {[0, 1, 2, 3, 4].map((i) => (
            <rect
              key={i}
              x={35 + i * 52}
              y={145 - i * 24}
              width="66"
              height={170 + i * 12}
              rx="32"
              fill="currentColor"
              opacity={0.25 + i * 0.17}
              transform="rotate(25 200 200)"
            />
          ))}
          <circle cx="100" cy="75" r="27" fill="var(--template-fg)" />
        </>
      ) : (
        <>
          <path d="M50 350V180a150 150 0 01300 0v170Z" fill="currentColor" />
          <path
            d="M110 350V185a90 90 0 01180 0v165Z"
            fill="var(--template-bg)"
          />
          <circle cx="200" cy="242" r="56" fill="var(--template-fg)" />
          <path d="M20 355H380" stroke="var(--template-fg)" strokeWidth="2" />
        </>
      )}
    </svg>
  );
}

export function TemplateSlide({
  content,
  index,
}: {
  content: Content;
  index: number;
}) {
  const slides = content.slides || [];
  const slide = slides[index];
  if (!slide) return null;
  const layout = slide.layout || (index === 0 ? "cover" : "split");
  const paragraphs = slide.body.split(/\n+/).filter(Boolean);
  return (
    <div
      className={`slide designed-slide layout-${layout}`}
      style={templateStyle(content)}
    >
      <div className="template-slide-top">
        <span>{slide.eyebrow || content.eyebrow || "FÉNIX STUDIO"}</span>
        <span>
          {String(index + 1).padStart(2, "0")} /{" "}
          {String(slides.length).padStart(2, "0")}
        </span>
      </div>
      <div className="template-slide-main">
        <div className="template-slide-copy">
          <h2>{slide.title}</h2>
          {layout === "steps" ? (
            <ol>
              {paragraphs.map((line, i) => (
                <li key={i}>
                  <span>{String(i + 1).padStart(2, "0")}</span>
                  <p>{line}</p>
                </li>
              ))}
            </ol>
          ) : (
            <div className="template-slide-body">
              {paragraphs.map((line, i) => (
                <p key={i}>{line}</p>
              ))}
            </div>
          )}
        </div>
        {(layout === "cover" || layout === "split") && (
          <div className="template-slide-art">
            <TemplateArtwork
              variant={
                index +
                Object.keys(templateThemes).indexOf(content.theme || "ink")
              }
            />
          </div>
        )}
        {layout === "statement" && (
          <span className="template-quote-mark" aria-hidden="true">
            “
          </span>
        )}
      </div>
      <div className="template-slide-bottom">
        <span>{content.display_title || "FÉNIX · ORIGINAL"}</span>
        <span aria-hidden="true">↗</span>
      </div>
    </div>
  );
}

export function TemplateFrame({
  content,
  title,
  kind,
  children,
}: {
  content: Content;
  title: string;
  kind: string;
  children: ReactNode;
}) {
  return (
    <section
      className={`template-frame template-${kind} ${content.composition ? `frame-${content.composition}` : ""}`}
      style={templateStyle(content)}
    >
      <header className="template-frame-heading">
        <div>
          <span>{content.eyebrow || "FÉNIX STUDIO"}</span>
          <h2>{content.display_title || title}</h2>
        </div>
        <div className="template-frame-emblem" aria-hidden="true">
          ✳
        </div>
      </header>
      <div className="template-frame-content">{children}</div>
      <footer>
        <span>
          {content.caption || "Original template · Plantilla original"}
        </span>
        <span>F /</span>
      </footer>
    </section>
  );
}

export function TemplateDocument({
  content,
  title,
}: {
  content: Content;
  title: string;
}) {
  const blocks = (content.text || "").split(/\n\s*\n/).filter(Boolean);
  // The source text remains the only authority: edits and text exports stay aligned.
  const heading = blocks[0] || title;
  return (
    <article
      className={`template-document document-${content.composition || "classic"}`}
      style={templateStyle(content)}
    >
      <header>
        <span>{content.eyebrow || "FÉNIX · FIELD NOTES"}</span>
        <span>01 /</span>
      </header>
      <h2>{heading}</h2>
      <div className="template-document-rule" />
      {blocks.slice(1).map((block, i) => {
        const [heading, ...lines] = block.split("\n");
        return (
          <section key={i}>
            <span className="template-section-number">
              {String(i + 1).padStart(2, "0")}
            </span>
            <div>
              <h3>{heading}</h3>
              {lines.map((line, j) => (
                <p key={j}>{line}</p>
              ))}
            </div>
          </section>
        );
      })}
      <footer>
        FÉNIX STUDIO <span>Sample content · Contenido de ejemplo</span>
      </footer>
    </article>
  );
}
