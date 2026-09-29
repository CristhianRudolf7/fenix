import { serviceError } from "./components/service-error";
import { ensureWorkspace } from "./workspace/Workspace";
import { configureWorkspace, openTarget } from "./workspace/store";
import type { DesignChoice } from "./design/types";
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
import { renderProfile, stopProfile } from "./profile/Profile";
const errorMessage = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);
import {
  renderShell,
  renderChatLayout,
  renderSetupMarkup,
} from "./components/layout";
import {
  renderChatDesigns,
  stopDesignViews,
  refreshDesigns,
  openTemplatePreview,
  legacyTemplate,
} from "./design/app";
import { currentPage, onPageChange, navigateTo } from "./components/routing";
import { Loading } from "./components/loading";
import { renderToStaticMarkup } from "react-dom/server";
import { flameMarkup, iconMarkup } from "./components/icons";
// The former Design page now lives in chat; its template links open a preview there.
let pendingTemplate = legacyTemplate(new URL(location.href));
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
  skills: "Skills",
  documents: t("Library", "Biblioteca"),
  browser: t("Browser", "Navegador"),
  integrations: t("Integrations", "Integraciones"),
  schedules: t("Schedules", "Programaciones"),
  activity: t("Tasks", "Tareas"),
  profile: t("My profile", "Mi perfil"),
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
  if (!response.ok)
    throw new Error(serviceError(result.error || `HTTP ${response.status}`));
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
  if (setup)
    return `<section class="card setup-form"><h2>${t("We are preparing your assistant", "Estamos preparando tu asistente")}</h2><p>${t("Your account is waiting for service configuration. Please contact support if it remains unavailable.", "Tu cuenta está pendiente de la configuración del servicio. Contacta con soporte si sigue sin estar disponible.")}</p></section>`;
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
    <details id="advanced-settings"><summary class="muted">${t("Advanced and local settings", "Opciones avanzadas y locales")}</summary><div class="divider"></div>
    <div id="endpoint-field">${input("base_url", t("API base URL", "URL base de la API"), config.base_url || "", 'type="url" autocomplete="off"')}</div>
    <div id="project-header-field">${input("provider_project", t("Provider project / workspace (optional)", "Proyecto / workspace del proveedor (opcional)"), config.provider_project || "")}</div>
    <div id="organization-field">${input("organization", t("Organization (optional)", "Organización (opcional)"), config.organization || "")}</div>
    ${input("workspace", t("File workspace", "Carpeta de archivos"), config.workspace, "required")}
    ${field(`<label for="command_policy">${t("Command permissions", "Permisos para comandos")}</label>`, policySelect("command_policy", config.command_policy))}
    ${field(`<label for="file_write_policy">${t("File changes", "Cambios en archivos")}</label>`, policySelect("file_write_policy", config.file_write_policy))}
    <details><summary>${t("Coding tools and context", "Herramientas de programación y contexto")}</summary>
    <p class="muted">${t("Configure installed language servers. Fénix starts these commands with your user permissions; it never installs servers automatically. Leave [] to disable LSP.", "Configura servidores de lenguaje instalados. Fénix inicia estos comandos con tus permisos de usuario; no instala servidores automáticamente. Deja [] para desactivar LSP.")}</p>
    <label for="language-servers">${t("Language servers (JSON)", "Servidores de lenguaje (JSON)")}</label>
    <textarea id="language-servers" rows="6" spellcheck="false" placeholder='[{"name":"rust","command":["rust-analyzer"],"extensions":["rs"]}]'>${escape(JSON.stringify(config.coding?.language_servers || [], null, 2))}</textarea>
    ${input("context_window_tokens", t("Model context capacity (tokens)", "Capacidad de contexto del modelo (tokens)"), config.coding?.context_window_tokens ?? 200000, 'type="number" min="16384" max="2000000" required')}
    ${input("compact_at_tokens", t("Compact active context at (estimated tokens)", "Compactar contexto activo al llegar a (tokens estimados)"), config.coding?.compact_at_tokens ?? 160000, 'type="number" min="8000" required')}
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
    `<div class="setup-shell"><header class="setup-top">${brand()}<div class="row">${preferences()}<span id="account-controls"></span></div></header><main class="setup-main"><section class="setup-copy"><div class="eyebrow">${t("Your assistant. Your space.", "Tu asistente. Tu espacio.")}</div><h1>${t("A little less to do.<br>A little more possible.", "Menos pendientes.<br>Más posibilidades.")}</h1><p>${t("Meet your personal assistant, running on your computer and ready to help with everyday work.", "Tu asistente personal, en tu computadora y listo para ayudarte con el trabajo de cada día.")}</p><div class="features"><div class="feature"><span>◻</span><div><strong>${t("One continuous conversation", "Una conversación continua")}</strong><p>${t("Pick up where you left off, here or on Telegram.", "Continúa donde lo dejaste, aquí o en Telegram.")}</p></div></div><div class="feature"><span>▤</span><div><strong>${t("Memory you control", "Memoria que tú controlas")}</strong><p>${t("Keep the details you explicitly choose to remember.", "Conserva los detalles que elijas recordar.")}</p></div></div><div class="feature"><span>◇</span><div><strong>${t("Built around your work", "Se adapta a tu trabajo")}</strong><p>${t("Local tools, manual skills and timely reminders.", "Herramientas locales, skills y recordatorios.")}</p></div></div></div></section>${modelForm(true)}</main><footer class="setup-footer">${t("One account · Your own space", "Una cuenta · Tu propio espacio")}</footer></div>`,
  );
  if ($("#provider")) updateProviderFields(false);
}

async function render() {
  stopLibrary();
  stopDesignViews();
  stopSkills();
  stopBrowser();
  stopIntegrations();
  stopSchedules();
  stopProfile();
  if (!state.status.ready && state.page !== "profile") {
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
  if (state.page !== "chat") {
    $("#page").innerHTML = renderToStaticMarkup(
      Loading({ label: t("Preparing your space…", "Preparando tu espacio…") }),
    );
  }
  const views = {
    chat: renderChat,
    documents: () => renderLibrary(state.language),
    skills: renderSkills,
    browser: () => renderBrowser(state.language),
    integrations: () => renderIntegrations(state.language),
    schedules: () => renderSchedulesPage(state.language),
    activity: renderActivity,
    profile: () => renderProfile(state.language),
  };
  try {
    await views[state.page]();
  } catch (error) {
    toast(errorMessage(error));
  }
}
async function renderChat() {
  renderChatLayout(state.language, state.status.settings.model);
  renderChatDesigns(state.language);
  renderChatLibrary(state.language);
  const layout = $(".chat-layout");
  drawReplyReference();
  drawTemplateContext();
  updateActivityStrip();
  drawLiveMessage();
  if (pendingTemplate) {
    openTemplatePreview(pendingTemplate);
    pendingTemplate = null;
  }
  try {
    await refreshMessages(true);
    await new Promise<void>((resolve) =>
      requestAnimationFrame(() => resolve()),
    );
  } catch (error) {
    if (layout.isConnected) {
      const list = layout.querySelector("#message-list");
      list.innerHTML = `<p role="alert">${escape(t("Could not load messages.", "No se pudieron cargar los mensajes."))}</p><button class="chat-history-retry">${t("Try again", "Reintentar")}</button>`;
      list
        .querySelector("button")
        .addEventListener("click", () => void render());
    }
    toast(errorMessage(error));
  } finally {
    if (layout.isConnected) {
      const scroll = layout.querySelector("#chat-scroll");
      scroll.scrollTop = scroll.scrollHeight;
      layout.classList.remove("chat-history-loading");
      layout.setAttribute("aria-busy", "false");
      layout.querySelector(".chat-history-skeleton")?.remove();
    }
  }
}
let selectedTemplate: { id: string; title: string } | null = null;
function drawTemplateContext() {
  let node = document.getElementById("chat-template-context");
  const form = document.getElementById("chat-form");
  if (!form) return;
  if (!node) {
    node = document.createElement("div");
    node.id = "chat-template-context";
    node.className = "chat-template-context";
    form.before(node);
  }
  node.hidden = !selectedTemplate;
  if (!selectedTemplate) {
    node.replaceChildren();
    return;
  }
  node.innerHTML = `<span>${iconMarkup("design")}<span><small>${t("Chosen design", "Diseño elegido")}</small><strong>${escape(selectedTemplate.title)}</strong></span></span><button type="button" class="subtle icon-button" aria-label="${t("Remove chosen design", "Quitar diseño elegido")}">${iconMarkup("close")}</button>`;
  node.querySelector("button")!.onclick = () => {
    selectedTemplate = null;
    drawTemplateContext();
  };
}
type ChatRequest = {
  text: string;
  document_ids: string[];
  reply_to_task_id: string | null;
  design_choice?: DesignChoice;
};
function chatRequest(): ChatRequest {
  return {
    text: $("#chat-input")?.value.trim() || "",
    document_ids: selectedDocumentIds(),
    reply_to_task_id: state.replyTo?.taskId || null,
    ...(selectedTemplate
      ? {
          design_choice: {
            mode: "template" as const,
            template_id: selectedTemplate.id,
          },
        }
      : {}),
  };
}
let sendingMessage = false;
const optimisticMessages = new Map<string, { text: string; after: number }>();
async function sendChatRequest(request: ChatRequest) {
  if (sendingMessage) return;
  sendingMessage = true;
  window.dispatchEvent(new Event("fenix:stop-dictation"));
  const id = `pending-${crypto.randomUUID()}`;
  const after = Math.max(
    0,
    ...[...state.messages.values()].map((message) => message.seq),
  );
  optimisticMessages.set(id, { text: request.text, after });
  state.messages.set(id, {
    id,
    seq: after + 0.5,
    role: "user",
    text: request.text,
    created_at: new Date().toISOString(),
  });
  const input = $("#chat-input");
  if (input?.value.trim() === request.text.trim()) {
    input.value = "";
    input.dispatchEvent(new Event("input", { bubbles: true }));
  }
  if ($("#message-list")) {
    drawMessages();
    drawLiveMessage();
    const scroll = $("#chat-scroll");
    scroll.scrollTo({ top: scroll.scrollHeight, behavior: "instant" });
  }
  try {
    await api("/messages", { method: "POST", body: request });
    if (
      request.design_choice?.mode === "template" &&
      selectedTemplate?.id === request.design_choice.template_id
    ) {
      selectedTemplate = null;
      drawTemplateContext();
    }
    if ((state.replyTo?.taskId || null) === request.reply_to_task_id) {
      state.replyTo = null;
      drawReplyReference();
    }
    await refreshMessages(true).catch((error) => toast(errorMessage(error)));
  } catch (error) {
    if (optimisticMessages.has(id)) {
      optimisticMessages.delete(id);
      state.messages.delete(id);
      const currentInput = $("#chat-input");
      if (currentInput && !currentInput.value)
        currentInput.value = request.text;
      if ($("#message-list")) drawMessages();
    }
    throw error;
  } finally {
    sendingMessage = false;
    drawLiveMessage();
  }
}

function projectName(id) {
  return (
    state.projects.find((project) => project.id === id)?.name ||
    t("Project", "Proyecto")
  );
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
  node.innerHTML = `<div><strong>${t("Replying to", "Respondiendo a")} ${t("this request", "esta solicitud")}</strong><p>${escape(state.replyTo.text)}</p></div><button type="button" class="subtle" data-action="clear-reply" aria-label="${t("Remove reply reference", "Quitar referencia de respuesta")}">×</button>`;
}
async function refreshMessages(force = false) {
  const messages = await api("/messages");
  const changed = messages.some(
    (message) =>
      JSON.stringify(state.messages.get(message.id)) !==
      JSON.stringify(message),
  );
  let reconciled = false;
  for (const [id, pending] of optimisticMessages) {
    if (
      messages.some(
        (message) =>
          message.role === "user" &&
          message.seq > pending.after &&
          (message.text.trim() === pending.text.trim() ||
            message.text.startsWith(pending.text.trim() + "\n\n")),
      )
    ) {
      state.messages.delete(id);
      optimisticMessages.delete(id);
      reconciled = true;
    }
  }
  for (const message of messages) state.messages.set(message.id, message);
  if (!$("#message-list") || (!changed && !force && !reconciled)) return;
  const scroll = $("#chat-scroll");
  const nearBottom =
    scroll.scrollHeight - scroll.scrollTop - scroll.clientHeight < 100;
  drawMessages();
  drawLiveMessage();
  if (!state.messages.size) scroll.scrollTop = 0;
  else if (nearBottom || force) scroll.scrollTop = scroll.scrollHeight;
}
function conversationText(text: string) {
  return text.replace(
    /\b[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}\b/gi,
    t("(this request)", "(esta solicitud)"),
  );
}
function drawMessages() {
  const messages = [...state.messages.values()].sort((a, b) => a.seq - b.seq);
  let previousDay = "";
  $(".load-older").hidden = !messages.length;
  $("#message-list").innerHTML = messages.length
    ? messages
        .map((message) => {
          const timestamp = new Date(message.created_at);
          const validDate = !Number.isNaN(timestamp.getTime());
          const day = validDate
            ? new Intl.DateTimeFormat("en-CA", {
                timeZone: "America/Lima",
                year: "numeric",
                month: "2-digit",
                day: "2-digit",
              }).format(timestamp)
            : "unknown";
          const label = validDate
            ? new Intl.DateTimeFormat(state.language, {
                timeZone: "America/Lima",
                day: "numeric",
                month: "long",
                ...(timestamp.getFullYear() !== new Date().getFullYear()
                  ? { year: "numeric" as const }
                  : {}),
              }).format(timestamp)
            : t("Undated messages", "Mensajes sin fecha");
          const separator =
            day !== previousDay
              ? `<div class="message-day"><span>${escape(label)}</span></div>`
              : "";
          previousDay = day;
          const time = validDate
            ? new Intl.DateTimeFormat(state.language, {
                timeZone: "America/Lima",
                hour: "2-digit",
                minute: "2-digit",
              }).format(timestamp)
            : "";
          return `${separator}<article class="message ${message.role}" data-task-id="${escape(message.task_id || "")}" data-created-at="${escape(message.created_at)}"><span class="avatar" aria-hidden="true">${message.role === "user" ? t("Y", "T") : flameMarkup}</span><div class="message-body"><div class="message-head">${message.role === "user" ? t("You", "Tú") : "Fénix"}${message.origin === "telegram" ? "<small>Telegram</small>" : ""}</div><div class="message-text markdown-content">${markdownHtml(message.role === "assistant" ? conversationText(message.text) : message.text)}</div><div class="message-footer">${message.task_id ? `<div class="message-meta">${projectBadge(message)} <button class="subtle reply-button" data-reply-task="${escape(message.task_id)}" data-reply-message="${escape(message.id)}">${t("Reply", "Responder")}</button></div>` : ""}<div class="message-actions"><time datetime="${escape(message.created_at)}">${escape(time)}</time><button type="button" class="subtle message-copy" data-copy-message="${escape(message.id)}" aria-label="${t("Copy message", "Copiar mensaje")}" title="${t("Copy message", "Copiar mensaje")}"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true"><rect x="8" y="8" width="12" height="12" rx="2"/><path d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3"/></svg></button><span class="sr-only" role="status"></span></div></div></div></article>`;
        })
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
  // Design proposals attach to the reply that presented them.
  window.dispatchEvent(new Event("fenix:messages-drawn"));
}
function progressLabel() {
  const progress = state.progress;
  if (!progress?.task_id) return "";
  const task = state.tasks.find((item) => item.id === progress.task_id);
  if (task && !["queued", "running", "waiting"].includes(task.status))
    return "";
  if (!state.progressConnected)
    return t("Reconnecting to live progress…", "Reconectando con el progreso…");
  if (progress.phase === "waiting")
    return t("Waiting for your reply", "Esperando tu respuesta");
  if (progress.phase === "retrying")
    return t("Trying to reconnect", "Intentando volver a conectarme");
  if (progress.phase === "preparing")
    return t("Reviewing your request", "Revisando lo que me pediste");
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
    project_manage: ["Reviewing the project", "Revisando el proyecto"],
    history_search: [
      "Searching the conversation",
      "Buscando en la conversación",
    ],
    history_read: ["Reading the history", "Leyendo el historial"],
    skill_load: ["Preparing what is needed", "Preparando lo necesario"],
    schedule_manage: ["Managing schedules", "Gestionando tareas programadas"],
    task_inspect: ["Inspecting a task", "Consultando una tarea"],
    subagent_spawn: ["Organizing the work", "Organizando el trabajo"],
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
      ? t("Waiting for your reply", "Espera tu respuesta por texto")
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
      ? pending.length === 1
        ? t(
            "You have a question to answer",
            "Tienes una pregunta por responder",
          )
        : t("You have questions to answer", "Tienes preguntas por responder")
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
    if (sendingMessage) {
      node.innerHTML = `<article class="message assistant"><span class="avatar" aria-hidden="true">${flameMarkup}</span><div class="message-body"><div class="chat-typing" role="status" aria-label="${t("Fénix is responding", "Fénix está respondiendo")}"><span></span><span></span><span></span></div></div></article>`;
    } else node.replaceChildren();
    return;
  }
  node.querySelector(".chat-typing")?.closest("article")?.remove();
  const scroll = $("#chat-scroll");
  const nearBottom =
    scroll.scrollHeight - scroll.scrollTop - scroll.clientHeight < 100;
  if (!node.firstChild) {
    node.innerHTML = `<article class="message assistant"><span class="avatar" aria-hidden="true">${flameMarkup}</span><div class="message-body"><div class="message-head">Fénix <small class="live-label"></small></div><div class="message-text markdown-content"></div></div></article>`;
  }
  node.querySelector(".live-label").textContent = state.progressConnected
    ? t("Writing…", "Escribiendo…")
    : t("Reconnecting…", "Reconectando…");
  node.querySelector(".message-text").innerHTML = markdownHtml(
    conversationText(text),
  );
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
  progressSource.onopen = () => {
    state.progressConnected = true;
    updateActivityStrip();
    drawLiveMessage();
  };
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
  target.innerHTML = `<div class="content task-content">${heading(t("Every step, accounted for.", "Cada paso, a la vista."), t("Follow your work here. To stop something, tell Fénix which task you want to stop in the chat.", "Sigue tus tareas aquí. Para detener alguna, dile a Fénix en el chat cuál quieres parar."))}<section class="card" id="plans-panel"></section><section class="card background-panel" aria-labelledby="background-title"><div class="row spread"><h2 id="background-title">${t("Background tasks", "Tareas en segundo plano")}</h2><span class="badge" id="background-count"></span></div><p class="muted">${t("You can see progress and results here and in the chat.", "Puedes ver los avances y resultados aquí y en la conversación.")}</p><div id="background-subagents"></div></section><section class="card pending-panel" id="task-pending" hidden></section><section class="card"><div class="row spread section-title"><h2>${t("Task history", "Historial de tareas")}</h2><button data-action="refresh-activity">${t("Refresh", "Actualizar")}</button></div><div id="task-history"></div></section></div>`;
  drawTaskActivity();
}
function drawPlans() {
  const panel = $("#plans-panel");
  if (!panel) return;
  const plans = state.status.plans || [];
  panel.innerHTML = `<h2>${t("Plans", "Planes")}</h2><p class="muted">${t("You can change a plan at any time in the conversation, including while it is being prepared.", "Puedes modificar un plan en cualquier momento desde la conversación, incluso mientras se prepara.")}</p>${plans.length ? plans.map((plan) => `<article class="record"><div class="row spread"><h3>${escape(plan.objective)}</h3><span class="badge">v${escape(plan.version)} · ${escape(statusName(plan.status))}</span></div><ol>${plan.nodes.map((node) => `<li><strong>${escape(node.objective)}</strong> · ${escape(statusName(node.status))}<p class="muted">${escape(node.expected_output)}</p>${node.depends_on.length ? `<small>${t("Depends on", "Depende de")}: ${escape(node.depends_on.map((id) => plan.nodes.find((item) => item.id === id)?.objective || t("Another step", "Otro paso")).join(", "))}</small>` : ""}</li>`).join("")}</ol>${completionAuditMarkup(plan)}${(plan.reviews || []).map((review) => `<div class="note"><strong>${t("Plan review", "Revisión del plan")}: ${escape(statusName(review.status))}</strong><p>${escape(review.reason)}</p><small>${t("Round", "Ronda")} ${review.round + 1}/2 · ${review.node_ids.length ? escape(review.node_ids.map((id) => plan.nodes.find((item) => item.id === id)?.objective || t("A plan step", "Un paso del plan")).join(", ")) : t("Whole plan", "Plan completo")}</small></div>`).join("")}<button data-plan="${escape(plan.id)}">${t("View revisions", "Ver revisiones")}</button><p class="note">${t("Tell me in the chat what you would like to change about this plan.", "Dime en el chat qué quieres cambiar de este plan.")}</p></article>`).join("") : `<p class="muted">${t("Long tasks can run as plans with milestones and dependencies.", "Las tareas largas pueden ejecutarse como planes con hitos y dependencias.")}</p>`}`;
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
            `<article class="background-task" data-subagent="${escape(job.id)}"><div class="row spread"><h3>${escape(job.spec.objective)} ${projectBadge(job)}</h3><span class="badge">${escape(statusName(job.status))}</span></div><p class="task-output"><span class="muted">${t("Expected output", "Resultado esperado")}:</span> ${escape(job.spec.expected_output)}</p><div class="background-meta"><span>${t("Updated", "Actualizada")}: ${date(job.updated_at)}</span></div><details><summary>${t("Technical details", "Detalles técnicos")}</summary><code>${escape(job.id)}</code><br><code>${escape(job.parent_task_id)}</code><p>${t("Attempts", "Intentos")}: ${escape(job.attempts)} / 3</p></details>${job.result ? `<details class="subagent-result" data-result="${escape(job.id)}" ${open.has(job.id) ? "open" : ""}><summary>${t("Result", "Resultado")}${job.result.summary ? ` · ${escape(job.result.summary)}` : ""}</summary><pre>${escape(typeof job.result === "string" ? job.result : JSON.stringify(job.result, null, 2))}</pre></details>` : ""}</article>`,
        )
        .join("")
    : `<p class="muted">${t("No background work yet.", "Todavía no hay tareas en segundo plano.")}</p>`;
  const requests = $("#task-pending");
  requests.hidden = !pending.length;
  requests.innerHTML = `<h2>${t("Waiting for your reply", "Esperan tu respuesta")}</h2><p class="muted">${t("Reply in the conversation in your own words.", "Responde en la conversación con tus propias palabras.")}</p>${pending.map((request) => `<article class="background-task"><p>${escape(request.question)}</p></article>`).join("")}`;
  $("#task-history").innerHTML = state.tasks.length
    ? `<div class="table-wrap"><table class="table"><thead><tr><th>${t("Request", "Solicitud")}</th><th>${t("State", "Estado")}</th><th>${t("Created", "Creada")}</th><th>${t("Details", "Detalles")}</th></tr></thead><tbody>${state.tasks.map((task) => `<tr><td><div class="task-request">${escape(task.request)}</div>${projectBadge(task)}</td><td><span class="badge">${escape(statusName(task.status))}</span></td><td>${date(task.created_at)}<br><small>${escape(task.origin)}</small></td><td><button data-task="${escape(task.id)}">${t("View details", "Ver detalles")}</button></td></tr>`).join("")}</tbody></table></div>`
    : empty(
        t("Ready for the first task", "Todo listo para la primera tarea"),
        t(
          "Send a message to see its progress here.",
          "Envía un mensaje para ver su progreso aquí.",
        ),
      );
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
    t("Task details", "Detalles del trabajo"),
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
    if (button.dataset.copyMessage) {
      const message = state.messages.get(button.dataset.copyMessage);
      if (message) {
        const feedback = button.parentElement.querySelector('[role="status"]');
        try {
          await navigator.clipboard.writeText(
            message.role === "assistant"
              ? conversationText(message.text)
              : message.text,
          );
          button.setAttribute("aria-label", t("Copied", "Copiado"));
          button.title = t("Copied", "Copiado");
          feedback.textContent = t("Message copied", "Mensaje copiado");
          button.classList.add("is-copied");
          setTimeout(() => {
            button.classList.remove("is-copied");
            button.setAttribute(
              "aria-label",
              t("Copy message", "Copiar mensaje"),
            );
            button.title = t("Copy message", "Copiar mensaje");
            feedback.textContent = "";
          }, 2000);
        } catch {
          feedback.textContent = t(
            "Could not copy. Try again.",
            "No se pudo copiar. Inténtalo de nuevo.",
          );
        }
      }
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
      if (fields.provider !== "openai") fields.reasoning_effort = null;
      const result = await api("/settings", { method: "POST", body: fields });
      state.status = await api("/status");
      await render();
      toast(
        `${t("Connection verified", "Conexión verificada")} · ${result.model}`,
      );
    }
    if (form.id === "chat-form") {
      const request = chatRequest();
      if (!request.text) return;
      await sendChatRequest(request);
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
    if (state.page === "chat" && status.ready) void refreshDesigns();
    if (state.page === "activity" && status.ready) drawTaskActivity();
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
  // Keep the initial phoenix mounted while authentication and startup finish.
  // Only retries need to replace the error screen with a loading state.
  if (!document.querySelector("#app .boot[role='status']")) {
    renderSetupMarkup(
      renderToStaticMarkup(
        Loading({ label: t("Connecting to Fénix…", "Conectando con Fénix…") }),
      ),
    );
  }
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
onPageChange((page) => {
  if (state.page === page) return;
  state.page = page;
  if (state.status) void render();
});
await boot();

window.addEventListener("fenix:design-chat", async (event) => {
  const { text, template } = (
    event as CustomEvent<{
      text?: string;
      template?: { id: string; title: string };
    }>
  ).detail;
  const draft =
    document.querySelector<HTMLTextAreaElement>("#chat-input")?.value;
  if (template) selectedTemplate = template;
  state.page = "chat";
  navigateTo("chat");
  await render();
  const input = document.querySelector<HTMLTextAreaElement>("#chat-input");
  if (input) {
    input.value = text ?? draft ?? "";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.focus();
  }
});

// Choosing a proposal from its preview sends the choice, so the agent continues
// with the request already in the conversation. When it cannot be sent now, the
// choice stays attached to the composer for the next message.
window.addEventListener("fenix:design-choose", async (event) => {
  const { template } = (
    event as CustomEvent<{ template: { id: string; title: string } }>
  ).detail;
  if (state.page !== "chat") {
    state.page = "chat";
    navigateTo("chat");
    await render();
  }
  const keep = (message: string) => {
    selectedTemplate = template;
    drawTemplateContext();
    $("#chat-input")?.focus();
    toast(message);
  };
  if (sendingMessage) {
    keep(
      t(
        "Your design is attached. Send a message when the current one finishes.",
        "Tu diseño quedó adjunto. Envía un mensaje cuando termine el actual.",
      ),
    );
    return;
  }
  try {
    await sendChatRequest({
      text: t(
        `I choose the “${template.title}” design.`,
        `Elijo el diseño «${template.title}».`,
      ),
      document_ids: [],
      reply_to_task_id: null,
      design_choice: { mode: "template", template_id: template.id },
    });
    if (selectedTemplate?.id === template.id) {
      selectedTemplate = null;
      drawTemplateContext();
    }
  } catch (error) {
    keep(
      `${errorMessage(error)} ${t("The design stays attached; send a message to continue.", "El diseño sigue adjunto; envía un mensaje para continuar.")}`,
    );
  }
});
