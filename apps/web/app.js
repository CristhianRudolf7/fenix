const state = {
  page: "chat",
  status: null,
  providers: [],
  messages: new Map(),
  tasks: [],
  language: localStorage.getItem("fenix.language") || "es",
  theme: localStorage.getItem("fenix.theme") || "light",
  polling: false,
  progress: null,
  progressConnected: false,
};
const $ = (selector, root = document) => root.querySelector(selector);
const t = (english, spanish) => (state.language === "es" ? spanish : english);
const escape = (value = "") =>
  String(value ?? "").replace(
    /[&<>"']/g,
    (char) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        char
      ],
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
  memory: t("Memory", "Memoria"),
  skills: "Skills",
  schedules: t("Schedules", "Programaciones"),
  activity: t("Activity", "Actividad"),
  settings: t("Settings", "Configuración"),
});
const icons = {
  chat: "◻",
  memory: "▤",
  skills: "◇",
  schedules: "◷",
  activity: "≋",
  settings: "⚙",
};
const statusName = (value) =>
  ({
    queued: t("Queued", "En cola"),
    running: t("Running", "En curso"),
    waiting: t("Waiting for you", "Espera tu respuesta"),
    completed: t("Completed", "Completada"),
    failed: t("Failed", "Fallida"),
    cancelled: t("Cancelled", "Cancelada"),
    interrupted: t("Interrupted", "Interrumpida"),
    active: t("Active", "Activa"),
    paused: t("Paused", "Pausada"),
    missed: t("Missed", "Vencida"),
  })[value] || value;

document.documentElement.dataset.theme =
  state.theme === "dark" ? "dark" : "light";
document.documentElement.lang = state.language;

async function api(path, options = {}) {
  const response = await fetch(`/api${path}`, {
    ...options,
    headers: { "Content-Type": "application/json", ...options.headers },
    body: options.body === undefined ? undefined : JSON.stringify(options.body),
  });
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

let toastTimer;
function toast(message) {
  const node = $("#toast");
  node.textContent = message;
  node.classList.add("visible");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => node.classList.remove("visible"), 6000);
}
function preferences() {
  return `<button type="button" data-action="language" class="subtle" aria-label="${t("Change language", "Cambiar idioma")}">${state.language === "es" ? "EN" : "ES"}</button><button type="button" data-action="theme" class="subtle" aria-label="${t("Toggle light and dark mode", "Cambiar modo claro y oscuro")}">${state.theme === "dark" ? "☀" : "☾"}</button>`;
}
function brand() {
  return '<div class="brand"><span class="brand-mark" aria-hidden="true">F</span>Fénix</div>';
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
function input(id, label, value = "", extra = "") {
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
    <p class="note">${t("Commands can use any directory accessible to your Linux account. The file workspace is not a shell sandbox.", "Los comandos pueden usar cualquier directorio accesible a tu cuenta de Linux. La carpeta de archivos no limita los comandos.")}</p></details>
    <div class="divider"></div><button class="primary full" type="submit">${setup ? t("Verify and start", "Verificar y comenzar") : t("Verify and save", "Verificar y guardar")} <span aria-hidden="true">→</span></button>
    <p id="model-status" class="form-status" role="status"></p>
    </form>`;
}
function policySelect(id, value) {
  return `<select id="${id}" name="${id}"><option value="ask" ${value !== "allow" ? "selected" : ""}>${t("Ask in the conversation", "Preguntar en la conversación")}</option><option value="allow" ${value === "allow" ? "selected" : ""}>${t("Allow for this local installation", "Permitir en esta instalación local")}</option></select>`;
}
function setupView() {
  $("#app").innerHTML =
    `<div class="setup-shell"><header class="setup-top">${brand()}<div class="row">${preferences()}</div></header><main class="setup-main"><section class="setup-copy"><div class="eyebrow">${t("Your assistant. Your space.", "Tu asistente. Tu espacio.")}</div><h1>${t("A little less to do.<br>A little more possible.", "Menos pendientes.<br>Más posibilidades.")}</h1><p>${t("Meet your personal assistant, running on your computer and ready to help with everyday work.", "Tu asistente personal, en tu computadora y listo para ayudarte con el trabajo de cada día.")}</p><div class="features"><div class="feature"><span>◻</span><div><strong>${t("One continuous conversation", "Una conversación continua")}</strong><p>${t("Pick up where you left off, here or on Telegram.", "Continúa donde lo dejaste, aquí o en Telegram.")}</p></div></div><div class="feature"><span>▤</span><div><strong>${t("Memory you control", "Memoria que tú controlas")}</strong><p>${t("Keep the details you explicitly choose to remember.", "Conserva los detalles que elijas recordar.")}</p></div></div><div class="feature"><span>◇</span><div><strong>${t("Built around your work", "Se adapta a tu trabajo")}</strong><p>${t("Local tools, manual skills and timely reminders.", "Herramientas locales, skills y recordatorios.")}</p></div></div></div></section>${modelForm(true)}</main><footer class="setup-footer">${t("Runs locally · Telegram is optional · Your key stays in the backend", "Ejecución local · Telegram es opcional · Tu clave permanece en el backend")}</footer></div>`;
  updateProviderFields(false);
}

async function render() {
  if (!state.status.ready) {
    setupView();
    return;
  }
  const labels = names();
  $("#app").innerHTML =
    `<div class="shell"><aside class="sidebar"><div class="sidebar-top">${brand()}<div class="mobile-brand">${preferences()}</div></div><div class="nav-label">${t("Workspace", "Tu espacio")}</div><nav class="nav" aria-label="${t("Main navigation", "Navegación principal")}">${Object.keys(
      labels,
    )
      .map(
        (page) =>
          `<button type="button" data-page="${page}" class="${state.page === page ? "active" : ""}" ${state.page === page ? 'aria-current="page"' : ""}><span class="nav-icon" aria-hidden="true">${icons[page]}</span>${labels[page]}</button>`,
      )
      .join(
        "",
      )}</nav><div class="sidebar-bottom"><div class="local-label"><span class="dot"></span>${t("Local installation", "Instalación local")}<br>America/Lima · Linux</div><div class="row">${preferences()}<small class="grow">v0.1.1</small></div></div></aside><main class="main"><header class="topbar"><div class="breadcrumb"><span>Fénix</span><span>/</span>${labels[state.page]}</div><span class="badge" id="connection-state">${escape(state.status.settings.model)}</span></header><div id="page"></div></main></div>`;
  const views = {
    chat: renderChat,
    memory: renderMemory,
    skills: renderSkills,
    schedules: renderSchedules,
    activity: renderActivity,
    settings: renderSettings,
  };
  try {
    await views[state.page]();
  } catch (error) {
    toast(error.message);
  }
}
function renderChat() {
  $("#page").innerHTML =
    `<section class="chat-layout"><div class="chat-scroll" id="chat-scroll"><div class="chat-inner"><button class="subtle load-older" data-action="older">${t("Earlier messages", "Mensajes anteriores")}</button><div id="message-list" role="log" aria-live="polite" aria-relevant="additions"></div><div id="live-message" aria-busy="true"></div></div></div><div class="composer-area"><div class="composer-wrap"><div class="activity-strip" id="activity-strip" role="status"></div><form id="chat-form" class="composer"><label class="muted" for="chat-input" hidden>${t("Message Fénix", "Escribe a Fénix")}</label><textarea id="chat-input" placeholder="${t("What can I help you with?", "¿En qué te ayudo?")}" maxlength="32000" rows="2" required></textarea><div class="composer-footer"><small>${escape(state.status.settings.model)} · ${t("Text only", "Sólo texto")}</small><button class="primary send" type="submit" aria-label="${t("Send message", "Enviar mensaje")}">↑</button></div></form><p class="composer-hint">${t("One conversation, always. New requests wait their turn. Enter to send · Shift + Enter for a new line.", "Una sola conversación, siempre. Los nuevos mensajes esperan su turno. Enter para enviar · Shift + Enter para nueva línea.")}</p></div></div></section>`;
  refreshMessages(true);
  updateActivityStrip();
  drawLiveMessage();
}
async function refreshMessages(force = false) {
  const messages = await api("/messages");
  const changed = messages.some((message) => !state.messages.has(message.id));
  for (const message of messages) state.messages.set(message.id, message);
  if (!$("#message-list") || (!changed && !force)) return;
  const scroll = $("#chat-scroll");
  const nearBottom =
    scroll.scrollHeight - scroll.scrollTop - scroll.clientHeight < 100;
  drawMessages();
  drawLiveMessage();
  if (nearBottom || force) scroll.scrollTop = scroll.scrollHeight;
}
function drawMessages() {
  const messages = [...state.messages.values()].sort((a, b) => a.seq - b.seq);
  $(".load-older").hidden = !messages.length;
  $("#message-list").innerHTML = messages.length
    ? messages
        .map(
          (message) =>
            `<article class="message ${message.role}"><span class="avatar" aria-hidden="true">${message.role === "user" ? t("Y", "T") : "F"}</span><div class="message-body"><div class="message-head">${message.role === "user" ? t("You", "Tú") : "Fénix"}<time datetime="${escape(message.created_at)}">${date(message.created_at)}</time>${message.origin === "telegram" ? "<small>Telegram</small>" : ""}</div><div class="message-text">${escape(message.text)}</div>${message.task_id ? `<div class="message-meta">${t("Task", "Tarea")} ${escape(message.task_id.slice(0, 8))}</div>` : ""}</div></article>`,
        )
        .join("")
    : `<div class="chat-welcome"><span class="brand-mark">F</span><h1>${t("A good place to begin.", "Empecemos por lo que necesitas.")}</h1><p>${t("Tell me what you would like to get done. I can work with local files, remember important details and keep track of your reminders.", "Dime qué te gustaría resolver. Puedo trabajar con tus archivos, recordar detalles importantes y gestionar tus recordatorios.")}</p><div class="suggestions">${[
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
          ([title, prompt]) =>
            `<button class="suggestion" data-prompt="${escape(prompt)}"><strong>${title}</strong>${escape(prompt)}</button>`,
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
    command_cancel: ["Cancelling a command", "Cancelando un comando"],
    memory_search: ["Searching memory", "Buscando en la memoria"],
    memory_manage: ["Managing memory", "Gestionando la memoria"],
    history_search: [
      "Searching the conversation",
      "Buscando en la conversación",
    ],
    history_read: ["Reading the history", "Leyendo el historial"],
    skill_load: ["Loading a skill", "Cargando un skill"],
    schedule_manage: ["Managing schedules", "Gestionando tareas programadas"],
    task_inspect: ["Inspecting a task", "Consultando una tarea"],
    ask_user: ["Preparing a question", "Preparando una pregunta"],
    request_approval: [
      "Preparing an approval request",
      "Preparando una solicitud de aprobación",
    ],
  };
  const label = labels[progress.tool];
  return label ? t(...label) : "";
}
function updateActivityStrip() {
  const node = $("#activity-strip");
  if (!node) return;
  const queued = state.tasks.filter((task) => task.status === "queued").length;
  const live = progressLabel();
  const activity =
    live ||
    (state.status.pending
      ? t(
          "Waiting for your textual answer. Follow the instructions in the conversation.",
          "Espera tu respuesta por texto. Sigue las instrucciones de la conversación.",
        )
      : state.status.active_task
        ? t("Working on your request", "Trabajando en tu solicitud")
        : "");
  node.textContent = [
    activity,
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
    node.innerHTML =
      '<article class="message assistant"><span class="avatar" aria-hidden="true">F</span><div class="message-body"><div class="message-head">Fénix <small class="live-label"></small></div><div class="message-text"></div></div></article>';
  }
  node.querySelector(".live-label").textContent = state.progressConnected
    ? t("Writing…", "Escribiendo…")
    : t("Reconnecting…", "Reconectando…");
  node.querySelector(".message-text").textContent = text;
  if (nearBottom) scroll.scrollTop = scroll.scrollHeight;
}
let progressSource;
function connectProgress() {
  progressSource?.close();
  progressSource = new EventSource("/api/progress");
  progressSource.addEventListener("progress", (event) => {
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
window.addEventListener("pageshow", (event) => {
  if (event.persisted) connectProgress();
});

async function renderMemory() {
  $("#page").innerHTML =
    `<div class="content">${heading(t("A memory you can trust.", "Lo importante, a mano."), t("Only the facts you ask to keep. Search, correct or remove them at any time.", "Sólo los datos que pides conservar. Búscalos, corrígelos o elimínalos cuando quieras."))}<div class="grid"><section class="card"><h2>${t("Saved memories", "Recuerdos guardados")}</h2><form class="search-form" id="memory-search"><input id="memory-query" aria-label="${t("Search memory", "Buscar recuerdos")}" placeholder="${t("Search by words, title or ID…", "Buscar por palabras, título o ID…")}"><button type="submit">${t("Search", "Buscar")}</button></form><div id="memory-list"></div></section><form class="card" id="memory-form"><h2>${t("Keep a detail", "Guardar un detalle")}</h2><input type="hidden" id="memory-id">${input("memory-title", t("Title", "Título"), "", "required maxlength=200")}${field(`<label for="memory-content">${t("What should I remember?", "¿Qué debo recordar?")}</label>`, '<textarea id="memory-content" required maxlength="16000" rows="5"></textarea>')}<button class="primary" type="submit">${t("Save memory", "Guardar recuerdo")}</button><button class="subtle" data-action="clear-memory" type="button">${t("Clear form", "Limpiar formulario")}</button><p class="form-status" id="memory-status"></p></form></div></div>`;
  await loadMemories();
}
let memories = [];
async function loadMemories(query = "") {
  memories = await api(`/memory?query=${encodeURIComponent(query)}`);
  if (!$("#memory-list")) return;
  $("#memory-list").innerHTML = memories.length
    ? memories
        .map(
          (memory) =>
            `<article class="record"><h3>${escape(memory.title)}</h3><p>${escape(memory.content)}</p><small>${date(memory.updated_at)}<br>${escape(memory.source)}<br>${escape(memory.id)}</small><div class="actions"><button data-edit-memory="${memory.id}">${t("Edit", "Editar")}</button><button data-delete-memory="${memory.id}">${t("Remove memory", "Eliminar recuerdo")}</button></div></article>`,
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
  const skills = await api("/skills");
  $("#page").innerHTML =
    `<div class="content">${heading(t("A familiar way to work.", "Una forma de trabajar, definida."), t("Manual skills give Fénix repeatable instructions for specific tasks. You choose which ones are active.", "Los skills manuales guían tareas específicas. Tú eliges cuáles están activos."))}<div class="grid">${skills.map((skill) => `<article class="card"><div class="skill-title"><h2>${escape(skill.name)}</h2><span class="badge">v${escape(skill.version)}</span></div><p class="muted">${escape(skill.description)}</p><div class="row spread"><label class="toggle"><input type="checkbox" data-skill="${escape(skill.name)}" ${skill.enabled ? "checked" : ""}>${t("Enabled", "Activo")}</label><button data-read-skill="${escape(skill.name)}" ${skill.enabled ? "" : "disabled"}>${t("Read instructions", "Ver instrucciones")}</button></div></article>`).join("")}</div><p class="note">${t("Install your own skill in", "Instala tus propios skills en")} <code>${escape(state.status.skills_directory)}/&lt;name&gt;/SKILL.md</code>. ${t("Skills are never created or rewritten automatically.", "Los skills nunca se crean ni reescriben automáticamente.")}</p></div>`;
}

let schedules = [];
async function renderSchedules() {
  schedules = await api("/schedules");
  $("#page").innerHTML =
    `<div class="content">${heading(t("The right thing, at the right time.", "Cada cosa, a su tiempo."), t("Reminders and recurring work in Peru time. Your computer and Fénix must be running.", "Recordatorios y trabajo recurrente en horario de Perú. Tu computadora y Fénix deben estar encendidos."))}<div class="grid"><section class="card"><h2>${t("Your schedules", "Tus programaciones")}</h2>${schedules.length ? schedules.map((schedule) => `<article class="record"><div class="row spread"><h3>${escape(schedule.name)}</h3><span class="badge">${statusName(schedule.status)}</span></div><p>${escape(schedule.content)}</p><small>${t("Next", "Próxima")}: ${date(schedule.next_run)} · America/Lima<br>${escape(schedule.kind)} · ${escape(schedule.mode)}<br>${escape(schedule.id)}${schedule.last_result ? `<br>${escape(schedule.last_result)}` : ""}</small><div class="actions"><button data-edit-schedule="${schedule.id}">${t("Edit", "Editar")}</button>${["active", "paused"].includes(schedule.status) ? `<button data-schedule-state="${schedule.status === "active" ? "pause" : "resume"}" data-id="${schedule.id}">${schedule.status === "active" ? t("Pause", "Pausar") : t("Resume future runs", "Reactivar futuras ejecuciones")}</button>` : ""}</div></article>`).join("") : empty(t("A little space in your day", "Un pendiente menos en tu cabeza"), t("Create a reminder here or ask for one in the conversation.", "Crea un recordatorio aquí o pídelo en la conversación."))}<p class="note">${t("To cancel a schedule, ask in the conversation and include its name or ID.", "Para cancelar una programación, pídelo en la conversación e indica su nombre o ID.")}</p></section><form class="card" id="schedule-form"><h2>${t("Create a schedule", "Crear programación")}</h2><input type="hidden" id="schedule-id">${input("schedule-name", t("Name", "Nombre"), "", "required maxlength=200")}${field(`<label for="schedule-content">${t("Reminder or task", "Recordatorio o tarea")}</label>`, '<textarea id="schedule-content" required maxlength="16000"></textarea>')}${field(`<label for="schedule-mode">${t("What happens", "Qué ocurre")}</label>`, `<select id="schedule-mode"><option value="reminder">${t("Deliver a reminder", "Entregar un recordatorio")}</option><option value="task">${t("Run an agent task", "Ejecutar una tarea del agente")}</option></select>`)}${field(`<label for="schedule-kind">${t("When", "Cuándo")}</label>`, `<select id="schedule-kind"><option value="once">${t("Once", "Una vez")}</option><option value="interval">${t("Every interval", "Por intervalo")}</option><option value="cron">Cron</option></select>`)}<div id="schedule-once">${input("schedule-at", t("Date and time · Peru", "Fecha y hora · Perú"), "", 'type="datetime-local"')}</div><div id="schedule-interval" hidden>${input("schedule-seconds", t("Interval in seconds", "Intervalo en segundos"), 3600, 'type="number" min="1" max="31536000"')}</div><div id="schedule-cron" hidden>${input("schedule-expression", t("Cron expression", "Expresión cron"), "0 9 * * 1-5")}</div><button class="primary" type="submit">${t("Save schedule", "Guardar programación")}</button><p id="schedule-status" class="form-status"></p></form></div></div>`;
}

async function renderActivity() {
  state.tasks = await api("/tasks");
  $("#page").innerHTML =
    `<div class="content">${heading(t("Every step, accounted for.", "Cada paso, a la vista."), t("Inspect task states, model usage and tool evidence. To cancel, type cancelar followed by the task ID in the conversation.", "Consulta estados, uso del modelo y evidencia de herramientas. Para cancelar, escribe cancelar seguido del ID de la tarea en la conversación."))}<section class="card"><div class="row spread section-title"><h2>${t("Task history", "Historial de tareas")}</h2><button data-action="refresh-activity">${t("Refresh", "Actualizar")}</button></div>${state.tasks.length ? `<div class="table-wrap"><table class="table"><thead><tr><th>${t("Request", "Solicitud")}</th><th>${t("State", "Estado")}</th><th>${t("Created", "Creada")}</th><th>${t("Details", "Detalles")}</th></tr></thead><tbody>${state.tasks.map((task) => `<tr><td><div class="task-request">${escape(task.request)}</div><small><code>${task.id}</code></small></td><td><span class="badge">${statusName(task.status)}</span></td><td>${date(task.created_at)}<br><small>${escape(task.origin)}</small></td><td><button data-task="${task.id}">${t("Inspect", "Inspeccionar")}</button></td></tr>`).join("")}</tbody></table></div>` : empty(t("Ready for the first task", "Todo listo para la primera tarea"), t("Send a message to see its progress here.", "Envía un mensaje para ver su progreso aquí."))}</section></div>`;
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
    `<p>${escape(value.task.request)}</p><p><span class="badge">${statusName(value.task.status)}</span></p><code>${escape(id)}</code>${value.task.error ? `<p class="note">${escape(value.task.error)}</p>` : ""}${value.events.map((event) => `<details class="record"><summary>${escape(event.kind)} · ${date(event.created_at)}</summary><pre>${escape(JSON.stringify(event.data, null, 2))}</pre></details>`).join("")}${value.events.length === 50 ? `<button data-task="${id}" data-after="${value.events.at(-1).seq}">${t("More events", "Más eventos")}</button>` : ""}`,
  );
}
function showDetail(title, html) {
  $("#detail-title").textContent = title;
  $("#detail-content").innerHTML = html;
  if (!$("#detail-dialog").open) $("#detail-dialog").showModal();
}
$("#close-dialog").addEventListener("click", () => $("#detail-dialog").close());

document.addEventListener("click", async (event) => {
  const button = event.target.closest("button");
  if (!button) return;
  try {
    if (button.dataset.page) {
      state.page = button.dataset.page;
      await render();
    }
    if (button.dataset.action === "theme") {
      state.theme = state.theme === "dark" ? "light" : "dark";
      localStorage.setItem("fenix.theme", state.theme);
      document.documentElement.dataset.theme = state.theme;
      document.querySelectorAll('[data-action="theme"]').forEach((node) => {
        node.textContent = state.theme === "dark" ? "☀" : "☾";
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
    if (button.dataset.editMemory) {
      const memory = memories.find((m) => m.id === button.dataset.editMemory);
      $("#memory-id").value = memory.id;
      $("#memory-title").value = memory.title;
      $("#memory-content").value = memory.content;
      $("#memory-title").focus();
    }
    if (button.dataset.deleteMemory) {
      await api(`/memory/${button.dataset.deleteMemory}`, {
        method: "DELETE",
        body: {},
      });
      await loadMemories();
      toast(t("Memory removed.", "Recuerdo eliminado."));
    }
    if (button.dataset.action === "clear-memory") $("#memory-form").reset();
    if (button.dataset.readSkill) {
      const result = await api(`/skills/${button.dataset.readSkill}`);
      showDetail(
        button.dataset.readSkill,
        `<pre>${escape(result.content)}</pre>`,
      );
    }
    if (button.dataset.editSchedule) {
      const schedule = schedules.find(
        (s) => s.id === button.dataset.editSchedule,
      );
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
    toast(error.message);
  }
});
document.addEventListener("change", async (event) => {
  try {
    if (event.target.id === "provider") updateProviderFields(true);
    if (event.target.id === "schedule-kind") scheduleKind();
    if (event.target.id === "instruction-name") await loadInstruction();
    if (event.target.dataset.skill) {
      await api(`/skills/${event.target.dataset.skill}`, {
        method: "PUT",
        body: { enabled: event.target.checked },
      });
      await renderSkills();
    }
  } catch (error) {
    toast(error.message);
  }
});
document.addEventListener("keydown", (event) => {
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
document.addEventListener("submit", async (event) => {
  event.preventDefault();
  const form = event.target;
  const button = form.querySelector('[type="submit"]');
  const original = button?.textContent;
  if (button?.disabled) return;
  if (button) button.disabled = true;
  try {
    if (form.id === "model-form") {
      $("#model-status").textContent = t(
        "Checking a real inference request. This may take a moment…",
        "Comprobando una llamada real al modelo. Puede tardar un momento…",
      );
      $("#model-status").classList.remove("error");
      const fields = Object.fromEntries(new FormData(form));
      for (const key of [
        "base_url",
        "project",
        "provider_project",
        "organization",
        "reasoning_effort",
      ])
        if (!fields[key]) fields[key] = null;
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
      const text = $("#chat-input").value.trim();
      if (!text) return;
      await api("/messages", { method: "POST", body: { text } });
      $("#chat-input").value = "";
      await refreshMessages(true);
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
        },
      });
      form.reset();
      $("#memory-status").textContent = t(
        "Memory saved and verified.",
        "Recuerdo guardado y verificado.",
      );
      await loadMemories();
    }
    if (form.id === "schedule-form") {
      const kind = $("#schedule-kind").value;
      const id = $("#schedule-id").value;
      const body = {
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
      target.textContent = error.message;
      target.classList.add("error");
    } else toast(error.message);
  } finally {
    if (button?.isConnected) {
      button.disabled = false;
      button.textContent = original;
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
    if (status.ready !== before) await render();
    if (state.page === "chat" && status.ready) {
      await refreshMessages();
      updateActivityStrip();
    }
    if ($("#connection-state"))
      $("#connection-state").textContent = status.settings?.model || "Fénix";
  } catch {
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
  try {
    [state.status, state.providers] = await Promise.all([
      api("/status"),
      api("/providers"),
    ]);
    await render();
    connectProgress();
    setInterval(poll, 1800);
  } catch (error) {
    $("#app").innerHTML =
      `<main class="boot">${brand()}<p>${t("Unable to reach Fénix", "No se pudo conectar con Fénix")}</p><span>${escape(error.message)}</span><button data-action="retry">${t("Try again", "Reintentar")}</button></main>`;
    $('[data-action="retry"]').addEventListener("click", boot);
  }
}
await boot();
