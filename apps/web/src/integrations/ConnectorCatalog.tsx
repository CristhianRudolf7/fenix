import { Loading } from "../components/loading";
import { useEffect, useState, type ReactNode } from "react";
import { api } from "../live/api";
import { tr } from "../workflows/types";
import "../workflows/workflows.css";
import { ProviderIcon } from "./ProviderIcon";
type Provider = {
  id: string;
  name: string;
  category: string;
  auth: string;
  services?: string[];
  service_options?: { id: string; name: string }[];
  operations?: string[];
  tools?: string[];
  route?: string;
  connect_via?: string[];
};
type Connection = {
  id: string;
  provider: string;
  name: string;
  state: string;
  allow_write: boolean;
  checked_at?: string;
  service_checks?: Record<string, { status: string; detail: string }>;
  settings?: {
    services?: string[];
    path?: string;
    scopes?: string;
    imap_host?: string;
    smtp_host?: string;
    smtp_starttls?: boolean;
  };
};
type Catalog = { catalog: Provider[]; connections: Connection[] };
export function ConnectorCatalog({
  language,
  whatsappCard,
}: {
  language: string;
  whatsappCard?: ReactNode;
}) {
  const [initialLoading, setInitialLoading] = useState(true);
  const t = tr(language),
    [data, setData] = useState<Catalog>({ catalog: [], connections: [] }),
    [error, setError] = useState(""),
    [search, setSearch] = useState("");
  const [selected, setSelected] = useState<Provider | null>(null),
    [editing, setEditing] = useState<Connection | null>(null),
    [busy, setBusy] = useState<string | null>(null);
  const load = async (signal?: AbortSignal) => {
    const value = await api<Catalog>("/connectors", undefined, "GET", signal);
    if (!signal?.aborted && Array.isArray(value.catalog)) setData(value);
  };
  useEffect(() => {
    const controller = new AbortController();
    void load(controller.signal)
      .catch((e) => {
        if (!controller.signal.aborted) setError(String(e));
      })
      .finally(() => {
        if (!controller.signal.aborted) setInitialLoading(false);
      });
    return () => controller.abort();
  }, []);
  async function action(id: string, action: string) {
    setBusy(id);
    setError("");
    try {
      await api(`/connectors/${id}/${action}`, {});
      await load();
    } catch (e) {
      setError(String(e));
      await load().catch(() => {});
    } finally {
      setBusy(null);
    }
  }
  const labels: Record<string, string> = {
    connected: t("Connected", "Conectada"),
    unverified: t("Not verified", "Sin verificar"),
    needs_attention: t("Needs attention", "Necesita atención"),
    authorized: t(
      "Authorized · checks pending",
      "Autorizada · comprobación pendiente",
    ),
    partially_checked: t("Partially checked", "Comprobada parcialmente"),
  };
  if (initialLoading) return <Loading />;
  return (
    <div className="connector-catalog">
      <label className="connector-search">
        <span>
          {t(
            "Find a connection or capability",
            "Busca una conexión o capacidad",
          )}
        </span>
        <input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Google Drive, GitHub, PDF…"
        />
      </label>
      {error && (
        <p role="alert" className="flow-error">
          {error}
        </p>
      )}
      <div className="connector-grid">
        {(data.catalog.some((p) => p.id === "whatsapp")
          ? data.catalog
          : [
              ...data.catalog,
              {
                id: "whatsapp",
                name: "WhatsApp",
                category: "communication",
                auth: "existing",
                services: ["WhatsApp Web"],
              },
            ]
        )
          .filter((p) =>
            `${p.name} ${p.services?.join(" ") || ""}`
              .toLowerCase()
              .includes(search.toLowerCase()),
          )
          .map((provider) => {
            if (provider.id === "whatsapp")
              return (
                <div className="connector-slot" key={provider.id}>
                  {whatsappCard}
                </div>
              );
            const accounts = data.connections.filter(
                (c) => c.provider === provider.id,
              ),
              builtin = provider.auth === "builtin",
              linked = provider.auth === "linked",
              external = !builtin && !linked && provider.auth !== "existing";
            return (
              <article className="connector-card" key={provider.id}>
                <div className="connector-title">
                  <span className="connector-symbol">
                    <ProviderIcon provider={provider.id} />
                  </span>
                  <div>
                    <h2>{provider.name}</h2>
                    <small className="muted">
                      {provider.services?.join(" · ") ||
                        (builtin
                          ? t("Built into Fénix", "Incluido en Fénix")
                          : provider.category)}
                    </small>
                  </div>
                  {builtin && (
                    <span className="flow-state completed">
                      {t("Available", "Disponible")}
                    </span>
                  )}
                </div>
                {accounts.map((connection) => (
                  <div className="connector-account" key={connection.id}>
                    <strong>{connection.name}</strong>
                    <span
                      className={`flow-state ${connection.state === "connected" ? "completed" : "paused"}`}
                    >
                      {labels[connection.state] || connection.state}
                    </span>
                    <small>
                      {connection.allow_write
                        ? t("Read and write", "Lectura y escritura")
                        : t("Read only", "Solo lectura")}
                    </small>
                    {connection.service_checks && (
                      <details className="connector-checks">
                        <summary>
                          {t("Service checks", "Comprobación por servicio")}
                        </summary>
                        <ul>
                          {Object.entries(connection.service_checks).map(
                            ([service, check]) => (
                              <li key={service} title={check.detail}>
                                <strong>
                                  {provider.service_options?.find(
                                    (s) => s.id === service,
                                  )?.name || service}
                                </strong>
                                :{" "}
                                {check.status === "passed"
                                  ? t(
                                      "Read access checked",
                                      "Lectura comprobada",
                                    )
                                  : check.status === "not_checked"
                                    ? t(
                                        "Check a specific document",
                                        "Falta comprobar un documento concreto",
                                      )
                                    : t("Needs attention", "Necesita atención")}
                              </li>
                            ),
                          )}
                        </ul>
                        <small>
                          {t(
                            "Checks do not confirm write access or access to every file.",
                            "La comprobación no confirma escritura ni acceso a todos los archivos.",
                          )}
                        </small>
                      </details>
                    )}
                    <div className="flow-actions">
                      <button
                        disabled={busy === connection.id}
                        onClick={() => void action(connection.id, "test")}
                      >
                        {t("Check connection", "Comprobar conexión")}
                      </button>
                      <button
                        onClick={() => {
                          setEditing(connection);
                          setSelected(provider);
                        }}
                      >
                        {t("Configure", "Configurar")}
                      </button>
                      <button
                        disabled={busy === connection.id}
                        onClick={() => void action(connection.id, "forget")}
                      >
                        {t("Disconnect", "Desconectar")}
                      </button>
                    </div>
                  </div>
                ))}
                {external && (
                  <button
                    className="connector-connect"
                    onClick={() => {
                      setEditing(null);
                      setSelected(provider);
                    }}
                  >
                    +{" "}
                    {accounts.length
                      ? t("Add another account", "Añadir otra cuenta")
                      : t("Connect", "Conectar")}
                  </button>
                )}
                {linked && (
                  <p>
                    {t(
                      "Use a connected Google Workspace or Microsoft 365 account to read and send email.",
                      "Usa una cuenta conectada de Google Workspace o Microsoft 365 para leer y enviar correo.",
                    )}
                  </p>
                )}
                {provider.route && (
                  <a className="connector-connect" href={provider.route}>
                    {t("Configure", "Configurar")} ↗
                  </a>
                )}
                {builtin && (
                  <p className="muted">
                    {t(
                      "Use it in the conversation or as a workflow step. No extra provider account is required.",
                      "Úsalo en la conversación o como paso de un flujo. No requiere una cuenta de otro proveedor.",
                    )}
                  </p>
                )}
                <details>
                  <summary>{t("Capabilities", "Capacidades")}</summary>
                  <ul>
                    {(
                      provider.operations ||
                      provider.tools ||
                      provider.services ||
                      []
                    ).map((op) => (
                      <li key={op}>{op.replaceAll("_", " ")}</li>
                    ))}
                  </ul>
                </details>
              </article>
            );
          })}
      </div>
      {selected && (
        <ConnectDialog
          provider={selected}
          connection={editing}
          language={language}
          onClose={() => setSelected(null)}
          onSaved={async () => {
            setSelected(null);
            await load();
          }}
        />
      )}
    </div>
  );
}
function ConnectDialog({
  provider,
  connection,
  language,
  onClose,
  onSaved,
}: {
  provider: Provider;
  connection: Connection | null;
  language: string;
  onClose: () => void;
  onSaved: () => Promise<void>;
}) {
  const t = tr(language),
    [name, setName] = useState(connection?.name || provider.name),
    [mode, setMode] = useState(
      provider.auth.startsWith("oauth") ? "oauth" : "token",
    );
  const [savedId, setSavedId] = useState(connection?.id);
  const [services, setServices] = useState<string[]>(
    connection?.settings?.services ||
      (connection
        ? ["drive", "docs", "sheets", "slides", "gmail", "calendar"]
        : ["drive"]),
  );
  const [token, setToken] = useState(""),
    [clientId, setClientId] = useState(""),
    [clientSecret, setClientSecret] = useState(""),
    [scopes, setScopes] = useState(connection?.settings?.scopes || "");
  const [mailUser, setMailUser] = useState(""),
    [mailPassword, setMailPassword] = useState(""),
    [imapHost, setImapHost] = useState(connection?.settings?.imap_host || ""),
    [smtpHost, setSmtpHost] = useState(connection?.settings?.smtp_host || ""),
    [starttls, setStarttls] = useState(
      connection?.settings?.smtp_starttls || false,
    );
  const [path, setPath] = useState(connection?.settings?.path || ""),
    [database, setDatabase] = useState(""),
    [write, setWrite] = useState(connection?.allow_write || false),
    [busy, setBusy] = useState(false),
    [error, setError] = useState("");
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !busy) onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [busy, onClose]);
  async function connect() {
    setBusy(true);
    setError("");
    try {
      const credentials: Record<string, string> = {},
        settings: Record<string, unknown> = {};
      if (token) credentials.access_token = token;
      if (clientId) credentials.client_id = clientId;
      if (clientSecret) credentials.client_secret = clientSecret;
      if (database) credentials.url = database;
      if (scopes) settings.scopes = scopes;
      if (provider.id === "google") settings.services = services;
      if (path) settings.path = path;
      if (mailUser) credentials.username = mailUser;
      if (mailPassword) credentials.password = mailPassword;
      if (provider.id === "email") {
        settings.imap_host = imapHost;
        settings.smtp_host = smtpHost;
        settings.smtp_starttls = starttls;
      }
      const saved = await api<Connection>("/connectors", {
        id: savedId,
        provider: provider.id,
        name,
        credentials,
        settings,
        allow_write: write,
      });
      setSavedId(saved.id);
      if (provider.auth.startsWith("oauth") && mode === "oauth") {
        const result = await api<{ url: string }>(
          `/connectors/${saved.id}/oauth`,
          {},
        );
        window.location.assign(result.url);
      } else {
        await api(`/connectors/${saved.id}/test`, {});
        await onSaved();
      }
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
        aria-labelledby="connect-dialog-title"
      >
        <header>
          <h2 id="connect-dialog-title">
            {t("Connect", "Conectar")} {provider.name}
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
            void connect();
          }}
        >
          <label>
            {t("Account name", "Nombre de la cuenta")}
            <input
              autoFocus
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </label>
          {provider.auth.startsWith("oauth") && (
            <label>
              {t("Connection method", "Método de conexión")}
              <select value={mode} onChange={(e) => setMode(e.target.value)}>
                <option value="oauth">
                  {t("Authorize my account", "Autorizar mi cuenta")}
                </option>
                <option value="token">
                  {t("Existing access token", "Token de acceso existente")}
                </option>
              </select>
            </label>
          )}
          {provider.id === "google" && (
            <fieldset className="connector-services">
              <legend>{t("Google services", "Servicios de Google")}</legend>
              <div>
                {(
                  provider.service_options || [
                    { id: "drive", name: "Drive" },
                    { id: "docs", name: "Docs" },
                    { id: "sheets", name: "Sheets" },
                    { id: "slides", name: "Slides" },
                    { id: "gmail", name: "Gmail" },
                    { id: "calendar", name: "Calendar" },
                  ]
                ).map((service) => (
                  <label className="connector-permission" key={service.id}>
                    <input
                      type="checkbox"
                      checked={services.includes(service.id)}
                      onChange={(e) =>
                        setServices((current) =>
                          e.target.checked
                            ? [...current, service.id]
                            : current.filter((id) => id !== service.id),
                        )
                      }
                    />
                    {service.name}
                  </label>
                ))}
              </div>
              <small>
                {t(
                  "Select the services you need. New permissions require Google authorization.",
                  "Selecciona los servicios que necesitas. Los permisos nuevos requieren autorización de Google.",
                )}
              </small>
              {scopes.trim() && (
                <small>
                  {t(
                    "Custom scopes override the suggested permissions. Selected services still limit Fénix operations.",
                    "Los scopes personalizados reemplazan los permisos sugeridos. Los servicios seleccionados siguen limitando las operaciones de Fénix.",
                  )}
                </small>
              )}
            </fieldset>
          )}
          {provider.id === "email" ? (
            <>
              <label>
                {t("Email / username", "Correo / usuario")}
                <input
                  autoComplete="off"
                  type="email"
                  required={!connection}
                  value={mailUser}
                  onChange={(e) => setMailUser(e.target.value)}
                />
              </label>
              <label>
                {t("Application password", "Contraseña de aplicación")}
                <input
                  type="password"
                  autoComplete="new-password"
                  required={!connection}
                  value={mailPassword}
                  onChange={(e) => setMailPassword(e.target.value)}
                />
              </label>
              <label>
                {t(
                  "IMAP server (TLS, port 993)",
                  "Servidor IMAP (TLS, puerto 993)",
                )}
                <input
                  required
                  value={imapHost}
                  onChange={(e) => setImapHost(e.target.value)}
                  placeholder="imap.example.com"
                />
              </label>
              <label>
                {t("SMTP server", "Servidor SMTP")}
                <input
                  required
                  value={smtpHost}
                  onChange={(e) => setSmtpHost(e.target.value)}
                  placeholder="smtp.example.com"
                />
              </label>
              <label className="connector-permission">
                <input
                  type="checkbox"
                  checked={starttls}
                  onChange={(e) => setStarttls(e.target.checked)}
                />
                STARTTLS · 587{" "}
                {t("(otherwise TLS · 465)", "(si no, TLS · 465)")}
              </label>
              <p className="muted">
                {t(
                  "For Gmail and Outlook, you can also use Google Workspace or Microsoft 365 OAuth.",
                  "Para Gmail y Outlook también puedes usar OAuth de Google Workspace o Microsoft 365.",
                )}
              </p>
            </>
          ) : provider.id === "sqlite" ? (
            <label>
              {t(
                "Database path in your workspace",
                "Ruta de la base de datos en tu espacio",
              )}
              <input
                required
                placeholder="data/reports.sqlite"
                value={path}
                onChange={(e) => setPath(e.target.value)}
              />
              <small>
                {t(
                  "Choose an existing database belonging to this account.",
                  "Elige una base existente que pertenezca a esta cuenta.",
                )}
              </small>
            </label>
          ) : provider.id === "postgresql" ? (
            <label>
              {t("PostgreSQL connection URL", "URL de conexión de PostgreSQL")}
              <input
                type="password"
                autoComplete="off"
                required={!connection}
                placeholder="postgresql://user:password@host/database"
                value={database}
                onChange={(e) => setDatabase(e.target.value)}
              />
              <small>
                {t(
                  "Verified TLS is required. Passwords stay out of chat and workflow diagrams.",
                  "Se requiere TLS verificado. Las contraseñas no se incluyen en el chat ni en los diagramas.",
                )}
              </small>
            </label>
          ) : mode === "oauth" ? (
            <>
              <p>
                {t(
                  "Continue to the provider to choose your account and authorize access. Your administrator can configure the OAuth application once for this installation.",
                  "Continúa al proveedor para elegir tu cuenta y autorizar el acceso. El administrador puede configurar la aplicación OAuth una vez para esta instalación.",
                )}
              </p>
              <details>
                <summary>
                  {t("Application settings", "Configuración de la aplicación")}
                </summary>
                <label>
                  Client ID
                  <input
                    autoComplete="off"
                    value={clientId}
                    onChange={(e) => setClientId(e.target.value)}
                  />
                </label>
                <label>
                  Client secret
                  <input
                    type="password"
                    autoComplete="new-password"
                    value={clientSecret}
                    onChange={(e) => setClientSecret(e.target.value)}
                  />
                </label>
                <label>
                  {t(
                    "Requested permissions (scopes)",
                    "Permisos solicitados (scopes)",
                  )}
                  <textarea
                    value={scopes}
                    onChange={(e) => setScopes(e.target.value)}
                  />
                </label>
                <p className="muted">
                  {t(
                    "Register this redirect URL",
                    "Registra esta URL de retorno",
                  )}
                  : <code>{location.origin}/api/connectors/oauth/callback</code>
                </p>
              </details>
            </>
          ) : (
            <label>
              {t("Access token", "Token de acceso")}
              <input
                type="password"
                autoComplete="new-password"
                required={!connection}
                value={token}
                onChange={(e) => setToken(e.target.value)}
              />
              <small>
                {t(
                  "Tokens with an expiration require reconnection. OAuth can renew access automatically.",
                  "Los tokens que caducan requieren reconexión. OAuth puede renovar el acceso automáticamente.",
                )}
              </small>
            </label>
          )}
          <label className="connector-permission">
            <input
              type="checkbox"
              checked={write}
              onChange={(e) => setWrite(e.target.checked)}
            />
            {t(
              "Allow write operations for requested tasks",
              "Permitir escritura para las tareas solicitadas",
            )}
          </label>
          <p className="muted">
            {t(
              "This connection belongs only to your Fénix account.",
              "Esta conexión pertenece únicamente a tu cuenta de Fénix.",
            )}
          </p>
          {error && (
            <p role="alert" className="flow-error">
              {error}
            </p>
          )}
          <button
            className="primary"
            disabled={
              busy || (provider.id === "google" && services.length === 0)
            }
          >
            {busy
              ? t("Connecting…", "Conectando…")
              : mode === "oauth" && provider.auth.startsWith("oauth")
                ? t("Continue to authorization", "Continuar a la autorización")
                : t("Save and check", "Guardar y comprobar")}
          </button>
        </form>
      </section>
    </div>
  );
}
