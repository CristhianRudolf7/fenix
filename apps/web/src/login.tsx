import { useState, type FormEvent } from "react";
import { createRoot } from "react-dom/client";
import { Flame, Icon } from "./components/icons";
import wing from "./assets/phoenix-wing.webp";
import { navigateTo } from "./components/routing";

function Login() {
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [english, setEnglish] = useState(
    localStorage.getItem("fenix.language") === "en",
  );
  const [dark, setDark] = useState(
    localStorage.getItem("fenix.theme") === "dark",
  );
  const [visible, setVisible] = useState(false);
  const t = (en: string, es: string) => (english ? en : es);
  function changeLanguage() {
    const language = english ? "es" : "en";
    setEnglish(!english);
    localStorage.setItem("fenix.language", language);
    document.documentElement.lang = language;
  }
  function changeTheme() {
    const theme = dark ? "light" : "dark";
    setDark(!dark);
    localStorage.setItem("fenix.theme", theme);
    document.documentElement.dataset.theme = theme;
  }
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const fields = new FormData(event.currentTarget);
    setBusy(true);
    setError("");
    try {
      const response = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          email: fields.get("email"),
          password: fields.get("password"),
        }),
      });
      if (!response.ok)
        throw new Error(
          response.status === 429
            ? english
              ? "Too many attempts. Try again in a minute."
              : "Demasiados intentos. Vuelve a intentar en un minuto."
            : response.status === 503
              ? t(
                  "Access is not configured or is temporarily unavailable.",
                  "El acceso aún no está configurado o no está disponible temporalmente.",
                )
              : english
                ? "Invalid email or password."
                : "Correo o contraseña incorrectos.",
        );
      location.reload();
    } catch (error) {
      setError(error instanceof Error ? error.message : "Connection error");
    } finally {
      setBusy(false);
    }
  }
  return (
    <main className="login-page">
      <section
        className="login-story"
        aria-label={t("Welcome to Fénix", "Bienvenido a Fénix")}
      >
        <img className="login-art" src={wing} alt="" fetchPriority="high" />
        <div className="login-brand">
          <Flame />
          <h1>Fénix</h1>
        </div>
        <div className="login-story-copy">
          <h2>
            {t("A space to think.", "Un espacio para pensar.")}
            <br />
            {t("An assistant to move forward.", "Un asistente para avanzar.")}
          </h2>
          <p>
            {t(
              "Your projects, your ideas and your everyday life. All in one conversation.",
              "Tus proyectos, tus ideas y tu día a día, en una sola conversación.",
            )}
          </p>
        </div>
        <p className="login-signature">
          {t("At your pace. In your own way.", "A tu ritmo. A tu manera.")}
        </p>
      </section>
      <section className="login-entry">
        <div className="login-preferences">
          <button
            type="button"
            className="subtle"
            onClick={changeLanguage}
            aria-label={t("Change language", "Cambiar idioma")}
          >
            {english ? "ES" : "EN"}
          </button>
          <button
            type="button"
            className="subtle icon-button"
            onClick={changeTheme}
            aria-label={t(
              "Toggle light and dark mode",
              "Cambiar modo claro y oscuro",
            )}
          >
            <Icon name={dark ? "sun" : "moon"} />
          </button>
        </div>
        <form className="login-card" onSubmit={submit} aria-busy={busy}>
          <div className="login-heading">
            <div className="eyebrow">
              {t("YOUR PERSONAL SPACE", "TU ESPACIO PERSONAL")}
            </div>
            <h2>{t("Good to see you.", "Qué bueno verte.")}</h2>
            <p>
              {t(
                "Sign in to your personal assistant.",
                "Entra a tu asistente personal.",
              )}
            </p>
          </div>
          <label>
            {english ? "Email" : "Correo"}
            <input
              name="email"
              type="email"
              autoComplete="username"
              placeholder={t("you@email.com", "tu@correo.com")}
              required
              autoCapitalize="none"
              spellCheck={false}
            />
          </label>
          <div className="password-field">
            <label htmlFor="login-password">
              {english ? "Password" : "Contraseña"}
            </label>
            <div className="password-input">
              <input
                id="login-password"
                name="password"
                type={visible ? "text" : "password"}
                autoComplete="current-password"
                required
              />
              <button
                type="button"
                className="subtle icon-button"
                onClick={() => setVisible(!visible)}
                aria-label={
                  visible
                    ? t("Hide password", "Ocultar contraseña")
                    : t("Show password", "Mostrar contraseña")
                }
                aria-pressed={visible}
              >
                <Icon name={visible ? "hide" : "eye"} />
              </button>
            </div>
          </div>
          {error && (
            <p className="login-error" role="alert">
              {error}
            </p>
          )}
          <button
            className="primary login-submit"
            type="submit"
            disabled={busy}
          >
            {busy
              ? english
                ? "Signing in…"
                : "Ingresando…"
              : english
                ? "Sign in"
                : "Ingresar"}
            <Icon name="arrow" />
          </button>
          <p className="login-footnote">
            {t(
              "One conversation. All your context.",
              "Una conversación. Todo tu contexto.",
            )}
          </p>
        </form>
      </section>
    </main>
  );
}

export async function authenticate(): Promise<boolean> {
  const response = await fetch("/api/auth/session");
  // Standalone local/evaluation runtimes intentionally retain their existing UI.
  if (response.status === 404) {
    document.documentElement.dataset.installation = "personal";
    return true;
  }
  if (response.status === 401) {
    createRoot(document.getElementById("app")!).render(<Login />);
    return false;
  }
  if (!response.ok) throw new Error("Authentication service unavailable");
  const session = await response.json();
  if (session.installation === "personal") {
    document.documentElement.dataset.installation = "personal";
  }
  if (typeof session.expires_at_ms === "number") {
    const remaining =
      typeof session.remaining_ms === "number"
        ? session.remaining_ms
        : session.expires_at_ms - Date.now();
    if (remaining <= 0) {
      location.reload();
      return false;
    }
    window.setTimeout(() => location.reload(), remaining);
  }
  const control = document.createElement("div");
  control.className = "account-menu";
  document.body.append(control);
  createRoot(control).render(<AccountMenu email={session.email || ""} />);
  return true;
}

/** Move the persistent account control before replacing any React/DOM shell. */
export function placeAccountControl(detach = false) {
  const button = document.querySelector(".account-menu");
  if (button)
    (detach
      ? document.body
      : document.getElementById("account-controls") || document.body
    ).append(button);
}

function AccountMenu({ email }: { email: string }) {
  const t = (en: string, es: string) =>
    document.documentElement.lang === "en" ? en : es;
  return (
    <button
      type="button"
      className="account-card"
      aria-label={t("View my profile", "Ver mi perfil")}
      title={`${t("View my profile", "Ver mi perfil")} · ${email}`}
      onClick={() => navigateTo("profile")}
    >
      <Icon name="user" />
    </button>
  );
}
