import { useEffect, useMemo, useState } from "react";
import { api } from "../lib/api";
import { Markdown } from "../components/markdown";
import { Icon } from "../components/icons";
import { navigateTo } from "../components/routing";
import {
  type Flow,
  type Run,
  type Step,
  tr,
  stateLabel,
  stepLabel,
} from "./types";
import "./workflows.css";

const symbols: Record<string, string> = {
  connector: "integrations",
  browser: "browser",
  script: "terminal",
  prompt: "chat",
  condition: "condition",
  wait: "question",
  notify: "notification",
  tool: "skills",
};
export function FlowView({
  flow: initial,
  language,
  compact = false,
  onBack,
  onSchedule,
}: {
  flow: Flow;
  language: string;
  compact?: boolean;
  onBack?: () => void;
  onSchedule?: (flow: Flow) => void;
}) {
  const t = tr(language);
  const [flow, setFlow] = useState(initial),
    [runs, setRuns] = useState<Run[]>([]),
    [runId, setRunId] = useState<string | null>(null);
  const [selected, setSelected] = useState(initial.nodes[0]?.id || ""),
    [zoom, setZoom] = useState(1),
    [tab, setTab] = useState("flow");
  const [error, setError] = useState(""),
    [busy, setBusy] = useState(false),
    [instruction, setInstruction] = useState("");
  const run = runId
    ? runs.find((r) => r.id === runId)
    : runs.find((r) => r.workflow.version === flow.version);
  const displayed = runId && run ? run.workflow : flow;
  const step = displayed.nodes.find((n) => n.id === selected);
  useEffect(() => {
    setFlow(initial);
    setSelected(initial.nodes[0]?.id || "");
    setRunId(null);
  }, [initial]);
  useEffect(() => {
    const controller = new AbortController();
    let busy = false;
    const refresh = async () => {
      if (busy) return;
      busy = true;
      try {
        const [current, history] = await Promise.all([
          api<Flow>(
            `/workflows/${initial.id}`,
            undefined,
            "GET",
            controller.signal,
          ),
          api<Run[]>(
            `/workflows/${initial.id}/runs`,
            undefined,
            "GET",
            controller.signal,
          ),
        ]);
        if (!controller.signal.aborted) {
          setFlow(current);
          setRuns(history);
        }
      } catch (e) {
        if (!controller.signal.aborted) setError(String(e));
      } finally {
        busy = false;
      }
    };
    void refresh();
    const timer = window.setInterval(refresh, 2500);
    return () => {
      controller.abort();
      window.clearInterval(timer);
    };
  }, [initial.id]);
  useEffect(() => {
    if (!compact) document.documentElement.classList.add("workflow-open");
    return () => {
      if (!compact) document.documentElement.classList.remove("workflow-open");
    };
  }, [compact]);
  const layout = useMemo(() => {
    const levels = new Map<string, number>();
    for (let pass = 0; pass < displayed.nodes.length; pass++)
      for (const node of displayed.nodes)
        if (node.depends_on.every((id) => levels.has(id)))
          levels.set(
            node.id,
            Math.max(-1, ...node.depends_on.map((id) => levels.get(id)!)) + 1,
          );
    const counts = new Map<number, number>();
    const positions = new Map<string, { x: number; y: number }>();
    for (const node of displayed.nodes) {
      const level = levels.get(node.id) || 0,
        row = counts.get(level) || 0;
      counts.set(level, row + 1);
      positions.set(node.id, { x: 38 + level * 278, y: 65 + row * 156 });
    }
    return {
      positions,
      width: Math.max(750, 100 + (Math.max(0, ...levels.values()) + 1) * 278),
      height: Math.max(390, 100 + Math.max(1, ...counts.values()) * 156),
    };
  }, [displayed]);
  async function act(path: string, body: unknown = {}) {
    setBusy(true);
    setError("");
    try {
      const run = await api<Run>(path, body);
      setRunId(run.id);
      setRuns((old) => [run, ...old.filter((r) => r.id !== run.id)]);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  async function edit() {
    if (!instruction.trim()) return;
    setBusy(true);
    try {
      await api("/messages", {
        text: `Edit workflow ${flow.id}, current version ${flow.version}. ${instruction}`,
      });
      setInstruction("");
      navigateTo("chat");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className={`flow-view ${compact ? "is-compact" : ""}`}>
      <header className="flow-heading">
        <div>
          {onBack && (
            <button className="subtle" onClick={onBack}>
              ← {t("Scheduled tasks", "Tareas programadas")}
            </button>
          )}
          <div className="eyebrow">
            {t("Workflow", "Flujo")} · v{displayed.version}
          </div>
          <h1>{displayed.name}</h1>
          <p className="muted">{displayed.description}</p>
        </div>
        <div className="flow-actions">
          <button
            disabled={busy}
            onClick={() =>
              void act(`/workflows/${flow.id}/run`, { test_mode: true })
            }
          >
            {t("Preview without effects", "Probar sin efectos")}
          </button>
          <button
            className="primary"
            disabled={busy}
            onClick={() =>
              void act(`/workflows/${flow.id}/run`, { test_mode: false })
            }
          >
            {t("Run", "Ejecutar")}
          </button>
          {onSchedule && (
            <button onClick={() => onSchedule(flow)}>
              {t("Schedule", "Programar")}
            </button>
          )}
        </div>
      </header>
      {error && (
        <p role="alert" className="flow-error">
          {error}
        </p>
      )}
      <nav
        className="flow-tabs"
        aria-label={t("Workflow views", "Vistas del flujo")}
      >
        <button aria-pressed={tab === "flow"} onClick={() => setTab("flow")}>
          {t("Flow", "Flujo")}
        </button>
        <button
          aria-pressed={tab === "history"}
          onClick={() => setTab("history")}
        >
          {t("Executions", "Ejecuciones")} <span>{runs.length}</span>
        </button>
        {runId && (
          <button onClick={() => setRunId(null)}>
            {t("Back to current version", "Ver versión actual")}
          </button>
        )}
      </nav>
      {tab === "history" ? (
        <div className="flow-history">
          {!runs.length && (
            <p className="muted">
              {t(
                "No executions yet. Preview the flow to start.",
                "Todavía no hay ejecuciones. Comienza probando el flujo.",
              )}
            </p>
          )}
          {runs.map((item) => (
            <button
              className="flow-history-row"
              key={item.id}
              onClick={() => {
                setRunId(item.id);
                setTab("flow");
              }}
            >
              <span className={`flow-state ${item.status}`}>
                {stateLabel(item.status, language)}
              </span>
              <span>
                {new Date(item.created_at).toLocaleString(language, {
                  timeZone: "America/Lima",
                })}
              </span>
              <span>
                v{item.workflow.version}{" "}
                {item.test_mode ? t("· preview", "· prueba") : ""}
              </span>
              <span>↗</span>
            </button>
          ))}
        </div>
      ) : (
        <div className="flow-body">
          <div className="flow-canvas-wrap">
            <div
              className="flow-canvas"
              role="region"
              aria-label={t("Workflow diagram", "Diagrama del flujo")}
            >
              <div
                style={{
                  width: layout.width * zoom,
                  height: layout.height * zoom,
                }}
              >
                <div
                  style={{
                    position: "relative",
                    width: layout.width,
                    height: layout.height,
                    transform: `scale(${zoom})`,
                    transformOrigin: "0 0",
                  }}
                >
                  <svg
                    width={layout.width}
                    height={layout.height}
                    aria-hidden="true"
                    className="flow-edges"
                  >
                    <defs>
                      <marker
                        id={`arrow-${compact ? "panel" : "page"}`}
                        markerWidth="8"
                        markerHeight="8"
                        refX="7"
                        refY="4"
                        orient="auto"
                      >
                        <path d="M0,0 L8,4 L0,8 Z" fill="currentColor" />
                      </marker>
                    </defs>
                    {displayed.nodes.flatMap((node) =>
                      node.depends_on.map((parent) => {
                        const from = layout.positions.get(parent),
                          to = layout.positions.get(node.id);
                        if (!from || !to) return null;
                        const a = from.x + 232,
                          b = to.x,
                          center = (a + b) / 2;
                        return (
                          <path
                            key={`${parent}-${node.id}`}
                            d={`M${a},${from.y + 44} H${center} V${to.y + 44} H${b - 5}`}
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="1.6"
                            markerEnd={`url(#arrow-${compact ? "panel" : "page"})`}
                          />
                        );
                      }),
                    )}
                  </svg>
                  {displayed.nodes.map((node) => {
                    const p = layout.positions.get(node.id)!;
                    const status = run?.nodes[node.id]?.status || "pending";
                    return (
                      <button
                        key={node.id}
                        className={`flow-node ${status} ${selected === node.id ? "selected" : ""}`}
                        style={{ left: p.x, top: p.y }}
                        onClick={() => setSelected(node.id)}
                        aria-pressed={selected === node.id}
                      >
                        <span className="flow-node-symbol">
                          <Icon name={symbols[node.kind] || "skills"} />
                        </span>
                        <span>
                          <strong>{node.label}</strong>
                          <small>
                            {node.kind === "browser"
                              ? `${node.steps?.length || 0} ${t("browser steps", "pasos de navegador")}`
                              : stepLabel(node.kind, language)}
                          </small>
                          <span className="flow-node-state">
                            {stateLabel(status, language)}
                          </span>
                        </span>
                        {node.when && (
                          <span
                            title={t("Conditional step", "Paso condicional")}
                          >
                            ◇
                          </span>
                        )}
                      </button>
                    );
                  })}
                </div>
              </div>
            </div>
            <div className="flow-zoom">
              <button
                aria-label={t("Zoom out", "Alejar")}
                onClick={() => setZoom((z) => Math.max(0.35, z - 0.15))}
              >
                −
              </button>
              <button onClick={() => setZoom(1)}>
                {Math.round(zoom * 100)}%
              </button>
              <button
                aria-label={t("Zoom in", "Acercar")}
                onClick={() => setZoom((z) => Math.min(2, z + 0.15))}
              >
                +
              </button>
            </div>
          </div>
          <aside className="flow-inspector">
            {step ? (
              <StepDetails node={step} run={run} language={language} />
            ) : (
              <p>{t("Select a step", "Selecciona un paso")}</p>
            )}
          </aside>
        </div>
      )}
      {run && (
        <div className="flow-run-strip">
          <span className={`flow-state ${run.status}`}>
            {stateLabel(run.status, language)}
          </span>
          <small>
            v{run.workflow.version} · {run.id.slice(0, 8)}
          </small>
          {["paused", "failed", "interrupted"].includes(run.status) && (
            <button
              disabled={busy}
              onClick={() => void act(`/workflow-runs/${run.id}/resume`)}
            >
              {t("Continue saved execution", "Continuar ejecución guardada")}
            </button>
          )}
          {[
            "queued",
            "running",
            "waiting",
            "paused",
            "failed",
            "interrupted",
          ].includes(run.status) && (
            <button
              disabled={busy}
              onClick={() => void act(`/workflow-runs/${run.id}/cancel`)}
            >
              {["paused", "failed", "interrupted"].includes(run.status)
                ? t("Close saved execution", "Cerrar ejecución guardada")
                : t("Stop execution", "Detener ejecución")}
            </button>
          )}
          {run.error && <p>{run.error}</p>}
        </div>
      )}
      <form
        className="flow-composer"
        onSubmit={(e) => {
          e.preventDefault();
          void edit();
        }}
      >
        <span>✦</span>
        <input
          value={instruction}
          onChange={(e) => setInstruction(e.target.value)}
          placeholder={t(
            "Ask Fénix to change this workflow…",
            "Pide a Fénix que modifique este flujo…",
          )}
          aria-label={t("Workflow change", "Cambio del flujo")}
        />
        <button className="primary" disabled={busy || !instruction.trim()}>
          {t("Send", "Enviar")} ↗
        </button>
      </form>
    </section>
  );
}
function StepDetails({
  node,
  run,
  language,
}: {
  node: Step;
  run?: Run;
  language: string;
}) {
  const t = tr(language),
    result = run?.nodes[node.id];
  return (
    <>
      <div className="eyebrow">{t("Step details", "Detalle del paso")}</div>
      <h2>{node.label}</h2>
      <p className="muted">
        {node.kind === "browser"
          ? t(
              "Fénix browser · reusable actions",
              "Navegador de Fénix · acciones reutilizables",
            )
          : stepLabel(node.kind, language)}
      </p>
      {node.when && (
        <p className="flow-condition">
          ◇ {node.when.reference} · {node.when.operator} ·{" "}
          {JSON.stringify(node.when.value)}
        </p>
      )}
      {node.steps?.length ? (
        <ol className="flow-browser-steps">
          {node.steps.map((action) => (
            <li key={action.id}>
              <strong>{action.id}</strong>
              <small>{action.tool}</small>
            </li>
          ))}
        </ol>
      ) : null}
      {typeof node.arguments.prompt === "string" && (
        <Markdown>{node.arguments.prompt}</Markdown>
      )}
      {typeof node.arguments.question === "string" && (
        <Markdown>{node.arguments.question}</Markdown>
      )}
      <details>
        <summary>
          {t("Inputs and data connections", "Entradas y conexiones de datos")}
        </summary>
        <pre>{JSON.stringify(node.arguments, null, 2)}</pre>
      </details>
      {result && (
        <>
          <p className={`flow-state ${result.status}`}>
            {stateLabel(result.status, language)}
          </p>
          {result.error && <p role="alert">{result.error}</p>}
          {result.output !== null && (
            <details open>
              <summary>{t("Result", "Resultado")}</summary>
              <pre>{JSON.stringify(result.output, null, 2)}</pre>
            </details>
          )}
        </>
      )}
    </>
  );
}
