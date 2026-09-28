import { useEffect, useState, type FormEvent } from "react";
import { createRoot, type Root } from "react-dom/client";
import { Icon } from "../components/icons";
import { Loading } from "../components/loading";
import "./profile.css";

type ProfileData = { email: string; display_name: string };
type Usage = {
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  requests: number;
  missing_usage: number;
};
type Report = {
  days: number;
  timezone: string;
  totals: Usage;
  daily: { date: string; usage: Usage }[];
};
async function api<T>(url: string, init: RequestInit = {}): Promise<T> {
  const response = await fetch(url, {
    cache: "no-store",
    ...init,
    headers: { "Content-Type": "application/json", ...init.headers },
  });
  if (response.status === 401) {
    location.reload();
    throw Error("session_expired");
  }
  const body = await response.json().catch(() => ({}));
  if (!response.ok)
    throw Error(
      body.error || (response.status === 404 ? "local_account" : "unavailable"),
    );
  return body as T;
}
function Profile({ language }: { language: string }) {
  const t = (en: string, es: string) => (language === "es" ? es : en);
  const [signingOut, setSigningOut] = useState(false);
  const [logoutError, setLogoutError] = useState("");
  async function signOut() {
    setSigningOut(true);
    setLogoutError("");
    try {
      await api("/api/auth/logout", { method: "POST", body: "{}" });
      location.reload();
    } catch {
      setLogoutError(
        t(
          "Could not sign out. Try again.",
          "No se pudo cerrar sesión. Inténtalo de nuevo.",
        ),
      );
      setSigningOut(false);
    }
  }
  const [tab, setTab] = useState("general");
  const [profile, setProfile] = useState<ProfileData>();
  const [name, setName] = useState("");
  const [report, setReport] = useState<Report>();
  const [days, setDays] = useState(30);
  const [reload, setReload] = useState(0);
  const [profileError, setProfileError] = useState("");
  const [usageError, setUsageError] = useState(false);
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState("");
  const [passwordError, setPasswordError] = useState("");
  const [passwordDone, setPasswordDone] = useState(false);
  const [changing, setChanging] = useState(false);
  const format = (value: number) =>
    new Intl.NumberFormat(language).format(value);
  const compact = (value: number) =>
    new Intl.NumberFormat(language, {
      notation: "compact",
      maximumFractionDigits: 1,
    }).format(value);
  const date = (value: string) =>
    new Intl.DateTimeFormat(language, {
      day: "numeric",
      month: "short",
      timeZone: "UTC",
    }).format(new Date(`${value}T12:00:00Z`));
  useEffect(() => {
    const controller = new AbortController();
    void api<ProfileData>("/api/auth/profile", { signal: controller.signal })
      .then((data) => {
        setProfile(data);
        setName(data.display_name);
      })
      .catch((e) => {
        if (!controller.signal.aborted) setProfileError(e.message);
      });
    return () => controller.abort();
  }, []);
  useEffect(() => {
    const controller = new AbortController();
    setReport(undefined);
    setUsageError(false);
    void api<Report>(`/api/account/usage?days=${days}`, {
      signal: controller.signal,
    })
      .then(setReport)
      .catch(() => {
        if (!controller.signal.aborted) setUsageError(true);
      });
    return () => controller.abort();
  }, [days, reload]);
  async function save(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSaving(true);
    setNotice("");
    try {
      const updated = await api<ProfileData>("/api/auth/profile", {
        method: "PUT",
        body: JSON.stringify({ display_name: name.trim() }),
      });
      setProfile(updated);
      setName(updated.display_name);
      setNotice(
        t("Your profile is up to date.", "Tu perfil está actualizado."),
      );
    } catch {
      setNotice(
        t(
          "We couldn't save your changes. Please try again.",
          "No pudimos guardar los cambios. Inténtalo de nuevo.",
        ),
      );
    } finally {
      setSaving(false);
    }
  }
  async function changePassword(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPasswordError("");
    const form = event.currentTarget,
      data = new FormData(form);
    if (data.get("new_password") !== data.get("confirm_password")) {
      setPasswordError(
        t("The passwords don't match.", "Las contraseñas no coinciden."),
      );
      return;
    }
    setChanging(true);
    try {
      await api("/api/auth/password", {
        method: "POST",
        body: JSON.stringify({
          current_password: data.get("current_password"),
          new_password: data.get("new_password"),
        }),
      });
      form.reset();
      setPasswordDone(true);
    } catch (e) {
      const code = e instanceof Error ? e.message : "";
      setPasswordError(
        code === "incorrect_password"
          ? t(
              "Your current password is incorrect.",
              "La contraseña actual es incorrecta.",
            )
          : code === "try_later"
            ? t(
                "Too many attempts. Please wait five minutes.",
                "Demasiados intentos. Espera cinco minutos.",
              )
            : code === "invalid_password"
              ? t(
                  "Use a different password with at least 8 characters.",
                  "Usa una contraseña diferente de al menos 8 caracteres.",
                )
              : t(
                  "We couldn't update your password. Try again.",
                  "No pudimos actualizar la contraseña. Inténtalo de nuevo.",
                ),
      );
    } finally {
      setChanging(false);
    }
  }
  const fullName = profile?.display_name || t("Your space", "Tu espacio");
  const initials = (profile?.display_name || profile?.email || "F")
    .split(/\s+/)
    .slice(0, 2)
    .map((part) => part[0])
    .join("")
    .toUpperCase();
  return (
    <div className="profile-page">
      <header className="profile-heading">
        <div>
          <span className="eyebrow">{t("MADE FOR YOU", "A TU MEDIDA")}</span>
          <h1>{t("My profile", "Mi perfil")}</h1>
          <p>
            {t(
              "A space for your details, your preferences and your activity.",
              "Un espacio para tus datos, tus preferencias y tu actividad.",
            )}
          </p>
        </div>
        <button
          type="button"
          className="subtle profile-signout"
          disabled={signingOut}
          onClick={signOut}
        >
          <Icon name="logout" />
          {signingOut
            ? t("Signing out…", "Cerrando sesión…")
            : t("Sign out", "Cerrar sesión")}
        </button>
      </header>
      {logoutError && <p role="alert">{logoutError}</p>}
      <div
        className="profile-tabs"
        role="tablist"
        aria-label={t("Profile sections", "Secciones del perfil")}
      >
        {[
          ["general", t("Overview", "General"), "user"],
          ["usage", t("Usage", "Consumo"), "activity"],
          ["security", t("Password", "Contraseña"), "settings"],
        ].map(([id, label, icon]) => (
          <button
            key={id}
            id={`profile-tab-${id}`}
            role="tab"
            aria-selected={tab === id}
            aria-controls="profile-panel"
            onClick={() => setTab(id)}
          >
            <Icon name={icon} />
            {label}
          </button>
        ))}
      </div>
      <div
        id="profile-panel"
        role="tabpanel"
        aria-labelledby={`profile-tab-${tab}`}
      >
        {tab === "general" && (
          <section className="profile-identity-grid">
            <div className="profile-persona">
              <div className="profile-avatar" aria-hidden="true">
                {initials}
              </div>
              <h2>{fullName}</h2>
              <p>
                {profile?.email ||
                  t(
                    "Your personal Fénix account",
                    "Tu cuenta personal de Fénix",
                  )}
              </p>
              <span className="profile-persona-note">
                <span />
                {t("Your own space in Fénix", "Tu propio espacio en Fénix")}
              </span>
            </div>
            <form className="profile-card profile-form" onSubmit={save}>
              <div className="profile-section-title">
                <h2>{t("Personal details", "Datos personales")}</h2>
                <p>
                  {t(
                    "Make this space feel a little more like you.",
                    "Haz que este espacio se sienta un poco más tuyo.",
                  )}
                </p>
              </div>
              {profileError ? (
                <p role="alert">
                  {profileError === "local_account"
                    ? t(
                        "This local installation has no managed account.",
                        "Esta instalación local no tiene una cuenta administrada.",
                      )
                    : t(
                        "We couldn't load your profile. Reload the page to try again.",
                        "No pudimos cargar tu perfil. Recarga la página para reintentar.",
                      )}
                </p>
              ) : !profile ? (
                <Loading
                  label={t("Loading your profile", "Cargando tu perfil")}
                />
              ) : (
                <>
                  <div className="profile-fields">
                    <label>
                      {t("Display name", "Nombre visible")}
                      <input
                        name="display_name"
                        autoComplete="nickname"
                        value={name}
                        maxLength={80}
                        placeholder={t(
                          "What should we call you?",
                          "¿Cómo te llamas?",
                        )}
                        onChange={(e) => setName(e.target.value)}
                      />
                    </label>
                    <label>
                      {t("Email address", "Correo electrónico")}
                      <input
                        type="email"
                        value={profile.email}
                        readOnly
                        autoComplete="email"
                      />
                      <small>
                        {t(
                          "This is the email you use to sign in.",
                          "Este es el correo con el que inicias sesión.",
                        )}
                      </small>
                    </label>
                  </div>
                  <div className="profile-form-footer">
                    <span role="status">{notice}</span>
                    <button
                      className="primary"
                      disabled={saving || name.trim() === profile.display_name}
                    >
                      {saving
                        ? t("Saving…", "Guardando…")
                        : t("Save changes", "Guardar cambios")}
                    </button>
                  </div>
                </>
              )}
            </form>
          </section>
        )}
        {tab !== "security" && (
          <section className="profile-usage">
            <div className="profile-section-title profile-usage-title">
              <div>
                <h2>
                  {t(
                    "Your activity, at a glance",
                    "Tu actividad, de un vistazo",
                  )}
                </h2>
                <p>
                  {t(
                    "Tokens recorded while Fénix helps you.",
                    "Tokens registrados mientras Fénix te ayuda.",
                  )}
                </p>
              </div>
              <div className="profile-periods">
                <label className="profile-period-label">
                  {t("Period", "Período")}
                  <select
                    aria-label={t("Usage period", "Período de consumo")}
                    value={days}
                    onChange={(e) => setDays(Number(e.target.value))}
                  >
                    {[7, 30, 90].map((n) => (
                      <option value={n} key={n}>
                        {t(`Last ${n} days`, `Últimos ${n} días`)}
                      </option>
                    ))}
                  </select>
                </label>
                <button
                  type="button"
                  className="subtle icon-button"
                  aria-label={t("Refresh usage", "Actualizar consumo")}
                  onClick={() => setReload((n) => n + 1)}
                >
                  <Icon name="refresh" />
                </button>
              </div>
            </div>
            {usageError ? (
              <div className="profile-card" role="alert">
                <p>
                  {t(
                    "Usage is unavailable right now.",
                    "No pudimos cargar el consumo.",
                  )}
                </p>
                <button onClick={() => setReload((n) => n + 1)}>
                  {t("Try again", "Reintentar")}
                </button>
              </div>
            ) : !report ? (
              <Loading label={t("Loading activity", "Cargando actividad")} />
            ) : (
              <>
                <div className="profile-metrics">
                  {[
                    [
                      t("Total tokens", "Tokens totales"),
                      report.totals.total_tokens,
                      t(
                        "Recorded in this period",
                        "Registrados en este período",
                      ),
                    ],
                    [
                      t("Input", "Entrada"),
                      report.totals.input_tokens,
                      t(
                        "Context read by the models",
                        "Contexto que leen los modelos",
                      ),
                    ],
                    [
                      t("Output", "Salida"),
                      report.totals.output_tokens,
                      t(
                        "Content generated by the models",
                        "Contenido que generan los modelos",
                      ),
                    ],
                  ].map(([label, value, hint], index) => (
                    <div
                      className={`profile-metric${index === 0 ? " is-featured" : ""}`}
                      key={label}
                    >
                      <span>{label}</span>
                      <strong title={format(Number(value))}>
                        {compact(Number(value))}
                      </strong>
                      <small>{hint}</small>
                    </div>
                  ))}
                </div>
                <div className="profile-card profile-chart-card">
                  <div className="profile-chart-heading">
                    <h3>{t("Daily consumption", "Consumo diario")}</h3>
                    <span>
                      {t("Tokens · Lima time", "Tokens · hora de Lima")}
                    </span>
                  </div>
                  {report.totals.requests === 0 ? (
                    <div className="profile-empty">
                      <Icon name="activity" />
                      <h3>
                        {t(
                          "Your activity starts here",
                          "Tu actividad empieza aquí",
                        )}
                      </h3>
                      <p>
                        {t(
                          "When you use Fénix, your recorded usage will appear here.",
                          "Cuando uses Fénix, aquí aparecerá tu consumo registrado.",
                        )}
                      </p>
                    </div>
                  ) : (
                    <UsageChart
                      report={report}
                      format={format}
                      date={date}
                      label={t(
                        "Daily token consumption",
                        "Consumo diario de tokens",
                      )}
                    />
                  )}
                  <div className="profile-chart-footer">
                    <span>
                      {format(report.totals.requests)}{" "}
                      {t("recorded responses", "respuestas registradas")}
                    </span>
                    <span>
                      {date(report.daily[0].date)} —{" "}
                      {date(report.daily.at(-1)!.date)}
                    </span>
                  </div>
                  <details className="profile-data">
                    <summary>
                      {t("View daily detail", "Ver detalle por día")}
                    </summary>
                    <div className="profile-table-scroll">
                      <table>
                        <caption>
                          {t(
                            "Recorded usage by day",
                            "Consumo registrado por día",
                          )}
                        </caption>
                        <thead>
                          <tr>
                            {[
                              t("Date", "Fecha"),
                              t("Input", "Entrada"),
                              t("Output", "Salida"),
                              t("Total", "Total"),
                            ].map((s) => (
                              <th key={s}>{s}</th>
                            ))}
                          </tr>
                        </thead>
                        <tbody>
                          {[...report.daily].reverse().map((row) => (
                            <tr key={row.date}>
                              <td>{date(row.date)}</td>
                              <td>{format(row.usage.input_tokens)}</td>
                              <td>{format(row.usage.output_tokens)}</td>
                              <td>{format(row.usage.total_tokens)}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </details>
                </div>
                {report.totals.missing_usage > 0 && (
                  <p className="profile-note">
                    {t(
                      `${report.totals.missing_usage} responses have incomplete token counts.`,
                      `${report.totals.missing_usage} respuestas tienen un recuento de tokens incompleto.`,
                    )}
                  </p>
                )}
                <p className="profile-note">
                  {t(
                    "Tokens are units of text processed by a model, not a charge. This view includes recorded task, conversation and document-indexing usage. Older voice activity, unrecorded calls and provider failures may be missing; it is not a billing statement.",
                    "Los tokens son unidades de texto que procesa un modelo, no un cobro. Esta vista incluye el consumo registrado de tareas, conversaciones e indexación de documentos. Puede faltar actividad de voz anterior, llamadas no registradas y fallos del proveedor; no es una factura.",
                  )}
                </p>
              </>
            )}
          </section>
        )}
        {tab === "security" && (
          <div className="profile-security-grid">
            <div className="profile-security-intro">
              <span className="profile-security-icon">
                <Icon name="settings" />
              </span>
              <h2>
                {t(
                  "A fresh start for your password.",
                  "Tu acceso, bajo tu control.",
                )}
              </h2>
              <p>
                {t(
                  "Change your password whenever you need to. We'll sign out all sessions so only your new password works.",
                  "Cambia tu contraseña cuando lo necesites. Cerraremos todas las sesiones para que solo puedas entrar con la nueva.",
                )}
              </p>
              <p className="profile-note">
                {t(
                  "Each sign-in lasts exactly seven days. Background tasks keep running when you sign out.",
                  "Cada inicio de sesión dura exactamente siete días. Tus tareas en segundo plano continúan aunque cierres sesión.",
                )}
              </p>
            </div>
            <form
              className="profile-card profile-form"
              onSubmit={changePassword}
            >
              <h2>{t("Change password", "Cambiar contraseña")}</h2>
              {passwordDone ? (
                <div role="status">
                  <p>
                    {t(
                      "Password updated. Sign in again to continue.",
                      "Contraseña actualizada. Inicia sesión de nuevo para continuar.",
                    )}
                  </p>
                  <button
                    type="button"
                    className="primary"
                    onClick={() => location.reload()}
                  >
                    {t("Sign in", "Iniciar sesión")}
                  </button>
                </div>
              ) : (
                <>
                  <label>
                    {t("Current password", "Contraseña actual")}
                    <input
                      required
                      name="current_password"
                      type="password"
                      autoComplete="current-password"
                      maxLength={1024}
                    />
                  </label>
                  <label>
                    {t("New password", "Nueva contraseña")}
                    <input
                      required
                      name="new_password"
                      aria-label={t("New password", "Nueva contraseña")}
                      type="password"
                      autoComplete="new-password"
                      minLength={8}
                      maxLength={1024}
                    />
                    <small>
                      {t(
                        "At least 8 characters. Use a unique password.",
                        "Al menos 8 caracteres. Usa una contraseña única.",
                      )}
                    </small>
                  </label>
                  <label>
                    {t("Confirm new password", "Confirmar nueva contraseña")}
                    <input
                      required
                      name="confirm_password"
                      type="password"
                      autoComplete="new-password"
                      minLength={8}
                      maxLength={1024}
                    />
                  </label>
                  {passwordError && <p role="alert">{passwordError}</p>}
                  <div className="profile-form-footer">
                    <button className="primary" disabled={changing || !profile}>
                      {changing
                        ? t("Updating…", "Actualizando…")
                        : t("Update password", "Actualizar contraseña")}
                    </button>
                  </div>
                </>
              )}
            </form>
          </div>
        )}
      </div>
    </div>
  );
}
function UsageChart({
  report,
  format,
  date,
  label,
}: {
  report: Report;
  format: (n: number) => string;
  date: (d: string) => string;
  label: string;
}) {
  const [selected, setSelected] = useState<number | null>(null);
  const width = 900,
    height = 205,
    left = 12,
    max = Math.max(1, ...report.daily.map((d) => d.usage.total_tokens));
  const step = (width - left * 2) / report.daily.length;
  const active = selected === null ? null : report.daily[selected];
  return (
    <div className="profile-chart">
      <div className="profile-chart-tooltip" aria-live="polite">
        {active
          ? `${date(active.date)} · ${format(active.usage.total_tokens)} tokens`
          : `${format(max)} tokens`}
      </div>
      <svg
        viewBox={`0 0 ${width} ${height + 12}`}
        role="img"
        aria-label={label}
      >
        {[0, 0.5, 1].map((n) => (
          <line
            key={n}
            x1={0}
            x2={width}
            y1={height - n * (height - 12)}
            y2={height - n * (height - 12)}
            className="profile-chart-grid"
          />
        ))}
        {report.daily.map((row, i) => {
          const barHeight = (row.usage.total_tokens / max) * (height - 12);
          return (
            <g key={row.date}>
              <rect
                x={left + i * step}
                y={0}
                width={step}
                height={height}
                fill="transparent"
                onMouseEnter={() => setSelected(i)}
                onMouseLeave={() => setSelected(null)}
                onClick={() => setSelected(i)}
              />
              <rect
                className={`profile-chart-bar${i === selected ? " is-selected" : ""}`}
                x={left + i * step + step * 0.16}
                y={height - Math.max(2, barHeight)}
                width={Math.max(2, step * 0.68)}
                height={Math.max(2, barHeight)}
                rx={Math.min(5, step * 0.2)}
                pointerEvents="none"
              >
                <title>
                  {date(row.date)}: {format(row.usage.total_tokens)} tokens
                </title>
              </rect>
            </g>
          );
        })}
      </svg>
    </div>
  );
}
let root: Root | undefined;
export function stopProfile() {
  root?.unmount();
  root = undefined;
}
export function renderProfile(language: string) {
  stopProfile();
  const target = document.getElementById("page");
  if (target) {
    root = createRoot(target);
    root.render(<Profile language={language} />);
  }
}
