import { ensureWorkspace } from "./workspace/Workspace";
import { configureWorkspace, openTarget } from "./workspace/store";
import {
  offerDesignCreation,
  finishDesignCreation,
  type CreationRequest,
} from "./design/creation";
import {
  renderLibrary,
  renderChatLibrary,
  stopLibrary,
  selectedDocumentIds,
} from "./documents/Library";
import { markdownHtml } from "./components/markdown";
import { renderSchedulesPage, stopSchedules } from "./workflows/Schedules";
import {
  renderIntegrations,
  stopIntegrations,
} from "./integrations/Integrations";
import { renderBrowser, stopBrowser } from "./browser/Browser";
import { renderSkillsPage, stopSkills } from "./skills/Skills";
const errorMessage = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);
import {
  renderShell,
  renderChatLayout,
  renderSetupMarkup,
} from "./components/layout";
import {
  renderDesign,
  renderChatDesigns,
  stopDesignViews,
  refreshDesigns,
} from "./design/app";
import { currentPage, onPageChange, navigateTo } from "./components/routing";
import { Loading } from "./components/loading";
import { renderToStaticMarkup } from "react-dom/server";
import { flameMarkup, iconMarkup } from "./components/icons";
const state: any = {
  page: currentPage(),
  status: null,
  providers: [],
  messages: new Map(),
  tasks: [],
  projects: [],
  replyTo: null,
  language: localStorage.getItem("fenix.language") || "es",
  theme: localStorage.getItem("fenix.theme") || "light",
  polling: false,
  progress: null,
  progressConnected: false,
};
const $ = (selector: string, root: ParentNode = document): any =>
  root.querySelector(selector);
const t = (english, spanish) => (state.language === "es" ? spanish : english);
const escape = (value: unknown = "") =>
  String(value ?? "").replace(
    /[&<>"']/g,
    (char) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        char
      ]!,
  );
const date = (value) =>
  value
    ? new Intl.DateTimeFormat(state.language, {
        dateStyle: "medium",
        timeStyle: "short",
        timeZone: "America/Lima",
      }).format(new Date(value))
    : "—";
const providerNames = {
  openai: "OpenAI",
  anthropic: "Anthropic",
  gemini: "Google Gemini",
  openrouter: "OpenRouter",
  deepseek: "DeepSeek",
  xai: "xAI",
  mistral: "Mistral AI",
  alibaba: "Alibaba Cloud",
  moonshot: "Moonshot AI",
  zai: "Z.ai",
  minimax: "MiniMax",
  groq: "Groq",
  together: "Together AI",
  bedrock: "Amazon Bedrock",
  azure: "Azure OpenAI",
  vertex: "Google Vertex AI",
  ollama: "Ollama",
};
const names = () => ({
  chat: t("Conversation", "Conversación"),
  projects: t("Projects", "Proyectos"),
  memory: t("Memory", "Memoria"),
  skills: "Skills",
  design: "Design",
  documents: t("Library", "Biblioteca"),
  browser: t("Browser", "Navegador"),
  integrations: t("Integrations", "Integraciones"),
  schedules: t("Schedules", "Programaciones"),
  activity: t("Tasks", "Tareas"),
  settings: t("Settings", "Configuración"),
});
const statusName = (value) =>
  ({
    queued: t("Queued", "En cola"),
    running: t("Running", "En curso"),
    waiting: t("Waiting for you", "Espera tu respuesta"),
    blocked: t("Blocked", "Bloqueada"),
    completed: t("Completed", "Completada"),
    verifying: t("Awaiting verification", "Pendiente de verificación"),
    verified: t("Verified", "Verificada"),
    doubtful: t("Unresolved doubt", "Duda pendiente"),
    refuted: t("Refuted", "Refutada"),
    failed: t("Failed", "Fallida"),
    cancelled: t("Cancelled", "Cancelada"),
    interrupted: t("Interrupted", "Interrumpida"),
    active: t("Active", "Activa"),
    paused: t("Paused", "Pausada"),
    missed: t("Missed", "Vencida"),
    pending: t("Pending", "Pendiente"),
    revising: t("Applying changes", "Aplicando cambios"),
    answered: t("Answered", "Respondida"),
    passed: t("Reviewed", "Revisada"),
    needs_resolution: t("Needs resolution", "Observaciones pendientes"),
    resolving: t("Resolving findings", "Resolviendo observaciones"),
    superseded: t("Superseded", "Sustituida"),
  })[value] || value;

document.documentElement.dataset.theme =
  state.theme === "dark" ? "dark" : "light";
document.documentElement.lang = state.language;

async function api(path: string, options: any = {}) {
  const response = await fetch(`/api${path}`, {
    ...options,
    headers: { "Content-Type": "application/json", ...options.headers },
    body: options.body === undefined ? undefined : JSON.stringify(options.body),
  });
  if (response.status === 401) {
    location.reload();
    throw new Error("Session expired");
  }
  let result;
  try {
    result = await response.json();
  } catch {
    throw new Error(
      t(
        "The local service returned an unexpected response.",
        "El servicio local devolvió una respuesta inesperada.",
      ),
    );
  }
  if (!response.ok) throw new Error(result.error || `HTTP ${response.status}`);
  return result;
}

let toastTimer: ReturnType<typeof setTimeout>;
function toast(message) {
  const node = $("#toast");
  node.textContent = message;
  node.classList.add("visible");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => node.classList.remove("visible"), 6000);
}
function preferences() {
  return `<button type="button" data-action="language" class="subtle" aria-label="${t("Change language", "Cambiar idioma")}">${state.language === "es" ? "EN" : "ES"}</button><button type="button" data-action="theme" class="subtle" aria-label="${t("Toggle light and dark mode", "Cambiar modo claro y oscuro")}">${iconMarkup(state.theme === "dark" ? "sun" : "moon")}</button>`;
}
function brand() {
  return `<div class="brand"><span class="flame-wrap">${flameMarkup}</span><span class="brand-name">Fénix</span></div>`;
}
function heading(title, description) {
  return `<div class="intro"><div><h1>${title}</h1><p>${description}</p></div></div>`;
}
function empty(title, detail) {
  return `<div class="empty"><h3>${title}</h3><p>${detail}</p></div>`;
}
function field(label, input, note = "") {
  return `<div class="field">${label}${input}${note ? `<small class="muted">${note}</small>` : ""}</div>`;
}
function input(id, label, value: string | number = "", extra = "") {
  return field(
    `<label for="${id}">${label}</label>`,
    `<input id="${id}" name="${id}" value="${escape(value)}" ${extra}>`,
  );
}

function modelForm(setup = false) {
  const config = state.status.settings || state.status.default_settings;
  return `<form id="model-form" class="card ${setup ? "setup-form" : ""}">
    <h2>${t("Connect your model", "Conecta tu modelo")}</h2><p class="muted">${t("Choose a provider. We will verify it with a real request.", "Elige un proveedor. Lo verificaremos con una llamada real.")}</p>
    ${field(`<label for="provider">${t("Provider", "Proveedor")}</label>`, `<select id="provider" name="provider">${state.providers.map((p) => `<option value="${p.id}" ${p.id === config.provider ? "selected" : ""}>${providerNames[p.id] || p.id}</option>`).join("")}</select>`)}
    ${input("model", t("Model", "Modelo"), config.model, 'required autocomplete="off" placeholder="Model ID" list="model-options"')}<datalist id="model-options"><option value="gpt-5.6-luna"></option></datalist>
    <div id="reasoning-field">${field(
      `<label for="reasoning_effort">${t("Reasoning effort", "Nivel de pensamiento")}</label>`,
      `<select id="reasoning_effort" name="reasoning_effort"><option value="">${t("Provider default", "Predeterminado del proveedor")}</option>${[
        ["none", t("None", "Sin razonamiento")],
        ["low", t("Low", "Bajo")],
        ["medium", t("Medium", "Medio")],
        ["high", t("High", "Alto")],
      ]
        .map(
          ([v, label]) =>
            `<option value="${v}" ${v === config.reasoning_effort ? "selected" : ""}>${label}</option>`,
        )
        .join("")}</select>`,
    )}</div>
    ${input("api_key", "API key", "", `type="password" autocomplete="new-password" ${setup ? "required" : ""} placeholder="${setup ? t("Paste your API key", "Pega tu API key") : t("Leave blank to keep the saved key", "Vacío para conservar la clave guardada")}"`)}
    <details id="advanced-settings"><summary class="muted">${t("Advanced and local settings", "Opciones avanzadas y locales")}</summary><div class="divider"></div>
    <div id="endpoint-field">${input("base_url", t("API base URL", "URL base de la API"), config.base_url || "", 'type="url" autocomplete="off"')}</div>
    <div id="project-header-field">${input("provider_project", t("Provider project / workspace (optional)", "Proyecto / workspace del proveedor (opcional)"), config.provider_project || "")}</div>
    <div id="organization-field">${input("organization", t("Organization (optional)", "Organización (opcional)"), config.organization || "")}</div>
    ${input("project", t("Credential project scope (optional)", "Ámbito de proyecto para la clave (opcional)"), config.project || "", 'pattern="[A-Za-z0-9_.-]{1,128}"')}
    ${input("workspace", t("File workspace", "Carpeta de archivos"), config.workspace, "required")}
    ${field(`<label for="command_policy">${t("Command permissions", "Permisos para comandos")}</label>`, policySelect("command_policy", config.command_policy))}
    ${field(`<label for="file_write_policy">${t("File changes", "Cambios en archivos")}</label>`, policySelect("file_write_policy", config.file_write_policy))}
    <details><summary>${t("Coding tools and context", "Herramientas de programación y contexto")}</summary>
    <p class="muted">${t("Configure installed language servers. Fénix starts these commands with your user permissions; it never installs servers automatically. Leave [] to disable LSP.", "Configura servidores de lenguaje instalados. Fénix inicia estos comandos con tus permisos de usuario; no instala servidores automáticamente. Deja [] para desactivar LSP.")}</p>
    <label for="language-servers">${t("Language servers (JSON)", "Servidores de lenguaje (JSON)")}</label>
    <textarea id="language-servers" rows="6" spellcheck="false" placeholder='[{"name":"rust","command":["rust-analyzer"],"extensions":["rs"]}]'>${escape(JSON.stringify(config.coding?.language_servers || [], null, 2))}</textarea>
    ${input("context_window_tokens", t("Model context capacity (tokens)", "Capacidad de contexto del modelo (tokens)"), config.coding?.context_window_tokens ?? 32768, 'type="number" min="16384" max="2000000" required')}
    ${input("compact_at_tokens", t("Compact active context at (estimated tokens)", "Compactar contexto activo al llegar a (tokens estimados)"), config.coding?.compact_at_tokens ?? 22000, 'type="number" min="8000" required')}
    ${input("keep_recent_tokens", t("Recent context to retain (estimated tokens)", "Contexto reciente a conservar (tokens estimados)"), config.coding?.keep_recent_tokens ?? 6000, 'type="number" min="1000" required')}
    <p class="muted">${t("Set capacity to your model's supported context. These are context limits, not a spending budget. Original context is saved before summarizing.", "Usa la capacidad admitida por tu modelo. Son límites de contexto, no un presupuesto de gasto. El contexto original se guarda antes de resumir.")}</p></details>
    <details><summary>${t("Execution limits", "Límites de ejecución")}</summary>
    ${input("max_model_steps", t("Model steps per task", "Pasos del modelo por tarea"), config.execution?.max_model_steps ?? 128, 'type="number" min="8" max="4096" required')}
    ${input("task_timeout_seconds", t("Task active time (seconds)", "Tiempo activo de tarea (segundos)"), config.execution?.task_timeout_seconds ?? 14400, 'type="number" min="60" max="604800" required')}
    ${input("subagent_timeout_seconds", t("Subagent attempt time (seconds)", "Tiempo por intento de subagente (segundos)"), config.execution?.subagent_timeout_seconds ?? 3600, 'type="number" min="60" max="604800" required')}
    ${input("command_timeout_seconds", t("Command default timeout (seconds)", "Tiempo por defecto de comando (segundos)"), config.execution?.command_timeout_seconds ?? 1800, 'type="number" min="1" max="604800" required')}
    ${input("service_timeout_seconds", t("Persistent session lifetime (seconds)", "Duración de sesión persistente (segundos)"), config.execution?.service_timeout_seconds ?? 86400, 'type="number" min="1" max="604800" required')}
    ${input("max_output_tokens", t("Maximum model output tokens", "Máximo de tokens de salida del modelo"), config.execution?.max_output_tokens ?? 16384, 'type="number" min="1024" max="131072" required')}
    <p class="muted">${t("Waiting for your answers does not consume active time. Persistent sessions stop on service shutdown. Retry and loop protections remain enabled.", "La espera de tus respuestas no consume tiempo activo. Las sesiones persistentes se detienen al apagar el servicio. Se mantienen las protecciones de reintentos y bucles.")}</p></details>
    <p class="note">${t("Commands can use any directory accessible to your Linux account. The file workspace is not a shell sandbox.", "Los comandos pueden usar cualquier directorio accesible a tu cuenta de Linux. La carpeta de archivos no limita los comandos.")}</p></details>
    <div class="divider"></div><button class="primary full" type="submit">${setup ? t("Verify and start", "Verificar y comenzar") : t("Verify and save", "Verificar y guardar")} <span aria-hidden="true">→</span></button>
    <p id="model-status" class="form-status" role="status"></p>
    </form>`;
}
function policySelect(id, value) {
  return `<select id="${id}" name="${id}"><option value="ask" ${value !== "allow" ? "selected" : ""}>${t("Ask in the conversation", "Preguntar en la conversación")}</option><option value="allow" ${value === "allow" ? "selected" : ""}>${t("Allow for this local installation", "Permitir en esta instalación local")}</option></select>`;
}
function setupView() {
  renderSetupMarkup(
    `<div class="setup-shell"><header class="setup-top">${brand()}<div class="row">${preferences()}<span id="account-controls"></span></div></header><main class="setup-main"><section class="setup-copy"><div class="eyebrow">${t("Your assistant. Your space.", "Tu asistente. Tu espacio.")}</div><h1>${t("A little less to do.<br>A little more possible.", "Menos pendientes.<br>Más posibilidades.")}</h1><p>${t("Meet your personal assistant, running on your computer and ready to help with everyday work.", "Tu asistente personal, en tu computadora y listo para ayudarte con el trabajo de cada día.")}</p><div class="features"><div class="feature"><span>◻</span><div><strong>${t("One continuous conversation", "Una conversación continua")}</strong><p>${t("Pick up where you left off, here or on Telegram.", "Continúa donde lo dejaste, aquí o en Telegram.")}</p></div></div><div class="feature"><span>▤</span><div><strong>${t("Memory you control", "Memoria que tú controlas")}</strong><p>${t("Keep the details you explicitly choose to remember.", "Conserva los detalles que elijas recordar.")}</p></div></div><div class="feature"><span>◇</span><div><strong>${t("Built around your work", "Se adapta a tu trabajo")}</strong><p>${t("Local tools, manual skills and timely reminders.", "Herramientas locales, skills y recordatorios.")}</p></div></div></div></section>${modelForm(true)}</main><footer class="setup-footer">${t("Runs locally · Telegram is optional · Your key stays in the backend", "Ejecución local · Telegram es opcional · Tu clave permanece en el backend")}</footer></div>`,
  );
  updateProviderFields(false);
}

async function render() {
  stopLibrary();
  stopDesignViews();
  stopSkills();
  stopBrowser();
  stopIntegrations();
  stopSchedules();
  if (!state.status.ready) {
    setupView();
    return;
  }
  ensureWorkspace();
  configureWorkspace(
    state.language,
    state.status.session_id,
    state.status.focus_project_id,
  );
  const labels = names();
  renderShell({
    page: state.page,
    language: state.language,
    theme: state.theme,
    model: state.status.settings.model,
    labels,
  });
  window.dispatchEvent(new Event("fenix:view"));
  $("#page").innerHTML = renderToStaticMarkup(
    Loading({ label: t("Preparing your space…", "Preparando tu espacio…") }),
  );
  const views = {
    chat: renderChat,
    design: () => renderDesign(state.language),
    documents: () => renderLibrary(state.language),
    projects: renderProjects,
    memory: renderMemory,
    skills: renderSkills,
    browser: () => renderBrowser(state.language),
    integrations: () => renderIntegrations(state.language),
    schedules: () => renderSchedulesPage(state.language),
    activity: renderActivity,
    settings: renderSettings,
  };
  try {
    await views[state.page]();
  } catch (error) {
    toast(errorMessage(error));
  }
}
function renderChat() {
  renderChatLayout(state.language, state.status.settings.model);
  renderChatDesigns(state.language);
  renderChatLibrary(state.language);
  refreshMessages(true);
  drawReplyReference();
  updateActivityStrip();
  drawLiveMessage();
}
function chatRequest(): CreationRequest {
  return {
    text: $("#chat-input")?.value.trim() || "",
    document_ids: selectedDocumentIds(),
    reply_to_task_id: state.replyTo?.taskId || null,
  };
}
async function sendChatRequest(request: CreationRequest) {
  await api("/messages", { method: "POST", body: request });
  finishDesignCreation(request.text);
  const input = $("#chat-input");
  if (input?.value.trim() === request.text.trim()) input.value = "";
  if ((state.replyTo?.taskId || null) === request.reply_to_task_id) {
    state.replyTo = null;
    drawReplyReference();
  }
  void refreshMessages(true).catch((error) => toast(errorMessage(error)));
}
function projectName(id) {
  return state.projects.find((project) => project.id === id)?.name || id;
}
function projectBadge(item) {
  const binding = item.binding;
  const ids = binding?.read_project_ids?.length
    ? [
        ...new Set(
          [binding.project_id, ...binding.read_project_ids].filter(Boolean),
        ),
      ]
    : [item.project_id || binding?.project_id].filter(Boolean);
  return ids
    .map(
      (id) =>
        `<span class="badge project-badge">${escape(projectName(id))}</span>`,
    )
    .join(" ");
}
function drawReplyReference() {
  let node = $("#reply-reference");
  if (!$("#chat-form")) return;
  if (!node) {
    node = document.createElement("div");
    node.id = "reply-reference";
    node.className = "reply-reference";
    $("#chat-form").before(node);
  }
  node.hidden = !state.replyTo;
  if (!state.replyTo) {
    node.replaceChildren();
    return;
  }
  node.innerHTML = `<div><strong>${t("Replying to", "Respondiendo a")} ${t("task", "la tarea")} ${escape(state.replyTo.taskId.slice(0, 8))}</strong><p>${escape(state.replyTo.text)}</p></div><button type="button" class="subtle" data-action="clear-reply" aria-label="${t("Remove reply reference", "Quitar referencia de respuesta")}">×</button>`;
}
async function refreshMessages(force = false) {
  const messages = await api("/messages");
  const changed = messages.some(
    (message) =>
      JSON.stringify(state.messages.get(message.id)) !==
      JSON.stringify(message),
  );
  for (const message of messages) state.messages.set(message.id, message);
  if (!$("#message-list") || (!changed && !force)) return;
  const scroll = $("#chat-scroll");
  const nearBottom =
    scroll.scrollHeight - scroll.scrollTop - scroll.clientHeight < 100;
  drawMessages();
  drawLiveMessage();
  if (!state.messages.size) scroll.scrollTop = 0;
  else if (nearBottom || force) scroll.scrollTop = scroll.scrollHeight;
}
function drawMessages() {
  const messages = [...state.messages.values()].sort((a, b) => a.seq - b.seq);
  $(".load-older").hidden = !messages.length;
  $("#conversation-context").hidden = !messages.length;
  $("#message-list").innerHTML = messages.length
    ? messages
        .map(
          (message) =>
            `<article class="message ${message.role}"><span class="avatar" aria-hidden="true">${message.role === "user" ? t("Y", "T") : flameMarkup}</span><div class="message-body"><div class="message-head">${message.role === "user" ? t("You", "Tú") : "Fénix"}<time datetime="${escape(message.created_at)}">${date(message.created_at)}</time>${message.origin === "telegram" ? "<small>Telegram</small>" : ""}</div><div class="message-text markdown-content">${markdownHtml(message.text)}</div>${message.task_id ? `<div class="message-meta">${projectBadge(message)} ${t("Task", "Tarea")} ${escape(message.task_id.slice(0, 8))} <button class="subtle reply-button" data-reply-task="${escape(message.task_id)}" data-reply-message="${escape(message.id)}">${t("Reply", "Responder")}</button></div>` : ""}</div></article>`,
        )
        .join("")
    : `<div class="chat-welcome"><span class="welcome-mark">${flameMarkup}</span><div class="eyebrow">${t("YOUR PERSONAL SPACE", "TU ESPACIO PERSONAL")}</div><h1>${t("Your ideas, in motion.", "Tus ideas, en marcha.")}</h1><p>${t("A little less on your mind. A little more possible. Tell me what you would like to do today.", "Menos pendientes en tu cabeza. Más espacio para lo que importa. Cuéntame qué te gustaría hacer hoy.")}</p><div class="suggestions">${[
        [
          t("Remember a detail", "Recordar un detalle"),
          t(
            "Remember that I prefer concise answers.",
            "Recuerda que prefiero respuestas concisas.",
          ),
        ],
        [
          t("Explore my files", "Explorar mis archivos"),
          t(
            "List the files in my workspace.",
            "Lista los archivos de mi carpeta de trabajo.",
          ),
        ],
        [
          t("Set a reminder", "Crear un recordatorio"),
          t(
            "Remind me in 30 minutes to take a break.",
            "Recuérdame en 30 minutos tomar un descanso.",
          ),
        ],
      ]
        .map(
          ([title, prompt], index) =>
            `<button class="suggestion" data-prompt="${escape(prompt)}"><span class="suggestion-icon">${iconMarkup(["memory", "projects", "schedules"][index])}</span><strong>${title}<span aria-hidden="true">↗</span></strong><span>${escape(prompt)}</span></button>`,
        )
        .join("")}</div></div>`;
}
function progressLabel() {
  const progress = state.progress;
  if (!progress?.task_id) return "";
  if (!state.progressConnected)
    return t("Reconnecting to live progress…", "Reconectando con el progreso…");
  if (progress.phase === "waiting")
    return t(
      "Waiting for your textual answer",
      "Esperando tu respuesta por texto",
    );
  if (progress.phase === "retrying")
    return t(
      "Retrying the model connection",
      "Reintentando la conexión con el modelo",
    );
  if (progress.phase === "preparing")
    return t("Preparing the context", "Preparando el contexto");
  if (progress.phase === "generating")
    return t("Generating a response", "Generando una respuesta");
  const labels = {
    files_list: ["Listing files", "Listando archivos"],
    file_read: ["Reading a file", "Leyendo un archivo"],
    files_search: ["Searching files", "Buscando en archivos"],
    file_write: ["Creating a file", "Creando un archivo"],
    file_edit: ["Editing a file", "Editando un archivo"],
    command_run: ["Starting a command", "Iniciando un comando"],
    command_status: ["Checking a command", "Consultando un comando"],
    document_extract: [
      "Reading PDF or image text",
      "Leyendo texto de PDF o imagen",
    ],
    command_cancel: ["Cancelling a command", "Cancelando un comando"],
    memory_search: ["Searching memory", "Buscando en la memoria"],
    memory_manage: ["Managing memory", "Gestionando la memoria"],
    project_manage: [
      "Consulting project context",
      "Consultando el contexto del proyecto",
    ],
    history_search: [
      "Searching the conversation",
      "Buscando en la conversación",
    ],
    history_read: ["Reading the history", "Leyendo el historial"],
    skill_load: ["Loading a skill", "Cargando un skill"],
    schedule_manage: ["Managing schedules", "Gestionando tareas programadas"],
    task_inspect: ["Inspecting a task", "Consultando una tarea"],
    subagent_spawn: ["Delegating work", "Delegando trabajo"],
    subagent_status: [
      "Checking background work",
      "Consultando trabajo en segundo plano",
    ],
    subagent_cancel: [
      "Cancelling background work",
      "Cancelando trabajo en segundo plano",
    ],
    ask_user: ["Preparing a question", "Preparando una pregunta"],
    request_approval: [
      "Preparing an approval request",
      "Preparando una solicitud de aprobación",
    ],
  };
  const label = labels[progress.tool];
  return label ? t(label[0], label[1]) : "";
}
function pendingRequests() {
  return (
    state.status.pending_requests ||
    (state.status.pending ? [state.status.pending] : [])
  );
}
function updateActivityStrip() {
  const node = $("#activity-strip");
  if (!node) return;
  const queued = state.tasks.filter(
    (task) => task.status === "queued" && task.origin !== "subagent",
  ).length;
  const background = (state.status.subagents || []).filter((task) =>
    ["queued", "running", "waiting"].includes(task.status),
  ).length;
  const pending = pendingRequests();
  const masterWaiting = pending.some(
    (request) => request.task_id === state.status.active_task,
  );
  const activity =
    progressLabel() ||
    (masterWaiting
      ? t("Waiting for your textual answer", "Espera tu respuesta por texto")
      : state.status.active_task
        ? t("Working on your request", "Trabajando en tu solicitud")
        : "");
  node.dataset.state = pending.length
    ? "waiting"
    : activity || background || queued
      ? "working"
      : "idle";
  node.textContent = [
    activity,
    background
      ? `${background} ${t("background tasks", "tareas en segundo plano")}`
      : "",
    pending.length
      ? `${pending.length} ${t("pending replies · use request IDs in the conversation", "respuestas pendientes · usa los IDs en la conversación")}`
      : "",
    queued ? `${queued} ${t("queued", "en cola")}` : "",
  ]
    .filter(Boolean)
    .join(" · ");
}
function drawLiveMessage() {
  const node = $("#live-message");
  if (!node) return;
  const progress = state.progress;
  const text = progress?.preview;
  if (
    !text ||
    progress.phase !== "generating" ||
    (progress.message_id && state.messages.has(progress.message_id))
  ) {
    node.replaceChildren();
    return;
  }
  const scroll = $("#chat-scroll");
  const nearBottom =
    scroll.scrollHeight - scroll.scrollTop - scroll.clientHeight < 100;
  if (!node.firstChild) {
    node.innerHTML = `<article class="message assistant"><span class="avatar" aria-hidden="true">${flameMarkup}</span><div class="message-body"><div class="message-head">Fénix <small class="live-label"></small></div><div class="message-text markdown-content"></div></div></article>`;
  }
  node.querySelector(".live-label").textContent = state.progressConnected
    ? t("Writing…", "Escribiendo…")
    : t("Reconnecting…", "Reconectando…");
  node.querySelector(".message-text").innerHTML = markdownHtml(text);
  if (nearBottom) scroll.scrollTop = scroll.scrollHeight;
}
let activitySource;
let activityRefresh;
function connectActivity() {
  activitySource?.close();
  const key = `fenix-events:${state.status?.session_id || "local"}`;
  const cursor = Number(
    sessionStorage.getItem(key) ?? state.status?.event_cursor ?? 0,
  );
  activitySource = new EventSource(
    `/api/events?after=${Number.isSafeInteger(cursor) ? cursor : 0}`,
  );
  activitySource.addEventListener("activity", (event: any) => {
    sessionStorage.setItem(key, event.lastEventId);
    try {
      const value = JSON.parse(event.data);
      if (value.topic === "workspace.open") openTarget(value.data);
    } catch {}
    // Durable history/state are authoritative. Debounce a burst of milestone
    // events into one UI refresh, without triggering model work.
    if (!activityRefresh)
      activityRefresh = setTimeout(() => {
        activityRefresh = null;
        poll();
      }, 100);
  });
}
window.addEventListener("pagehide", () => activitySource?.close());
window.addEventListener("pageshow", (event: any) => {
  if (event.persisted) connectActivity();
});

let progressSource;
function connectProgress() {
  progressSource?.close();
  progressSource = new EventSource("/api/progress");
  progressSource.addEventListener("progress", (event: any) => {
    const previous = state.progress;
    state.progress = JSON.parse(event.data);
    state.progressConnected = true;
    updateActivityStrip();
    drawLiveMessage();
    if (
      state.page === "chat" &&
      state.status?.ready &&
      (!previous ||
        previous.response_id !== state.progress.response_id ||
        previous.phase !== state.progress.phase)
    ) {
      refreshMessages().catch(() => {});
    }
  });
  progressSource.onerror = () => {
    state.progressConnected = false;
    updateActivityStrip();
    drawLiveMessage();
  };
}
window.addEventListener("pagehide", () => progressSource?.close());
window.addEventListener("pageshow", (event: any) => {
  if (event.persisted) connectProgress();
});

let viewLoadSequence = 0;
async function renderProjects() {
  if (state.page !== "projects") return;
  const target = $("#page");
  const sequence = ++viewLoadSequence;
  state.projects = await api("/projects");
  if (
    state.page !== "projects" ||
    target !== $("#page") ||
    sequence !== viewLoadSequence
  )
    return;
  target.innerHTML = `<div class="content">${heading(t("Each project, in context.", "Cada proyecto, con su contexto."), t("Keep objectives and decisions separate while talking in the same conversation. Mention a project by its name or alias when you need it.", "Mantén separados los objetivos y decisiones, conversando siempre en el mismo chat. Menciona el nombre o alias de un proyecto cuando lo necesites."))}<div class="grid"><section class="card"><div class="row spread"><h2>${t("Your projects", "Tus proyectos")}</h2><button data-action="refresh-projects">${t("Refresh", "Actualizar")}</button></div><div id="project-list"></div></section><form class="card" id="project-form"><h2 id="project-form-title">${t("Create a project", "Crear un proyecto")}</h2><input type="hidden" id="project-id">${input("project-name", t("Name", "Nombre"), "", "required maxlength=200")}${input("project-aliases", t("Aliases, separated by commas", "Alias, separados por comas"), "", "maxlength=2000")}${field(`<label for="project-description">${t("Description", "Descripción")}</label>`, '<textarea id="project-description" rows="3" maxlength="4000"></textarea>')}${input("project-workspace", t("Project folder (optional)", "Carpeta del proyecto (opcional)"), "", 'placeholder="/home/user/projects/example"')}${field(`<label for="project-brief">${t("Project context", "Contexto del proyecto")}</label>`, `<textarea id="project-brief" rows="12" maxlength="16000" placeholder="${escape(t("Objective\nConstraints and preferences\nProgress\nKey decisions\nRelevant files\nNext steps\nCritical context", "Objetivo\nRestricciones y preferencias\nProgreso\nDecisiones clave\nArchivos relevantes\nPróximos pasos\nContexto crítico"))}"></textarea>`, t("Keep the decisions and constraints that apply to this project. An optional folder supports projects with local files.", "Conserva las decisiones y restricciones de este proyecto. La carpeta es opcional y sirve para proyectos con archivos locales."))}<button class="primary" type="submit">${t("Save project", "Guardar proyecto")}</button><button class="subtle" type="button" data-action="clear-project">${t("New project", "Nuevo proyecto")}</button><p class="form-status" id="project-status"></p></form></div></div>`;
  drawProjects();
}
function drawProjects() {
  const node = $("#project-list");
  if (!node) return;
  node.innerHTML = state.projects.length
    ? state.projects
        .map(
          (project) =>
            `<article class="record"><h3>${escape(project.name)}</h3><p>${escape(project.description)}</p>${project.aliases?.length ? `<p class="muted">${t("Aliases", "Alias")}: ${escape(project.aliases.join(", "))}</p>` : ""}<small>${t("Updated", "Actualizado")}: ${date(project.updated_at)}${project.workspace ? `<br>${escape(project.workspace)}` : ""}</small><div class="actions"><button data-edit-project="${escape(project.id)}">${t("Edit", "Editar")}</button><button data-project-details="${escape(project.id)}">${t("View progress", "Ver progreso")}</button></div></article>`,
        )
        .join("")
    : empty(
        t("Your first project", "Tu primer proyecto"),
        t(
          "Create a project to give its work a name and its own context. General conversation remains available.",
          "Crea un proyecto para darle un nombre y contexto propio a su trabajo. La conversación general sigue disponible.",
        ),
      );
}
function projectOptions(selected = "") {
  return `<option value="">${t("General memory", "Memoria general")}</option>${state.projects.map((project) => `<option value="${escape(project.id)}" ${project.id === selected ? "selected" : ""}>${escape(project.name)}</option>`).join("")}`;
}
async function renderMemory() {
  if (state.page !== "memory") return;
  const target = $("#page");
  const sequence = ++viewLoadSequence;
  state.projects = await api("/projects");
  if (
    state.page !== "memory" ||
    target !== $("#page") ||
    sequence !== viewLoadSequence
  )
    return;
  target.innerHTML = `<div class="content">${heading(t("A memory you can trust.", "Lo importante, a mano."), t("Review memory candidates, confirmed facts and their sources. Correct or withdraw them at any time.", "Revisa candidatos, recuerdos confirmados y sus fuentes. Corrígelos o retíralos cuando quieras."))}<div class="grid"><section class="card"><h2>${t("Saved memories", "Recuerdos guardados")}</h2>${field(`<label for="memory-filter">${t("Memory scope", "Ámbito de memoria")}</label>`, `<select id="memory-filter">${projectOptions()}</select>`)}<form class="search-form" id="memory-search"><input id="memory-query" aria-label="${t("Search memory", "Buscar recuerdos")}" placeholder="${t("Search by words, title or ID…", "Buscar por palabras, título o ID…")}"><button type="submit">${t("Search", "Buscar")}</button></form><div id="memory-list"></div></section><form class="card" id="memory-form"><h2>${t("Keep a detail", "Guardar un detalle")}</h2><input type="hidden" id="memory-id">${field(`<label for="memory-kind">${t("Type", "Tipo")}</label>`, `<select id="memory-kind">${["fact", "preference", "decision", "assumption", "commitment", "constraint"].map((k) => `<option value="${k}">${escape(memoryLabel(k))}</option>`).join("")}</select>`)}${field(`<label for="memory-state">${t("Status", "Estado")}</label>`, `<select id="memory-state">${["confirmed", "candidate", "conflict", "expired", "superseded"].map((k) => `<option value="${k}">${escape(memoryLabel(k))}</option>`).join("")}</select>`)}${field(`<label for="memory-until">${t("Valid until (optional)", "Válido hasta (opcional)")}</label>`, '<input id="memory-until" type="datetime-local">')}${field(`<label for="memory-project">${t("Save in", "Guardar en")}</label>`, `<select id="memory-project">${projectOptions()}</select>`)}${input("memory-title", t("Title", "Título"), "", "required maxlength=200")}${field(`<label for="memory-content">${t("What should I remember?", "¿Qué debo recordar?")}</label>`, '<textarea id="memory-content" required maxlength="16000" rows="5"></textarea>')}<button class="primary" type="submit">${t("Save memory", "Guardar recuerdo")}</button><button class="subtle" data-action="clear-memory" type="button">${t("Clear form", "Limpiar formulario")}</button><p class="form-status" id="memory-status"></p></form></div></div>`;
  $(".content").insertAdjacentHTML(
    "beforeend",
    `<section class="card"><h2>${t("Automatic memory", "Memoria automática")}</h2><label><input type="checkbox" id="memory-auto"> ${t("Collect relevant statements as candidates in the background", "Recopilar afirmaciones relevantes como candidatos en segundo plano")}</label><p>${t("Extraction uses the configured model. Repeated preferences can be promoted; inferred assumptions remain candidates.", "La extracción utiliza el modelo configurado. Las preferencias repetidas pueden promoverse; los supuestos inferidos permanecen como candidatos.")}</p><label><input type="checkbox" id="memory-review"> ${t("Review ambiguous memory pairs in the background", "Revisar pares de recuerdos ambiguos en segundo plano")}</label><p>${t("Uses the configured model only for ambiguous pairs, with at most two attempts per version. Proposals never overwrite facts.", "Utiliza el modelo configurado solo para pares ambiguos, con hasta dos intentos por versión. Las propuestas nunca sobrescriben hechos.")}</p><p id="memory-health"></p><h2>${t("Entities and connections", "Entidades y conexiones")}</h2><p>${t("Manage names, aliases and relationships in the conversation. Each project keeps its own structure.", "Gestiona nombres, alias y relaciones en la conversación. Cada proyecto conserva su propia estructura.")}</p><div id="memory-entities"></div></section>`,
  );
  const config = await api("/memory/settings");
  if (
    state.page !== "memory" ||
    target !== $("#page") ||
    sequence !== viewLoadSequence
  )
    return;
  $("#memory-auto").checked = config.settings.automatic_capture;
  $("#memory-review").checked = config.settings.automatic_review !== false;
  const saveMemorySettings = async (event: Event) => {
    try {
      await api("/memory/settings", {
        method: "POST",
        body: {
          ...config.settings,
          automatic_capture: $("#memory-auto").checked,
          automatic_review: $("#memory-review").checked,
        },
      });
    } catch (error) {
      toast(errorMessage(error));
      const checkbox = event.target as HTMLInputElement;
      checkbox.checked = !checkbox.checked;
    }
  };
  $("#memory-auto").onchange = saveMemorySettings;
  $("#memory-review").onchange = saveMemorySettings;
  $("#memory-health").textContent =
    `${t("Pending candidates", "Candidatos pendientes")}: ${config.maintenance?.candidates || 0} · ${t("Failed extraction jobs", "Extracciones fallidas")}: ${config.maintenance?.failed_jobs || 0}`;
  await loadMemories();
}
function memoryLabel(value) {
  const labels = {
    fact: ["Fact", "Hecho"],
    preference: ["Preference", "Preferencia"],
    decision: ["Decision", "Decisión"],
    assumption: ["Assumption", "Supuesto"],
    commitment: ["Commitment", "Compromiso"],
    constraint: ["Constraint", "Restricción"],
    confirmed: ["Confirmed", "Confirmado"],
    candidate: ["Candidate", "Candidato"],
    conflict: ["Conflict", "Conflicto"],
    expired: ["Expired", "Caducado"],
    superseded: ["Superseded", "Sustituido"],
  };
  return labels[value] ? t(labels[value][0], labels[value][1]) : value;
}
let memories: Array<Record<string, any>> = [];
let memoryLoadSequence = 0;
async function loadMemories(query = "") {
  const sequence = ++memoryLoadSequence;
  const projectId = $("#memory-filter")?.value;
  const loaded = await api(
    `/memory?query=${encodeURIComponent(query)}${projectId ? `&project_id=${encodeURIComponent(projectId)}` : ""}`,
  );
  if (
    sequence !== memoryLoadSequence ||
    !$("#memory-list") ||
    $("#memory-filter")?.value !== projectId
  )
    return;
  const entities = await api("/memory/knowledge", {
    method: "POST",
    body: { action: "entities", project_id: projectId || null },
  });
  if (
    sequence !== memoryLoadSequence ||
    !$("#memory-list") ||
    $("#memory-filter")?.value !== projectId
  )
    return;
  memories = loaded;
  if ($("#memory-entities"))
    $("#memory-entities").innerHTML =
      entities
        .map(
          (e) =>
            `<button data-entity-card="${escape(e.id)}">${escape(e.name)} · ${escape(e.kind)}</button>`,
        )
        .join("") ||
      t(
        "No entities in this scope yet.",
        "Aún no hay entidades en este ámbito.",
      );
  $("#memory-list").innerHTML = memories.length
    ? memories
        .map(
          (memory) =>
            `<article class="record"><h3>${escape(memory.title)}</h3><span class="badge">${escape(memory.project_id ? projectName(memory.project_id) : t("General memory", "Memoria general"))}</span><span class="badge">${escape(memoryLabel(memory.metadata?.kind || "fact"))} · ${escape(memoryLabel(memory.metadata?.status || "confirmed"))}</span><div class="markdown-content">${markdownHtml(memory.content)}</div>${(memory.metadata?.evidence || []).map((e) => `<blockquote>${escape(e.quote)}<small>${escape(e.reference)}</small></blockquote>`).join("")}<small>${date(memory.updated_at)}<br>${escape(memory.source)}<br>${escape(memory.id)}</small><div class="actions"><button data-edit-memory="${memory.id}">${t("Edit", "Editar")}</button><button data-history-memory="${memory.id}">${t("History", "Historial")}</button><button data-delete-memory="${memory.id}">${t("Remove memory", "Eliminar recuerdo")}</button></div></article>`,
        )
        .join("")
    : empty(
        t("Nothing saved here yet", "Aún no hay recuerdos aquí"),
        t(
          "Add a detail or ask Fénix to remember it in the conversation.",
          "Agrega un detalle o pide a Fénix que lo recuerde en la conversación.",
        ),
      );
}
async function renderSkills() {
  if (state.page === "skills") renderSkillsPage(state.language);
}

let schedules: Array<Record<string, any>> = [];
async function renderSchedules() {
  renderSchedulesPage(state.language);
}

async function renderActivity() {
  if (state.page !== "activity") return;
  const target = $("#page");
  const sequence = ++viewLoadSequence;
  [state.tasks, state.status] = await Promise.all([
    api("/tasks"),
    api("/status"),
  ]);
  if (
    state.page !== "activity" ||
    target !== $("#page") ||
    sequence !== viewLoadSequence
  )
    return;
  target.innerHTML = `<div class="content task-content">${heading(t("Every step, accounted for.", "Cada paso, a la vista."), t("Follow tasks while Fénix keeps the conversation available. To cancel, type cancelar and the task ID in the conversation.", "Sigue las tareas mientras Fénix mantiene disponible la conversación. Para cancelar, escribe cancelar y el ID de la tarea en la conversación."))}<section class="card" id="plans-panel"></section><section class="card background-panel" aria-labelledby="background-title"><div class="row spread"><h2 id="background-title">${t("Background tasks", "Tareas en segundo plano")}</h2><span class="badge" id="background-count"></span></div><p class="muted">${t("Delegated work reports back here and in the conversation.", "El trabajo delegado devuelve su resultado aquí y en la conversación.")}</p><div id="background-subagents"></div></section><section class="card pending-panel" id="task-pending" hidden></section><section class="card"><div class="row spread section-title"><h2>${t("Task history", "Historial de tareas")}</h2><button data-action="refresh-activity">${t("Refresh", "Actualizar")}</button></div><div id="task-history"></div></section></div>`;
  drawTaskActivity();
}
function drawPlans() {
  const panel = $("#plans-panel");
  if (!panel) return;
  const plans = state.status.plans || [];
  panel.innerHTML = `<h2>${t("Plans", "Planes")}</h2><p class="muted">${t("You can change a plan at any time in the conversation, including while it is being prepared.", "Puedes modificar un plan en cualquier momento desde la conversación, incluso mientras se prepara.")}</p>${plans.length ? plans.map((plan) => `<article class="record"><div class="row spread"><h3>${escape(plan.objective)}</h3><span class="badge">v${escape(plan.version)} · ${escape(statusName(plan.status))}</span></div><code>${escape(plan.id)}</code><ol>${plan.nodes.map((node) => `<li><strong>${escape(node.objective)}</strong> · ${escape(statusName(node.status))}<p class="muted">${escape(node.expected_output)}</p>${node.depends_on.length ? `<small>${t("Depends on", "Depende de")}: ${escape(node.depends_on.join(", "))}</small>` : ""}</li>`).join("")}</ol>${completionAuditMarkup(plan)}${(plan.reviews || []).map((review) => `<div class="note"><strong>${t("Plan review", "Revisión del plan")}: ${escape(statusName(review.status))}</strong><p>${escape(review.reason)}</p><small>${t("Round", "Ronda")} ${review.round + 1}/2 · ${review.node_ids.length ? escape(review.node_ids.join(", ")) : t("Whole plan", "Plan completo")}</small></div>`).join("")}<button data-plan="${escape(plan.id)}">${t("View revisions", "Ver revisiones")}</button><p class="note">${t("In the chat", "En el chat")}: <code>replanificar ${escape(plan.id)} ${t("your change", "tu cambio")}</code></p></article>`).join("") : `<p class="muted">${t("Long tasks can run as plans with milestones and dependencies.", "Las tareas largas pueden ejecutarse como planes con hitos y dependencias.")}</p>`}`;
}

function completionAuditMarkup(plan) {
  const audits = (plan.completion_audits || [])
    .map(
      (audit) =>
        `<div class="note"><strong>${t("Result verification", "Verificación del resultado")}: ${escape(statusName(audit.status))}</strong><p>${escape(audit.node_id || t("Integrated result", "Resultado completo"))}</p>${audit.report?.summary ? `<p>${escape(audit.report.summary)}</p>` : ""}</div>`,
    )
    .join("");
  const evidence = plan.evidence_summary;
  if (!evidence?.total) return audits;
  return `${audits}<p class="muted">${t("Evidence ledger", "Registro de evidencia")}: ${escape(evidence.verified)} ${t("verified", "verificadas")} · ${escape(evidence.pending)} ${t("pending", "pendientes")} · ${escape(evidence.doubtful)} ${t("doubts", "dudas")}</p>`;
}

function drawTaskActivity() {
  drawPlans();
  const node = $("#background-subagents");
  if (!node) return;
  const jobs = state.status.subagents || [];
  const pending = pendingRequests();
  const signature = JSON.stringify([
    state.language,
    jobs,
    pending,
    state.tasks,
  ]);
  if (node.dataset.snapshot === signature) return;
  const open = new Set(
    [...node.querySelectorAll("details[open]")].map(
      (detail) => detail.dataset.result,
    ),
  );
  node.dataset.snapshot = signature;
  const activeCount = jobs.filter((job) =>
    ["queued", "running", "waiting"].includes(job.status),
  ).length;
  $("#background-count").textContent =
    `${activeCount} ${t("active", activeCount === 1 ? "activa" : "activas")}`;
  node.innerHTML = jobs.length
    ? jobs
        .map(
          (job) =>
            `<article class="background-task" data-subagent="${escape(job.id)}"><div class="row spread"><h3>${escape(job.spec.objective)} ${projectBadge(job)}</h3><span class="badge">${escape(statusName(job.status))}</span></div><p class="task-output"><span class="muted">${t("Expected output", "Resultado esperado")}:</span> ${escape(job.spec.expected_output)}</p><div class="background-meta"><span>${t("Task ID", "ID de tarea")}: <code>${escape(job.id)}</code></span><span>${t("Parent", "Principal")}: <code>${escape(job.parent_task_id)}</code></span><span>${t("Attempts", "Intentos")}: ${escape(job.attempts)} / 3</span><span>${t("Updated", "Actualizada")}: ${date(job.updated_at)}</span></div>${job.result ? `<details class="subagent-result" data-result="${escape(job.id)}" ${open.has(job.id) ? "open" : ""}><summary>${t("Result", "Resultado")}${job.result.summary ? ` · ${escape(job.result.summary)}` : ""}</summary><pre>${escape(typeof job.result === "string" ? job.result : JSON.stringify(job.result, null, 2))}</pre></details>` : ""}</article>`,
        )
        .join("")
    : `<p class="muted">${t("No delegated tasks yet.", "Todavía no hay tareas delegadas.")}</p>`;
  const requests = $("#task-pending");
  requests.hidden = !pending.length;
  requests.innerHTML = `<h2>${t("Waiting for your reply", "Esperan tu respuesta")}</h2><p class="muted">${t("Reply in the conversation using the request ID. Other background work can continue.", "Responde en la conversación usando el ID de la solicitud. El resto del trabajo en segundo plano puede continuar.")}</p>${pending.map((request) => `<article class="background-task"><p>${escape(request.question)}</p><div class="background-meta"><span>${t("Task ID", "ID de tarea")}: <code>${escape(request.task_id)}</code></span></div><p class="textual-control">${request.kind === "approval" ? `<code>aprobar ${escape(request.id)}</code> ${t("or", "o")} <code>rechazar ${escape(request.id)}</code>` : `<code>responder ${escape(request.id)} &lt;${t("answer", "respuesta")}&gt;</code>`}</p></article>`).join("")}`;
  $("#task-history").innerHTML = state.tasks.length
    ? `<div class="table-wrap"><table class="table"><thead><tr><th>${t("Request", "Solicitud")}</th><th>${t("State", "Estado")}</th><th>${t("Created", "Creada")}</th><th>${t("Details", "Detalles")}</th></tr></thead><tbody>${state.tasks.map((task) => `<tr><td><div class="task-request">${escape(task.request)}</div>${projectBadge(task)}<small><code>${escape(task.id)}</code></small></td><td><span class="badge">${escape(statusName(task.status))}</span></td><td>${date(task.created_at)}<br><small>${escape(task.origin)}</small></td><td><button data-task="${escape(task.id)}">${t("Inspect", "Inspeccionar")}</button></td></tr>`).join("")}</tbody></table></div>`
    : empty(
        t("Ready for the first task", "Todo listo para la primera tarea"),
        t(
          "Send a message to see its progress here.",
          "Envía un mensaje para ver su progreso aquí.",
        ),
      );
}
function renderSettings() {
  const telegram = state.status.telegram;
  $("#page").innerHTML =
    `<div class="content">${heading(t("Make it yours.", "Hazlo tuyo."), t("Your model, your instructions and your connection to Telegram. Everything belongs to this local installation.", "Tu modelo, tus instrucciones y tu conexión a Telegram. Todo pertenece a esta instalación local."))}<div class="split-settings">${modelForm()}<div class="stack"><form class="card" id="telegram-form"><div class="row spread"><h2>Telegram</h2><span class="badge">${t("Optional", "Opcional")}</span></div><p class="muted">${t("Continue this same conversation from your phone. Create a bot with @BotFather and connect its token here.", "Continúa la misma conversación desde tu teléfono. Crea un bot con @BotFather y conecta su token aquí.")}</p>${input("telegram-token", t("Bot token", "Token del bot"), "", `type="password" autocomplete="new-password" placeholder="${telegram ? t("Leave blank to keep it", "Vacío para conservarlo") : "123456:…"}"`)}<div class="field"><label class="toggle"><input id="telegram-enabled" type="checkbox" ${telegram?.enabled ? "checked" : ""}>${t("Enable Telegram", "Activar Telegram")}</label></div><button type="submit" class="primary">${t("Save Telegram", "Guardar Telegram")}</button><p class="form-status" id="telegram-status"></p>${telegram?.pairing_code && telegram.enabled ? `<p class="note">${t("Send this text to", "Envía este texto a")} <strong>@${escape(telegram.username)}</strong>:<br><code>/start ${escape(telegram.pairing_code)}</code></p>` : telegram?.owner_id ? `<p class="note">@${escape(telegram.username)} · ${t("Owner paired", "Propietario vinculado")}</p>` : ""}${telegram?.last_error ? `<p class="note">${escape(telegram.last_error)}</p>` : ""}</form><form class="card" id="instructions-form"><h2>${t("Identity and instructions", "Identidad e instrucciones")}</h2>${field('<label for="instruction-name">Markdown</label>', '<select id="instruction-name"><option>AGENTS.md</option><option>SOUL.md</option><option>IDENTITY.md</option><option>USER.md</option></select>')}<label for="instruction-text">${t("Instructions", "Instrucciones")}</label><textarea id="instruction-text" rows="8" maxlength="8000"></textarea><p class="note">${t("Applied from the next task. Manage MEMORY.md through the Memory section.", "Se aplican desde la siguiente tarea. Gestiona MEMORY.md desde la sección Memoria.")}</p><button type="submit">${t("Save instructions", "Guardar instrucciones")}</button><p id="instruction-status" class="form-status"></p></form></div></div></div>`;
  updateProviderFields(false);
  return loadInstruction();
}
async function loadInstruction() {
  const value = await api(`/instructions/${$("#instruction-name").value}`);
  $("#instruction-text").value = value.text;
}
function updateProviderFields(changed) {
  const id = $("#provider").value;
  const provider = state.providers.find((p) => p.id === id);
  $("#model-options").replaceChildren(
    ...(provider.default_model
      ? [new Option(provider.default_model, provider.default_model)]
      : []),
  );
  $("#reasoning-field").hidden = id !== "openai";
  $("#project-header-field").hidden = !provider.project_header;
  $("#organization-field").hidden = !provider.organization_header;
  $("#base_url").required = provider.requires_base_url;
  if (provider.requires_base_url) $("#advanced-settings").open = true;
  $("#base_url").placeholder = provider.base_url || "https://…";
  if (changed) {
    $("#model").value = provider.default_model || "";
    $("#base_url").value = "";
    $("#provider_project").value = "";
    $("#organization").value = "";
    $("#api_key").value = "";
    $("#reasoning_effort").value = id === "openai" ? "low" : "";
  }
}
function scheduleKind() {
  const kind = $("#schedule-kind").value;
  for (const value of ["once", "interval", "cron"])
    $(`#schedule-${value}`).hidden = kind !== value;
}

async function showTask(id, after = 0) {
  const value = await api(`/tasks/${id}?after=${after}`);
  showDetail(
    `${t("Task", "Tarea")} ${id.slice(0, 8)}`,
    `<p>${escape(value.task.request)}</p><p><span class="badge">${statusName(value.task.status)}</span> ${projectBadge(value)}</p>${value.binding?.workspace ? `<p class="muted">${escape(value.binding.workspace)}</p>` : ""}<code>${escape(id)}</code>${value.task.error ? `<p class="note">${escape(value.task.error)}</p>` : ""}${value.questions?.length ? `<h3>${t("Questions", "Preguntas")}</h3>${value.questions.map((q) => `<p>${escape(q.question)} · ${escape(statusName(q.status))}<br><code>${escape(q.id)}</code></p>`).join("")}` : ""}${value.events.map((event: any) => `<details class="record"><summary>${escape(event.kind)} · ${date(event.created_at)}</summary><pre>${escape(JSON.stringify(event.data, null, 2))}</pre></details>`).join("")}${value.events.length === 50 ? `<button data-task="${id}" data-after="${value.events.at(-1).seq}">${t("More events", "Más eventos")}</button>` : ""}`,
  );
}
function showDetail(title, html) {
  $("#detail-title").textContent = title;
  $("#detail-content").innerHTML = html;
  if (!$("#detail-dialog").open) $("#detail-dialog").showModal();
}
$("#close-dialog").addEventListener("click", () => $("#detail-dialog").close());

document.addEventListener("click", async (event: any) => {
  const button = event.target.closest("button");
  if (!button) return;
  try {
    if (button.dataset.plan) {
      const value = await api(
        `/plans/${encodeURIComponent(button.dataset.plan)}`,
      );
      showDetail(
        t("Plan revisions", "Revisiones del plan"),
        (value.evidence?.length
          ? `<details class="record"><summary>${t("Requirements, artifacts and evidence", "Requisitos, artefactos y evidencia")}</summary>${value.evidence.map((entry) => `<article class="record"><strong>${escape(entry.kind)} · ${escape(statusName(entry.state))}</strong><p>${escape(entry.statement)}</p><small>${escape(entry.source)}</small>${entry.question ? `<p>${escape(entry.question)}</p>` : ""}</article>`).join("")}</details>`
          : "") +
          (value.completion_audits || [])
            .map(
              (audit) =>
                `<details class="record"><summary>${t("Completion audit", "Auditoría del resultado")} · ${escape(statusName(audit.status))}</summary><pre>${escape(JSON.stringify(audit, null, 2))}</pre></details>`,
            )
            .join("") +
          (value.review_metrics?.length
            ? `<details class="record"><summary>${t("Review usage and elapsed time", "Consumo y tiempo de revisiones")}</summary><pre>${escape(JSON.stringify(value.review_metrics, null, 2))}</pre></details>`
            : "") +
          (value.reviews || [])
            .map(
              (review) =>
                `<details class="record"><summary>${t("Review", "Revisión")} · v${escape(review.version)} · ${escape(statusName(review.status))}</summary><p>${escape(review.reason)}</p><pre>${escape(JSON.stringify(review, null, 2))}</pre></details>`,
            )
            .join("") +
          value.revisions
            .map(
              (revision) =>
                `<details class="record"><summary>v${escape(revision.version)} · ${date(revision.created_at)}</summary><p>${escape(revision.reason)}</p><pre>${escape(JSON.stringify(revision.plan, null, 2))}</pre></details>`,
            )
            .join(""),
      );
    }
    if (button.dataset.action === "theme") {
      state.theme = state.theme === "dark" ? "light" : "dark";
      localStorage.setItem("fenix.theme", state.theme);
      document.documentElement.dataset.theme = state.theme;
      document.querySelectorAll('[data-action="theme"]').forEach((node) => {
        node.innerHTML = iconMarkup(state.theme === "dark" ? "sun" : "moon");
      });
    }
    if (button.dataset.action === "language") {
      state.language = state.language === "es" ? "en" : "es";
      localStorage.setItem("fenix.language", state.language);
      document.documentElement.lang = state.language;
      await render();
    }
    if (button.dataset.prompt) {
      $("#chat-input").value = button.dataset.prompt;
      $("#chat-input").focus();
    }
    if (button.dataset.replyTask) {
      const message = state.messages.get(button.dataset.replyMessage);
      state.replyTo = {
        taskId: button.dataset.replyTask,
        text: (message?.text || "").slice(0, 180),
      };
      drawReplyReference();
      $("#chat-input").focus();
    }
    if (button.dataset.action === "clear-reply") {
      state.replyTo = null;
      drawReplyReference();
    }
    if (button.dataset.action === "refresh-projects") {
      state.projects = await api("/projects");
      drawProjects();
    }
    if (button.dataset.action === "clear-project") {
      $("#project-form").reset();
      $("#project-id").value = "";
      $("#project-form-title").textContent = t(
        "Create a project",
        "Crear un proyecto",
      );
      $("#project-status").textContent = "";
    }
    if (button.dataset.editProject) {
      const { project } = await api(`/projects/${button.dataset.editProject}`);
      $("#project-id").value = project.id;
      $("#project-name").value = project.name;
      $("#project-aliases").value = project.aliases.join(", ");
      $("#project-description").value = project.description;
      $("#project-workspace").value = project.workspace || "";
      $("#project-brief").value = project.brief;
      $("#project-form-title").textContent = t(
        "Edit project",
        "Editar proyecto",
      );
      $("#project-name").focus();
    }
    if (button.dataset.projectDetails) {
      const { project, tasks } = await api(
        `/projects/${button.dataset.projectDetails}`,
      );
      showDetail(
        project.name,
        `${project.brief ? `<div class="markdown-content">${markdownHtml(project.brief)}</div>` : ""}<h3>${t("Recent tasks", "Tareas recientes")}</h3>${tasks.length ? tasks.map((task) => `<article class="record"><div class="row spread"><h3>${escape(task.request)}</h3><span class="badge">${escape(statusName(task.status))}</span></div><small>${date(task.updated_at)}</small><div class="actions"><button data-task="${escape(task.id)}">${t("Inspect", "Inspeccionar")}</button></div></article>`).join("") : `<p class="muted">${t("No tasks yet. Mention this project in the conversation to start.", "Aún no hay tareas. Menciona este proyecto en la conversación para comenzar.")}</p>`}`,
      );
    }
    if (button.dataset.action === "older") {
      const oldest = Math.min(
        ...[...state.messages.values()].map((m) => m.seq),
      );
      const messages = await api(`/messages?before=${oldest}`);
      for (const message of messages) state.messages.set(message.id, message);
      if (!messages.length)
        toast(
          t(
            "You are at the beginning of this conversation.",
            "Llegaste al inicio de esta conversación.",
          ),
        );
      drawMessages();
    }
    if (button.dataset.historyMemory || button.dataset.entityCard) {
      const body = button.dataset.historyMemory
        ? { action: "history", id: button.dataset.historyMemory }
        : { action: "card", query: button.dataset.entityCard };
      const data = await api("/memory/knowledge", {
        method: "POST",
        body: { ...body, project_id: $("#memory-filter").value || null },
      });
      const renderFact = (m) =>
        `<article class="record"><h3>${escape(m.title)}</h3><span class="badge">${escape(memoryLabel(m.metadata?.status || "confirmed"))}</span><p>${escape(m.content)}</p>${(m.metadata?.evidence || []).map((e) => `<blockquote>${escape(e.quote)}<small>${escape(e.reference)}</small></blockquote>`).join("")}</article>`;
      const html = button.dataset.historyMemory
        ? data
            .map(
              (revision) =>
                `<section><small>${date(revision.changed_at)}</small>${renderFact(revision.snapshot)}</section>`,
            )
            .join("")
        : `<h3>${escape(data.entity?.name)}</h3><p>${escape(data.entity?.description)}</p><small>${escape((data.entity?.aliases || []).join(", "))}</small>${(data.facts || []).map(renderFact).join("")}<h3>${t("Connections", "Conexiones")}</h3>${(data.relations?.neighbors || []).map((n) => `<article class="record"><p>${escape(n.edge.predicate.replaceAll("_", " "))} · ${escape(n.entity.name)}</p><small>${t("Evidence", "Evidencia")}: ${escape(n.edge.evidence_memory_id)}</small></article>`).join("")}`;
      showDetail(t("Memory evidence", "Evidencia de memoria"), html);
    }
    if (button.dataset.editMemory) {
      const memory = memories.find((m) => m.id === button.dataset.editMemory);
      if (!memory) return;
      $("#memory-id").value = memory.id;
      $("#memory-kind").value = memory.metadata?.kind || "fact";
      $("#memory-state").value = memory.metadata?.status || "confirmed";
      $("#memory-until").value = memory.metadata?.valid_until
        ? new Date(
            new Date(memory.metadata.valid_until).getTime() -
              new Date(memory.metadata.valid_until).getTimezoneOffset() * 60000,
          )
            .toISOString()
            .slice(0, 16)
        : "";
      $("#memory-title").value = memory.title;
      $("#memory-content").value = memory.content;
      $("#memory-project").value = memory.project_id || "";
      $("#memory-project").disabled = true;
      $("#memory-title").focus();
    }
    if (button.dataset.deleteMemory) {
      const memory = memories.find(
        (item) => item.id === button.dataset.deleteMemory,
      );
      await api(
        `/memory/${button.dataset.deleteMemory}${memory?.project_id ? `?project_id=${encodeURIComponent(memory.project_id)}` : ""}`,
        {
          method: "DELETE",
          body: {},
        },
      );
      await loadMemories();
      toast(t("Memory removed.", "Recuerdo eliminado."));
    }
    if (button.dataset.action === "clear-memory") {
      $("#memory-form").reset();
      $("#memory-id").value = "";
      $("#memory-project").disabled = false;
      $("#memory-project").value = $("#memory-filter").value;
    }
    if (button.dataset.readSkill) {
      const result = await api(`/skills/${button.dataset.readSkill}`);
      showDetail(
        button.dataset.readSkill,
        `<div class="markdown-content">${markdownHtml(result.content)}</div>`,
      );
    }
    if (button.dataset.editSchedule) {
      const schedule = schedules.find(
        (s) => s.id === button.dataset.editSchedule,
      );
      if (!schedule) return;
      $("#schedule-id").value = schedule.id;
      $("#schedule-name").value = schedule.name;
      $("#schedule-content").value = schedule.content;
      $("#schedule-mode").value = schedule.mode;
      $("#schedule-kind").value = schedule.kind;
      $("#schedule-seconds").value = schedule.interval_seconds || 3600;
      $("#schedule-expression").value = schedule.cron || "";
      $("#schedule-at").value = schedule.at
        ? new Date(new Date(schedule.at).getTime() - 5 * 3600 * 1000)
            .toISOString()
            .slice(0, 16)
        : "";
      scheduleKind();
      $("#schedule-name").focus();
    }
    if (button.dataset.scheduleState) {
      await api("/schedules", {
        method: "POST",
        body: { action: button.dataset.scheduleState, id: button.dataset.id },
      });
      await renderSchedules();
    }
    if (button.dataset.action === "refresh-activity") await renderActivity();
    if (button.dataset.task)
      await showTask(button.dataset.task, Number(button.dataset.after || 0));
  } catch (error) {
    toast(errorMessage(error));
  }
});
document.addEventListener("change", async (event: any) => {
  try {
    if (event.target.id === "provider") updateProviderFields(true);
    if (event.target.id === "schedule-kind") scheduleKind();
    if (event.target.id === "memory-filter") {
      $("#memory-form").reset();
      $("#memory-id").value = "";
      $("#memory-project").disabled = false;
      $("#memory-project").value = event.target.value;
      await loadMemories($("#memory-query").value);
    }
    if (event.target.id === "instruction-name") await loadInstruction();
    if (event.target.dataset.skill) {
      await api(`/skills/${event.target.dataset.skill}`, {
        method: "PUT",
        body: { enabled: event.target.checked },
      });
      await renderSkills();
    }
  } catch (error) {
    toast(errorMessage(error));
  }
});
document.addEventListener("keydown", (event: any) => {
  if (
    event.target.id === "chat-input" &&
    event.key === "Enter" &&
    !event.shiftKey &&
    !event.isComposing
  ) {
    event.preventDefault();
    $("#chat-form").requestSubmit();
  }
});
document.addEventListener("submit", async (event: any) => {
  if (event.target.closest("#voice-root")) return;
  event.preventDefault();
  const form = event.target;
  const button = form.querySelector('[type="submit"]');
  if (button?.disabled) return;
  if (button) button.disabled = true;
  try {
    if (form.id === "model-form") {
      $("#model-status").textContent = t(
        "Checking a real inference request. This may take a moment…",
        "Comprobando una llamada real al modelo. Puede tardar un momento…",
      );
      $("#model-status").classList.remove("error");
      const fields: any = Object.fromEntries(new FormData(form));
      for (const key of [
        "base_url",
        "project",
        "provider_project",
        "organization",
        "reasoning_effort",
      ])
        if (!fields[key]) fields[key] = null;
      fields.coding = {
        language_servers: JSON.parse($("#language-servers").value),
      };
      for (const key of [
        "context_window_tokens",
        "compact_at_tokens",
        "keep_recent_tokens",
      ]) {
        fields.coding[key] = Number(fields[key]);
        delete fields[key];
      }
      fields.execution = {};
      for (const key of [
        "max_model_steps",
        "task_timeout_seconds",
        "subagent_timeout_seconds",
        "command_timeout_seconds",
        "service_timeout_seconds",
        "max_output_tokens",
      ]) {
        fields.execution[key] = Number(fields[key]);
        delete fields[key];
      }
      fields.flex_processing =
        fields.provider === "openai" &&
        Boolean(state.status.settings?.flex_processing);
      if (!fields.api_key) fields.api_key = null;
      if (fields.provider !== "openai") fields.reasoning_effort = null;
      const result = await api("/settings", { method: "POST", body: fields });
      $("#api_key").value = "";
      state.status = await api("/status");
      await render();
      toast(
        `${t("Connection verified", "Conexión verificada")} · ${result.model}`,
      );
    }
    if (form.id === "chat-form") {
      const request = chatRequest();
      if (!request.text) return;
      if (!offerDesignCreation(request, sendChatRequest))
        await sendChatRequest(request);
    }
    if (form.id === "project-form") {
      const id = $("#project-id").value;
      await api(id ? `/projects/${id}` : "/projects", {
        method: id ? "PUT" : "POST",
        body: {
          name: $("#project-name").value,
          aliases: $("#project-aliases")
            .value.split(",")
            .map((alias) => alias.trim())
            .filter(Boolean),
          description: $("#project-description").value,
          workspace: $("#project-workspace").value.trim() || null,
          brief: $("#project-brief").value,
        },
      });
      await renderProjects();
      toast(
        t(
          "Project saved. Continue in the same conversation.",
          "Proyecto guardado. Continúa en la misma conversación.",
        ),
      );
    }
    if (form.id === "memory-search")
      await loadMemories($("#memory-query").value);
    if (form.id === "memory-form") {
      await api("/memory", {
        method: "POST",
        body: {
          id: $("#memory-id").value || null,
          title: $("#memory-title").value,
          content: $("#memory-content").value,
          metadata: {
            ...(memories.find((m) => m.id === $("#memory-id").value)
              ?.metadata || {}),
            kind: $("#memory-kind").value,
            status: $("#memory-state").value,
            assertion:
              $("#memory-state").value === "confirmed"
                ? "explicit"
                : memories.find((m) => m.id === $("#memory-id").value)?.metadata
                    ?.assertion || "explicit",
            valid_until: $("#memory-until").value
              ? new Date($("#memory-until").value).toISOString()
              : null,
          },
          project_id: $("#memory-project").value || null,
        },
      });
      const savedProject = $("#memory-project").value;
      form.reset();
      $("#memory-id").value = "";
      $("#memory-project").disabled = false;
      $("#memory-project").value = savedProject;
      $("#memory-filter").value = savedProject;
      $("#memory-status").textContent = t(
        "Memory saved and verified.",
        "Recuerdo guardado y verificado.",
      );
      await loadMemories();
    }
    if (form.id === "schedule-form") {
      const kind = $("#schedule-kind").value;
      const id = $("#schedule-id").value;
      const body: any = {
        action: id ? "update" : "create",
        name: $("#schedule-name").value,
        content: $("#schedule-content").value,
        mode: $("#schedule-mode").value,
        kind,
      };
      if (id) body.id = id;
      if (kind === "once") body.at = `${$("#schedule-at").value}:00-05:00`;
      if (kind === "interval")
        body.interval_seconds = Number($("#schedule-seconds").value);
      if (kind === "cron") body.cron = $("#schedule-expression").value;
      await api("/schedules", { method: "POST", body });
      await renderSchedules();
      toast(t("Schedule saved.", "Programación guardada."));
    }
    if (form.id === "telegram-form") {
      await api("/telegram", {
        method: "POST",
        body: {
          token: $("#telegram-token").value || null,
          enabled: $("#telegram-enabled").checked,
        },
      });
      $("#telegram-token").value = "";
      state.status = await api("/status");
      await renderSettings();
      toast(
        t("Telegram settings saved.", "Configuración de Telegram guardada."),
      );
    }
    if (form.id === "instructions-form") {
      await api(`/instructions/${$("#instruction-name").value}`, {
        method: "PUT",
        body: { text: $("#instruction-text").value },
      });
      $("#instruction-status").textContent = t(
        "Saved. Applies from the next task.",
        "Guardado. Se aplica desde la siguiente tarea.",
      );
    }
  } catch (error) {
    const target = form.querySelector(".form-status");
    if (target) {
      target.textContent = errorMessage(error);
      target.classList.add("error");
    } else toast(errorMessage(error));
  } finally {
    if (button?.isConnected) {
      button.disabled = false;
    }
  }
});

async function poll() {
  if (state.polling || document.hidden) return;
  state.polling = true;
  try {
    const before = state.status?.ready;
    const [status, tasks] = await Promise.all([api("/status"), api("/tasks")]);
    state.status = status;
    state.tasks = tasks;
    const projectsChanged =
      status.projects &&
      (status.projects.length !== state.projects.length ||
        status.projects.some(
          (project) =>
            !state.projects.some(
              (existing) =>
                existing.id === project.id &&
                existing.context_version === project.context_version,
            ),
        ));
    if (status.projects)
      state.projects = status.projects.map((project) => {
        const existing = state.projects.find((item) => item.id === project.id);
        return existing?.context_version === project.context_version
          ? existing
          : { ...existing, ...project };
      });
    if (status.ready !== before) await render();
    if (state.page === "chat" && status.ready) {
      await refreshMessages(projectsChanged);
      updateActivityStrip();
    }
    if (["chat", "design"].includes(state.page) && status.ready)
      void refreshDesigns();
    if (state.page === "activity" && status.ready) drawTaskActivity();
    if (state.page === "projects" && status.ready) {
      if (projectsChanged) state.projects = await api("/projects");
      drawProjects();
    }
    if ($("#connection-state"))
      $("#connection-state").textContent = t("Connected", "Conectado");
    if ($("#connection-state"))
      $("#connection-state").dataset.state = "connected";
  } catch {
    if ($("#connection-state"))
      $("#connection-state").dataset.state = "disconnected";
    if ($("#connection-state"))
      $("#connection-state").textContent = t(
        "Service disconnected",
        "Servicio desconectado",
      );
  } finally {
    state.polling = false;
  }
}
async function boot() {
  renderSetupMarkup(
    renderToStaticMarkup(
      Loading({
        label: t("Connecting to your space…", "Conectando con tu espacio…"),
      }),
    ),
  );
  try {
    [state.status, state.providers] = await Promise.all([
      api("/status"),
      api("/providers"),
    ]);
    state.projects = state.status.projects || [];
    await render();
    connectProgress();
    connectActivity();
    setInterval(poll, 1800);
  } catch (error) {
    renderSetupMarkup(
      `<main class="boot">${brand()}<p>${t("Unable to reach Fénix", "No se pudo conectar con Fénix")}</p><span>${escape(errorMessage(error))}</span><button data-action="retry">${t("Try again", "Reintentar")}</button></main>`,
    );
    $('[data-action="retry"]').addEventListener("click", boot);
  }
}
window.addEventListener("fenix:messages", () => refreshMessages());
window.addEventListener("fenix:choose-design", () => {
  offerDesignCreation(chatRequest(), sendChatRequest, true);
});
window.addEventListener("fenix:design-draft", (event) => {
  const input = $("#chat-input");
  if (input) input.value = (event as CustomEvent<{ text: string }>).detail.text;
});
onPageChange((page) => {
  if (state.page === page) return;
  state.page = page;
  if (state.status) void render();
});
await boot();

window.addEventListener("fenix:design-chat", async (event) => {
  const { text } = (event as CustomEvent<{ text: string }>).detail;
  state.page = "chat";
  navigateTo("chat");
  await render();
  const input = document.querySelector<HTMLTextAreaElement>("#chat-input");
  if (input) {
    input.value = text;
    input.focus();
  }
});
