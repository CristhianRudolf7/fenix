import { useEffect, useState, type FormEvent } from "react";
import { createRoot } from "react-dom/client";
import { Flame, Icon } from "./components/icons";
import wing from "./assets/phoenix-wing.webp";
import { navigateTo } from "./components/routing";

function Login() {
  const [register, setRegister] = useState(false);
  const [googleEnabled, setGoogleEnabled] = useState<boolean | null>(null);
  const [googleError, setGoogleError] = useState("");
  useEffect(() => {
    let active = true;
    fetch("/api/auth/google")
      .then((r) => (r.ok ? r.json() : { enabled: false }))
      .then((value) => {
        if (active) setGoogleEnabled(value.enabled === true);
      })
      .catch(() => {
        if (active) setGoogleEnabled(false);
      });
    const query = new URLSearchParams(location.search);
    const code = query.get("auth_error");
    if (code) {
      setGoogleError(code);
      if (code === "google_registration_required") setRegister(true);
      query.delete("auth_error");
      history.replaceState(
        null,
        "",
        location.pathname + (query.size ? "?" + query : ""),
      );
    }
    return () => {
      active = false;
    };
  }, []);
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
  const googleMessages: Record<string, [string, string]> = {
    google_unavailable: [
      "Google sign-in is not available yet. You can still sign in with your password.",
      "El acceso con Google todavía no está disponible. Puedes seguir entrando con tu contraseña.",
    ],
    google_expired: [
      "This sign-in expired. Please try again.",
      "Este acceso caducó. Vuelve a intentarlo.",
    ],
    google_cancelled: [
      "You cancelled Google sign-in. You can try again whenever you like.",
      "Cancelaste el acceso con Google. Puedes volver a intentarlo cuando quieras.",
    ],
    google_registration_required: [
      "You don't have a Fénix account yet. Choose Create account to get started.",
      "Todavía no tienes una cuenta de Fénix. Elige Crear cuenta para empezar.",
    ],
    google_capacity_unavailable: [
      "We cannot open a new space right now. Please try again later.",
      "Ahora mismo no podemos abrir un nuevo espacio. Vuelve a intentarlo más tarde.",
    ],
    google_link_required: [
      "This email already has an account. Sign in with your password to link Google from your profile.",
      "Este correo ya tiene una cuenta. Entra con tu contraseña para vincular Google desde tu perfil.",
    ],
    google_account_conflict: [
      "Choose the Google account that belongs to this Fénix account.",
      "Elige la cuenta de Google que corresponde a esta cuenta de Fénix.",
    ],
    google_account_unavailable: [
      "This account is unavailable. Please contact support.",
      "Esta cuenta no está disponible. Contacta con soporte.",
    ],
    google_failed: [
      "We couldn't complete Google sign-in. Please try again.",
      "No pudimos completar el acceso con Google. Vuelve a intentarlo.",
    ],
  };
  async function googleSignIn() {
    setBusy(true);
    setError("");
    setGoogleError("");
    try {
      const response = await fetch("/api/auth/google", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ register }),
      });
      const value = await response.json();
      if (!response.ok) {
        setGoogleError(value.error || "google_failed");
        setBusy(false);
        return;
      }
      const destination = new URL(value.url);
      if (destination.origin !== "https://accounts.google.com")
        throw Error("Invalid authorization URL");
      location.assign(destination.href);
    } catch {
      setGoogleError("google_failed");
      setBusy(false);
    }
  }
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (register) {
      if (googleEnabled) void googleSignIn();
      return;
    }
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
              ? "Too many attempts. Try again in five minutes."
              : "Demasiados intentos. Vuelve a intentar en cinco minutos."
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
            <h2>
              {register
                ? t("Make yourself at home.", "Este es tu espacio.")
                : t("Good to see you.", "Qué bueno verte.")}
            </h2>
            <p>
              {register
                ? t(
                    "Create your account with Google and start your conversation.",
                    "Crea tu cuenta con Google y empieza tu conversación.",
                  )
                : t(
                    "Sign in and pick up where you left off.",
                    "Entra y continúa donde lo dejaste.",
                  )}
            </p>
          </div>
          <div className="login-google-section">
            <button
              type="button"
              className="login-google"
              onClick={googleSignIn}
              disabled={busy || googleEnabled !== true}
            >
              <img
                src="/assets/integrations/google.svg"
                width="20"
                height="20"
                alt=""
              />
              {busy
                ? t("Connecting…", "Conectando…")
                : register
                  ? t("Sign up with Google", "Registrarse con Google")
                  : t("Sign in with Google", "Iniciar sesión con Google")}
            </button>
            {googleEnabled === false && (
              <p className="login-provider-note">
                {t(
                  "Google access will be available soon.",
                  "El acceso con Google estará disponible pronto.",
                )}
              </p>
            )}
            {googleError && (
              <p className="login-error" role="alert">
                {t(
                  ...(googleMessages[googleError] ||
                    googleMessages.google_failed)!,
                )}
              </p>
            )}
          </div>
          {!register && (
            <>
              <div className="login-divider">
                <span>{t("or use your email", "o usa tu correo")}</span>
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
            </>
          )}
          <button
            type="button"
            className="subtle login-mode"
            disabled={busy}
            onClick={() => {
              setRegister(!register);
              setError("");
              setGoogleError("");
            }}
          >
            {register
              ? t(
                  "Already have an account? Sign in",
                  "¿Ya tienes una cuenta? Inicia sesión",
                )
              : t(
                  "New to Fénix? Create account",
                  "¿Nuevo en Fénix? Crear cuenta",
                )}
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
