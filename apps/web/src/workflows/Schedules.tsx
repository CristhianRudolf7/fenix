import { Loading } from "../components/loading";
import { useEffect, useState } from "react";
import { createRoot, type Root } from "react-dom/client";
import { api } from "../lib/api";
import { navigateTo } from "../components/routing";
import { FlowView } from "./FlowView";
import { type Flow, type Schedule, stateLabel, tr } from "./types";
import "./workflows.css";

function Schedules({ language }: { language: string }) {
  const [initialLoading, setInitialLoading] = useState(true);
  const t = tr(language);
  const [schedules, setSchedules] = useState<Schedule[]>([]),
    [flows, setFlows] = useState<Flow[]>([]),
    [selected, setSelected] = useState<Flow | null>(null);
  const [query, setQuery] = useState(""),
    [filter, setFilter] = useState("all"),
    [editing, setEditing] = useState<Schedule | null>(null),
    [create, setCreate] = useState(false);
  const [instruction, setInstruction] = useState(""),
    [busy, setBusy] = useState(false),
    [error, setError] = useState("");
  async function refresh(signal?: AbortSignal) {
    const [s, f] = await Promise.all([
      api<Schedule[]>("/schedules", undefined, "GET", signal),
      api<Flow[]>("/workflows", undefined, "GET", signal),
    ]);
    if (!signal?.aborted) {
      setSchedules(s);
      setFlows(f);
    }
  }
  useEffect(() => {
    const controller = new AbortController();
    void refresh(controller.signal)
      .catch((e) => {
        if (!controller.signal.aborted) setError(String(e));
      })
      .finally(() => {
        if (!controller.signal.aborted) setInitialLoading(false);
      });
    return () => controller.abort();
  }, []);
  async function act(schedule: Schedule) {
    setBusy(true);
    try {
      await api("/schedules", {
        action: schedule.status === "active" ? "pause" : "resume",
        id: schedule.id,
      });
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  async function describe() {
    if (!instruction.trim()) return;
    setBusy(true);
    setError("");
    try {
      await api("/messages", {
        text: `Create a reusable workflow with workflow_manage. Clarify missing connections, file-selection rules, destination or schedule. Do not invent these details. Request: ${instruction}`,
      });
      navigateTo("chat");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  const matches = (name: string) =>
    name.toLocaleLowerCase().includes(query.toLocaleLowerCase());
  function scheduleFlow(flow: Flow) {
    setEditing({
      name: flow.name,
      content: flow.description || flow.name,
      mode: "workflow",
      kind: "cron",
      cron: "0 8 * * 1",
      workflow_id: flow.id,
      project_id: flow.project_id,
    });
    setCreate(true);
  }
  if (initialLoading) return <Loading />;
  return (
    <section className="schedules-page">
      {selected ? (
        <FlowView
          flow={selected}
          language={language}
          onBack={() => setSelected(null)}
          onSchedule={scheduleFlow}
        />
      ) : (
        <>
          <header className="schedules-heading">
            <div>
              <div className="eyebrow">
                {t("Your assistant, on time", "Tu asistente, a tiempo")}
              </div>
              <h1>{t("Scheduled tasks", "Tareas programadas")}</h1>
              <p className="muted">
                {t(
                  "Prompts and workflows, in one place. Fénix must be running.",
                  "Prompts y flujos, en un solo lugar. Fénix debe estar encendido.",
                )}
              </p>
            </div>
            <button
              className="primary"
              onClick={() => {
                setEditing(null);
                setCreate(true);
              }}
            >
              + {t("New task", "Nueva tarea")}
            </button>
          </header>
          <div className="schedules-toolbar">
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t("Search tasks…", "Buscar tareas…")}
              aria-label={t("Search tasks", "Buscar tareas")}
            />
            <div className="flow-tabs">
              {[
                ["all", t("All", "Todas")],
                ["prompts", "Prompts"],
                ["flows", t("Flows", "Flujos")],
              ].map(([id, label]) => (
                <button
                  key={id}
                  aria-pressed={filter === id}
                  onClick={() => setFilter(id)}
                >
                  {label}
                </button>
              ))}
            </div>
          </div>
          <div className="schedule-cards">
            {schedules
              .filter(
                (s) =>
                  matches(s.name) &&
                  (filter === "all" ||
                    (filter === "flows"
                      ? s.mode === "workflow"
                      : s.mode !== "workflow")),
              )
              .map((s) => (
                <article className="schedule-card" key={s.id}>
                  <span className="schedule-symbol">
                    {s.mode === "workflow" ? "↔" : "◷"}
                  </span>
                  <div className="schedule-card-main">
                    <div className="schedule-card-title">
                      <h2>{s.name}</h2>
                      <span className="badge">
                        {s.mode === "workflow"
                          ? t("Flow", "Flujo")
                          : s.mode === "reminder"
                            ? t("Reminder", "Recordatorio")
                            : "Prompt"}
                      </span>
                      <span className={`flow-state ${s.status}`}>
                        {stateLabel(s.status || "active", language)}
                      </span>
                    </div>
                    <p className="muted">{s.content}</p>
                    <small>
                      {t("Next", "Próxima")}:{" "}
                      {s.next_run
                        ? new Date(s.next_run).toLocaleString(language, {
                            timeZone: "America/Lima",
                          })
                        : "—"}{" "}
                      · America/Lima
                    </small>
                    {s.last_result && (
                      <p className="schedule-result">{s.last_result}</p>
                    )}
                    <div className="flow-actions">
                      <button
                        onClick={() => {
                          setEditing(s);
                          setCreate(true);
                        }}
                      >
                        {t("Configure", "Configurar")}
                      </button>
                      {s.workflow_id && (
                        <button
                          onClick={() =>
                            setSelected(
                              flows.find((f) => f.id === s.workflow_id) || null,
                            )
                          }
                        >
                          {t("Open flow", "Abrir flujo")}
                        </button>
                      )}
                      {["active", "paused"].includes(s.status || "") && (
                        <button disabled={busy} onClick={() => void act(s)}>
                          {s.status === "active"
                            ? t("Pause", "Pausar")
                            : t("Resume", "Reactivar")}
                        </button>
                      )}
                    </div>
                  </div>
                </article>
              ))}
            {filter !== "prompts" &&
              flows
                .filter(
                  (f) =>
                    matches(f.name) &&
                    !schedules.some((s) => s.workflow_id === f.id),
                )
                .map((flow) => (
                  <article className="schedule-card" key={flow.id}>
                    <span className="schedule-symbol">↔</span>
                    <div className="schedule-card-main">
                      <div className="schedule-card-title">
                        <h2>{flow.name}</h2>
                        <span className="badge">
                          {t("Flow", "Flujo")} · v{flow.version}
                        </span>
                      </div>
                      <p className="muted">{flow.description}</p>
                      <div className="schedule-mini-route">
                        {flow.nodes.slice(0, 5).map((n, i) => (
                          <span key={n.id}>
                            {i ? " → " : ""}
                            {n.label}
                          </span>
                        ))}
                        {flow.nodes.length > 5 ? " …" : ""}
                      </div>
                      <div className="flow-actions">
                        <button onClick={() => setSelected(flow)}>
                          {t("Open flow", "Abrir flujo")}
                        </button>
                        <button onClick={() => scheduleFlow(flow)}>
                          {t("Schedule", "Programar")}
                        </button>
                      </div>
                    </div>
                  </article>
                ))}
          </div>
          {!schedules.length && !flows.length && (
            <div className="schedule-empty">
              <span>✦</span>
              <h2>{t("Make room in your day", "Haz espacio en tu día")}</h2>
              <p>
                {t(
                  "Schedule a prompt or describe a workflow. Fénix will help you build it.",
                  "Programa un prompt o describe un flujo. Fénix te ayudará a construirlo.",
                )}
              </p>
            </div>
          )}
          <form
            className="flow-composer"
            onSubmit={(e) => {
              e.preventDefault();
              void describe();
            }}
          >
            <span>✦</span>
            <input
              value={instruction}
              onChange={(e) => setInstruction(e.target.value)}
              aria-label={t(
                "Describe an automation",
                "Describe una automatización",
              )}
              placeholder={t(
                "Every Monday, find my report in Drive and upload it to campus…",
                "Cada lunes busca mi informe en Drive y súbelo al campus…",
              )}
            />
            <button className="primary" disabled={busy || !instruction.trim()}>
              {t("Create with Fénix", "Crear con Fénix")} ↗
            </button>
          </form>
        </>
      )}
      {error && (
        <p role="alert" className="flow-error">
          {error}
        </p>
      )}
      {create && (
        <ScheduleDialog
          key={editing?.id || editing?.workflow_id || "new"}
          value={editing}
          flows={flows}
          language={language}
          onClose={() => setCreate(false)}
          onSaved={async () => {
            setCreate(false);
            await refresh();
          }}
        />
      )}
    </section>
  );
}
function ScheduleDialog({
  value,
  flows,
  language,
  onClose,
  onSaved,
}: {
  value: Schedule | null;
  flows: Flow[];
  language: string;
  onClose: () => void;
  onSaved: () => Promise<void>;
}) {
  const t = tr(language),
    [draft, setDraft] = useState<Schedule>(
      value
        ? { ...value, at: value.at ? limaInputTime(value.at) : value.at }
        : {
            name: "",
            content: "",
            mode: "task",
            kind: "cron",
            cron: "0 9 * * 1-5",
            interval_seconds: 3600,
          },
    ),
    [busy, setBusy] = useState(false),
    [error, setError] = useState("");
  const set = (key: keyof Schedule, value: unknown) =>
    setDraft((d) => ({ ...d, [key]: value }));
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !busy) onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [busy, onClose]);
  async function save() {
    setBusy(true);
    setError("");
    try {
      const flow = flows.find((f) => f.id === draft.workflow_id);
      const body: Record<string, unknown> = {
        action: draft.id ? "update" : "create",
        name: draft.name,
        content:
          draft.mode === "workflow"
            ? flow?.description || flow?.name || draft.content
            : draft.content,
        mode: draft.mode,
        kind: draft.kind,
      };
      if (draft.id) body.id = draft.id;
      if (draft.mode === "workflow") {
        body.workflow_id = draft.workflow_id;
        if (flow?.project_id) body.project_id = flow.project_id;
      } else if (draft.project_id) body.project_id = draft.project_id;
      if (draft.kind === "cron") body.cron = draft.cron;
      if (draft.kind === "interval")
        body.interval_seconds = Number(draft.interval_seconds);
      if (draft.kind === "once")
        body.at =
          draft.at?.includes("-05:00") || draft.at?.endsWith("Z")
            ? draft.at
            : `${draft.at}:00-05:00`;
      await api("/schedules", body);
      await onSaved();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <div className="flow-modal-backdrop">
      <section
        className="flow-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="schedule-dialog-title"
      >
        <header>
          <h2 id="schedule-dialog-title">
            {value?.id
              ? t("Edit schedule", "Editar programación")
              : t("New scheduled task", "Nueva tarea programada")}
          </h2>
          <button
            disabled={busy}
            onClick={onClose}
            aria-label={t("Close", "Cerrar")}
          >
            ×
          </button>
        </header>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void save();
          }}
        >
          <label>
            {t("Name", "Nombre")}
            <input
              autoFocus
              required
              value={draft.name}
              onChange={(e) => set("name", e.target.value)}
            />
          </label>
          <label>
            {t("Type", "Tipo")}
            <select
              value={draft.mode}
              onChange={(e) => set("mode", e.target.value)}
            >
              <option value="task">
                {t("Scheduled prompt", "Prompt programado")}
              </option>
              <option value="reminder">{t("Reminder", "Recordatorio")}</option>
              <option value="workflow">
                {t("Complete workflow", "Flujo completo")}
              </option>
            </select>
          </label>
          {draft.mode === "workflow" ? (
            <label>
              {t("Workflow", "Flujo")}
              <select
                required
                value={draft.workflow_id || ""}
                onChange={(e) => set("workflow_id", e.target.value)}
              >
                <option value="">
                  {t("Choose a saved flow", "Elige un flujo guardado")}
                </option>
                {flows.map((f) => (
                  <option key={f.id} value={f.id}>
                    {f.name}
                  </option>
                ))}
              </select>
              {!flows.length && (
                <small>
                  {t(
                    "Describe your flow in the conversation first.",
                    "Primero describe el flujo en la conversación.",
                  )}
                </small>
              )}
            </label>
          ) : (
            <label>
              {t("Instructions", "Instrucciones")}
              <textarea
                required
                value={draft.content}
                onChange={(e) => set("content", e.target.value)}
              />
            </label>
          )}
          <label>
            {t("Frequency", "Frecuencia")}
            <select
              value={draft.kind}
              onChange={(e) => set("kind", e.target.value)}
            >
              <option value="cron">
                {t("Recurring schedule", "Horario recurrente")}
              </option>
              <option value="interval">
                {t("Every interval", "Por intervalo")}
              </option>
              <option value="once">{t("Once", "Una vez")}</option>
            </select>
          </label>
          {draft.kind === "cron" && (
            <>
              <label>
                {t("Quick schedule", "Horario rápido")}
                <select
                  onChange={(e) => {
                    if (e.target.value) set("cron", e.target.value);
                  }}
                  value={
                    ["0 9 * * *", "0 9 * * 1-5", "0 8 * * 1"].includes(
                      draft.cron || "",
                    )
                      ? draft.cron!
                      : ""
                  }
                >
                  <option value="">{t("Custom", "Personalizado")}</option>
                  <option value="0 9 * * *">
                    {t("Every day at 09:00", "Todos los días a las 09:00")}
                  </option>
                  <option value="0 9 * * 1-5">
                    {t("Weekdays at 09:00", "Lunes a viernes a las 09:00")}
                  </option>
                  <option value="0 8 * * 1">
                    {t("Mondays at 08:00", "Lunes a las 08:00")}
                  </option>
                </select>
              </label>
              <label>
                {t("Cron expression", "Expresión cron")}
                <input
                  required
                  value={draft.cron || ""}
                  onChange={(e) => set("cron", e.target.value)}
                />
              </label>
            </>
          )}
          {draft.kind === "interval" && (
            <label>
              {t("Interval in seconds", "Intervalo en segundos")}
              <input
                required
                type="number"
                min="1"
                max="31536000"
                value={draft.interval_seconds || 3600}
                onChange={(e) =>
                  set("interval_seconds", Number(e.target.value))
                }
              />
            </label>
          )}
          {draft.kind === "once" && (
            <label>
              {t("Date and time", "Fecha y hora")}
              <input
                required
                type="datetime-local"
                value={draft.at ? draft.at.slice(0, 16) : ""}
                onChange={(e) => set("at", e.target.value)}
              />
            </label>
          )}
          <p className="muted">America/Lima · UTC−05:00</p>
          {error && <p role="alert">{error}</p>}
          <button className="primary" disabled={busy}>
            {busy
              ? t("Saving…", "Guardando…")
              : t("Save schedule", "Guardar programación")}
          </button>
        </form>
      </section>
    </div>
  );
}

function limaInputTime(timestamp: string) {
  const parts = new Intl.DateTimeFormat("en-CA", {
    timeZone: "America/Lima",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hourCycle: "h23",
  }).formatToParts(new Date(timestamp));
  const part = (type: string) => parts.find((p) => p.type === type)?.value;
  return `${part("year")}-${part("month")}-${part("day")}T${part("hour")}:${part("minute")}`;
}
let root: Root | undefined;
export function stopSchedules() {
  root?.unmount();
  root = undefined;
}
export function renderSchedulesPage(language: string) {
  stopSchedules();
  root = createRoot(document.getElementById("page")!);
  root.render(<Schedules language={language} />);
}
