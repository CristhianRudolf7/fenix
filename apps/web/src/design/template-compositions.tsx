import type { Content } from "./types";
import { templateStyle } from "./template-visuals";
import "./template-compositions.css";

/** Original, local vector scenes. No downloaded artwork or external rendering. */
export function CompositionScene({ family }: { family: string }) {
  return (
    <svg
      viewBox={family === "product-stage" ? "0 0 600 260" : "0 0 600 400"}
      preserveAspectRatio={
        family === "culture-journal" ? "xMidYMid slice" : "xMidYMid meet"
      }
      className="composition-scene"
      aria-hidden="true"
    >
      <g transform={family === "product-stage" ? "scale(1 .65)" : undefined}>
        {family === "research-atlas" ? (
          <>
            <g fill="none" stroke="currentColor" strokeWidth="1.5">
              {Array.from({ length: 16 }, (_, i) => (
                <path
                  key={i}
                  d={`M${20 + i * 8} 360 C${-40 + i * 15} ${190 - i * 5},${470 - i * 9} ${300 - i * 13},${250 + i * 16} 30`}
                />
              ))}
            </g>
            <path
              d="M100 305L235 205L400 245L480 100"
              fill="none"
              stroke="currentColor"
              strokeWidth="3"
            />
            {[
              [100, 305],
              [235, 205],
              [400, 245],
              [480, 100],
            ].map(([x, y], i) => (
              <g key={i}>
                <circle cx={x} cy={y} r="8" fill="currentColor" />
                <text
                  x={x + 16}
                  y={y - 10}
                  fill="currentColor"
                  fontSize="13"
                  fontFamily="monospace"
                >{`0${i + 1}`}</text>
              </g>
            ))}
          </>
        ) : family === "venture-stack" ? (
          <g transform="translate(50 5)">
            {[0, 1, 2, 3].map((i) => (
              <g key={i} transform={`translate(${i * 18} ${250 - i * 65})`}>
                <path
                  d="M0 50L220 0L440 85L220 140Z"
                  fill={i % 2 ? "var(--composition-paper)" : "currentColor"}
                />
                <path
                  d="M0 50V73L220 163V140M220 163L440 108V85"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                />
              </g>
            ))}
          </g>
        ) : family === "business-ledger" ? (
          <>
            {[0, 1, 2, 3].map((i) => (
              <g key={i}>
                <rect
                  x={30 + i * 140}
                  y={230 - i * 50}
                  width="110"
                  height={130 + i * 50}
                  fill="currentColor"
                  opacity={0.22 + i * 0.23}
                />
                <text
                  x={35 + i * 140}
                  y={210 - i * 50}
                  fill="currentColor"
                  fontSize="22"
                >{`0${i + 1}`}</text>
              </g>
            ))}
            <path d="M20 365H585" stroke="currentColor" />
          </>
        ) : family === "workshop-notes" ? (
          <>
            {[
              [60, 20, -8],
              [255, 65, 8],
              [130, 205, -3],
            ].map(([x, y, r], i) => (
              <g key={i} transform={`translate(${x} ${y}) rotate(${r})`}>
                <rect
                  width="210"
                  height="155"
                  fill={i === 1 ? "currentColor" : "var(--composition-paper)"}
                  stroke="currentColor"
                  strokeWidth="2"
                />
                <path
                  d="M25 35H160M25 65H120M25 95H155"
                  stroke={i === 1 ? "var(--composition-paper)" : "currentColor"}
                  strokeWidth="6"
                />
                <circle cx="105" cy="4" r="9" fill="var(--composition-ink)" />
              </g>
            ))}
          </>
        ) : family === "product-stage" ? (
          <>
            <rect
              x="15"
              y="25"
              width="570"
              height="350"
              rx="15"
              fill="var(--composition-paper)"
              stroke="currentColor"
              strokeWidth="2"
            />
            <path
              d="M15 70H585M160 70V375"
              stroke="currentColor"
              opacity=".3"
            />
            {[40, 60, 80].map((x) => (
              <circle key={x} cx={x} cy="48" r="5" fill="currentColor" />
            ))}
            <g fill="currentColor">
              <rect x="35" y="100" width="100" height="12" rx="6" />
              <rect x="35" y="140" width="65" height="8" rx="4" opacity=".4" />
              <rect x="35" y="170" width="80" height="8" rx="4" opacity=".4" />
              {[0, 1, 2].map((i) => (
                <g key={i}>
                  <rect
                    x={188 + i * 123}
                    y="106"
                    width="104"
                    height="65"
                    rx="7"
                    opacity={0.1 + i * 0.1}
                  />
                  <rect
                    x={200 + i * 123}
                    y={280 - i * 40}
                    width="76"
                    height={50 + i * 40}
                    rx="4"
                  />
                </g>
              ))}
            </g>
            <path
              d="M186 211H556M186 250H556M186 290H556"
              stroke="currentColor"
              opacity=".15"
            />
          </>
        ) : family === "studio-arch" ? (
          <g fill="none" stroke="currentColor">
            {[0, 1, 2, 3, 4, 5].map((i) => (
              <path
                key={i}
                d={`M${100 + i * 22} 365V${160 + i * 8}a${200 - i * 22} ${160 - i * 18} 0 01${400 - i * 44} 0v${205 - i * 8}`}
                strokeWidth={i === 0 ? 5 : 1.5}
              />
            ))}
            <path d="M45 365H590M300 5V400M80 80H520" strokeDasharray="4 8" />
            <circle cx="300" cy="230" r="68" strokeWidth="2" />
          </g>
        ) : family === "type-poster" ? (
          <g fill="currentColor">
            {[0, 1, 2, 3, 4, 5, 6].map((i) => (
              <rect
                key={i}
                x={20 + i * 83}
                y="30"
                width="42"
                height="340"
                transform="skewX(-15)"
              />
            ))}
          </g>
        ) : family === "welcome-collage" ? (
          <>
            {[
              [25, 20],
              [225, 65],
              [410, 10],
            ].map(([x, y], i) => (
              <g
                key={i}
                transform={`translate(${x} ${y}) rotate(${i === 1 ? 8 : -5} 80 120)`}
              >
                <rect
                  width="155"
                  height="275"
                  rx="80"
                  fill={i === 1 ? "var(--composition-paper)" : "currentColor"}
                />
                <circle
                  cx="78"
                  cy="90"
                  r="46"
                  fill={i === 1 ? "currentColor" : "var(--composition-paper)"}
                />
                <path
                  d="M30 205Q78 150 125 205"
                  fill="none"
                  stroke={i === 1 ? "currentColor" : "var(--composition-paper)"}
                  strokeWidth="30"
                />
              </g>
            ))}
          </>
        ) : family === "systems-blueprint" ? (
          <g fill="none" stroke="currentColor" strokeWidth="2">
            {[
              [70, 80],
              [270, 160],
              [430, 65],
            ].map(([x, y], i) => (
              <g key={i} transform={`translate(${x} ${y})`}>
                <path d="M0 30L65 0L130 30V135L65 170L0 135ZM0 30L65 65L130 30M65 65V170M0 65L65 100L130 65M0 100L65 135L130 100" />
                <circle cx="25" cy="88" r="4" fill="currentColor" />
              </g>
            ))}
            <path d="M135 250V340H335V330M495 235V365H335V340" />
            <circle cx="335" cy="340" r="7" fill="currentColor" />
          </g>
        ) : (
          <>
            <rect width="600" height="400" fill="#e3cfae" />
            <circle cx="450" cy="110" r="65" fill="#be4e33" />
            <path
              d="M0 275L170 90L365 300L500 145L600 265V400H0Z"
              fill="#b59879"
            />
            <path
              d="M0 330L225 210L340 340L475 260L600 345V400H0Z"
              fill="#687363"
            />
            <path d="M0 365Q250 275 600 355V400H0Z" fill="#293f38" />
          </>
        )}
      </g>
    </svg>
  );
}

export function ComposedSlide({
  content,
  index,
}: {
  content: Content;
  index: number;
}) {
  const slide = content.slides?.[index];
  if (!slide) return null;
  const family = content.composition || "research-atlas";
  const lines = slide.body.split(/\n+/).filter(Boolean);
  return (
    <article
      className={`slide designed-slide composed-slide composition-${family} scene-${slide.layout || "cover"}`}
      style={templateStyle(content)}
    >
      <div className="composition-kicker">
        <span>{slide.eyebrow || content.eyebrow}</span>
        <span>
          {String(index + 1).padStart(2, "0")} /{" "}
          {String(content.slides?.length).padStart(2, "0")}
        </span>
      </div>
      <div className="composition-heading">
        <h2>{slide.title}</h2>
      </div>
      <div className="composition-copy">
        {slide.layout === "steps" ? (
          <ol>
            {lines.map((line, i) => (
              <li key={i}>
                <b>{String(i + 1).padStart(2, "0")}</b>
                <p>{line}</p>
              </li>
            ))}
          </ol>
        ) : (
          lines.map((line, i) => <p key={i}>{line}</p>)
        )}
      </div>
      <div className="composition-visual">
        <CompositionScene family={family} />
      </div>
      <div className="composition-folio">
        <span>{content.display_title}</span>
        <span>FÉNIX / ORIGINAL</span>
      </div>
    </article>
  );
}

/** All data marks come from the editable values; no decorative invented metrics. */
export function ComposedChart({ content: c }: { content: Content }) {
  const values = c.values || [],
    labels = c.labels || [];
  const max = Math.max(1, ...values.map(Math.abs));
  const sum = values.reduce((a, b) => a + b, 0);
  const min = Math.min(0, ...values),
    high = Math.max(1, ...values);
  const px = (i: number) => 70 + (i * 530) / Math.max(1, values.length - 1);
  const py = (v: number) => 255 - ((v - min) / (high - min)) * 195;
  const palette = ["#205acc", "#ed7845", "#389585", "#b793da", "#e5b640"];
  const name = c.composition;
  let offset = 0;
  return (
    <svg
      className={`composed-chart chart-${name}`}
      viewBox="0 0 680 330"
      role="img"
      aria-label={c.caption || "Chart / Gráfica"}
    >
      <rect width="680" height="330" fill="white" />
      {name === "expense-ring" ? (
        <>
          {values.map((v, i) => {
            const start = offset;
            offset += (v / sum) * 100;
            return (
              <circle
                key={i}
                cx="210"
                cy="155"
                r="100"
                fill="none"
                stroke={palette[i % 5]}
                strokeWidth="42"
                pathLength="100"
                strokeDasharray={`${(v / sum) * 100} 100`}
                strokeDashoffset={-start}
                transform="rotate(-90 210 155)"
              >
                <title>
                  {labels[i]}: {v}
                </title>
              </circle>
            );
          })}
          <text
            x="210"
            y="153"
            textAnchor="middle"
            fontSize="34"
            fontWeight="700"
          >
            {sum}
          </text>
          <text x="210" y="176" textAnchor="middle" fontSize="12">
            TOTAL
          </text>
          {values.map((v, i) => (
            <g key={i}>
              <circle cx="395" cy={80 + i * 48} r="5" fill={palette[i % 5]} />
              <text x="411" y={85 + i * 48} fontSize="15">
                {labels[i]}
              </text>
              <text x="624" y={85 + i * 48} fontSize="17" textAnchor="end">
                {v}
              </text>
            </g>
          ))}
        </>
      ) : name === "channel-stack" ? (
        <>
          {values.map((v, i) => {
            const start = offset;
            offset += (v / sum) * 560;
            return (
              <g key={i}>
                <rect
                  x={60 + start}
                  y="65"
                  width={(v / sum) * 560}
                  height="100"
                  fill={palette[i % 5]}
                />
                <text
                  x={60 + start + (v / sum) * 280}
                  y="123"
                  fill="white"
                  textAnchor="middle"
                  fontSize="23"
                >
                  {Math.round((v / sum) * 100)}%
                </text>
                <circle
                  cx={85 + i * 195}
                  cy="220"
                  r="5"
                  fill={palette[i % 5]}
                />
                <text x={100 + i * 195} y="225" fontSize="14">
                  {labels[i]}
                </text>
                <text x={100 + i * 195} y="264" fontSize="27" fontWeight="700">
                  {v}
                </text>
              </g>
            );
          })}
        </>
      ) : name === "survey-rank" || name === "cash-balance" ? (
        <>
          {values.map((v, i) => {
            const y = 40 + (i * 230) / Math.max(values.length, 1);
            const signed =
              name === "cash-balance" || values.some((value) => value < 0);
            const origin = signed ? 340 : 175;
            const width = (Math.abs(v) / max) * (signed ? 240 : 420);
            return (
              <g key={i}>
                <text x="24" y={y + 25} fontSize="15">
                  {labels[i]}
                </text>
                <line
                  x1={origin}
                  y1={y - 5}
                  x2={origin}
                  y2={y + 46}
                  stroke="#cbd1d9"
                />
                <rect
                  x={v < 0 ? origin - width : origin}
                  y={y}
                  width={width}
                  height="35"
                  rx={signed ? 0 : 5}
                  fill={v < 0 ? "#e27751" : palette[i % 5]}
                />
                <text
                  x={v < 0 ? origin - width - 10 : origin + width + 10}
                  y={y + 24}
                  textAnchor={v < 0 ? "end" : "start"}
                  fontSize="14"
                >
                  {v}
                </text>
              </g>
            );
          })}
        </>
      ) : (
        <>
          {[0, 0.25, 0.5, 0.75, 1].map((t) => (
            <g key={t}>
              <line
                x1="50"
                y1={60 + t * 195}
                x2="630"
                y2={60 + t * 195}
                stroke="#e9edf1"
              />
              <text
                x="40"
                y={65 + t * 195}
                textAnchor="end"
                fontSize="11"
                fill="#6c7684"
              >
                {Number((high - t * (high - min)).toFixed(1))}
              </text>
            </g>
          ))}
          {(name === "growth-area" || name === "latency-step") && (
            <>
              {name === "growth-area" && (
                <path
                  d={`M70 ${py(0)} ${values.map((v, i) => `L${px(i)} ${py(v)}`).join(" ")} L${px(values.length - 1)} ${py(0)}Z`}
                  fill="#e1ecff"
                />
              )}
              <path
                d={values
                  .map((v, i) =>
                    i === 0
                      ? `M${px(i)} ${py(v)}`
                      : name === "latency-step"
                        ? `H${px(i)}V${py(v)}`
                        : `L${px(i)} ${py(v)}`,
                  )
                  .join(" ")}
                fill="none"
                stroke="#205acc"
                strokeWidth="3"
              />
            </>
          )}
          {values.map((v, i) => {
            const x =
              name === "weekly-columns"
                ? 90 + (i * 490) / Math.max(values.length - 1, 1)
                : px(i);
            return (
              <g key={i}>
                {name === "weekly-columns" ? (
                  <rect
                    x={x - 26}
                    y={Math.min(py(v), py(0))}
                    width="52"
                    height={Math.abs(py(0) - py(v))}
                    rx="5"
                    fill={
                      i === values.indexOf(Math.max(...values))
                        ? "#ed7845"
                        : "#205acc"
                    }
                  />
                ) : (
                  <>
                    {name === "experiment-dots" && (
                      <line
                        x1={x}
                        y1={py(0)}
                        x2={x}
                        y2={py(v)}
                        stroke={palette[i % 5]}
                        strokeWidth="7"
                      />
                    )}
                    <circle
                      cx={x}
                      cy={py(v)}
                      r={name === "experiment-dots" ? 11 : 5}
                      fill={palette[i % 5]}
                    />
                  </>
                )}
                <text
                  x={x}
                  y={py(v) - 17}
                  fontSize="15"
                  fontWeight="700"
                  textAnchor="middle"
                >
                  {v}
                </text>
                <text x={x} y="286" fontSize="12" textAnchor="middle">
                  {labels[i]?.slice(0, 16)}
                </text>
              </g>
            );
          })}
        </>
      )}
    </svg>
  );
}
