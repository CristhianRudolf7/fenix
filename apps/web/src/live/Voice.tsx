import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { api, type LiveStatus } from "./api";
import { LiveCall, type CallState } from "./call";
import { recognitionAvailable, recognizer } from "./speech";
import { Icon } from "../components/icons";
const initial: CallState = {
  active: false,
  starting: false,
  phase: "idle",
  transcript: "",
  reply: "",
  error: "",
  muted: false,
};
export function Voice() {
  const [language, setLanguage] = useState(
    document.documentElement.lang || "es",
  );
  const [slot, setSlot] = useState<HTMLElement | null>(null);
  const [state, setState] = useState(initial);
  const call = useRef<LiveCall | null>(null);
  if (!call.current) call.current = new LiveCall(setState);
  const [config, setConfig] = useState<LiveStatus | null>(null),
    [settingsOpen, setSettingsOpen] = useState(false),
    [providers, setProviders] = useState<{ id: string }[]>([]);
  const [saving, setSaving] = useState(false),
    [error, setError] = useState(""),
    [dictating, setDictating] = useState(false),
    [text, setText] = useState("");
  const dictation = useRef<ReturnType<typeof recognizer> | undefined>(
    undefined,
  );
  const t = (en: string, es: string) => (language === "es" ? es : en);
  useEffect(() => {
    const view = () => {
      setSlot(document.getElementById("voice-controls"));
      setLanguage(document.documentElement.lang || "es");
      dictation.current?.dispose();
      dictation.current = undefined;
      setDictating(false);
    };
    window.addEventListener("fenix:view", view);
    view();
    const close = () => {
      call.current?.close();
      dictation.current?.dispose();
    };
    window.addEventListener("pagehide", close);
    return () => {
      window.removeEventListener("fenix:view", view);
      window.removeEventListener("pagehide", close);
      close();
    };
  }, []);
  const loadConfig = async () => {
    try {
      const [value, list] = await Promise.all([
        api<LiveStatus>("/live/settings"),
        api<{ id: string }[]>("/providers"),
      ]);
      setConfig(value);
      setProviders(list);
      return value;
    } catch (e) {
      setError(String(e));
      return null;
    }
  };
  const start = async () => {
    dictation.current?.dispose();
    setDictating(false);
    const config = await loadConfig();
    if (!config) return;
    if (!config.configured) {
      setSettingsOpen(true);
      return;
    }
    await call.current!.start(language);
  };
  const dictate = () => {
    if (dictating) {
      dictation.current?.dispose();
      setDictating(false);
      return;
    }
    try {
      dictation.current = recognizer(
        language === "es" ? "es-PE" : "en-US",
        (value) => {
          const input =
            document.querySelector<HTMLTextAreaElement>("#chat-input");
          if (input) {
            input.value = (input.value + " " + value).trim();
            input.dispatchEvent(new Event("input", { bubbles: true }));
          }
        },
        () => {},
        (e) => {
          setError(e);
          setDictating(false);
        },
      );
      dictation.current.start();
      setDictating(true);
    } catch (e) {
      setError(String(e));
    }
  };
  const supported = recognitionAvailable() && "speechSynthesis" in window;
  const notice = error || (!state.active && !state.starting ? state.error : "");
  const phases: Record<string, string> = {
    connecting: t("Connecting", "Conectando"),
    listening: t("Listening", "Escuchando"),
    thinking: t("Responding", "Preparando respuesta"),
    speaking: t("Speaking", "Hablando"),
    "microphone-error": t("Microphone unavailable", "Micrófono no disponible"),
    error: t("Connection problem", "Problema de conexión"),
  };
  return (
    <>
      {slot &&
        createPortal(
          <div className="voice-actions flex items-center gap-2">
            <button
              type="button"
              onClick={dictate}
              disabled={
                !recognitionAvailable() || state.active || state.starting
              }
              aria-pressed={dictating}
              aria-label={t("Dictate message", "Dictar mensaje")}
              title={t("Dictate, review, then send", "Dicta, revisa y envía")}
            >
              <Icon name={dictating ? "stop" : "mic"} />
            </button>
            <button
              type="button"
              className="primary live-button"
              disabled={!supported}
              onClick={() =>
                state.active || state.starting
                  ? call.current!.close()
                  : void start()
              }
              aria-label={t("Start or end Live", "Iniciar o cerrar Live")}
            >
              <Icon name="wave" />
              {state.active || state.starting
                ? t("End Live", "Cerrar Live")
                : "Live"}
            </button>
            <button
              type="button"
              className="subtle voice-settings-button"
              aria-label={t(
                "Conversation settings",
                "Configuración de conversación",
              )}
              onClick={() => {
                void loadConfig();
                setSettingsOpen(true);
              }}
            >
              <Icon name="settings" />
            </button>
          </div>,
          slot,
        )}
      {!supported &&
        slot &&
        createPortal(
          <small className="muted voice-availability-note">
            {t(
              "Browser voice is unavailable. Text chat still works.",
              "La voz no está disponible en este navegador. Puedes usar el chat escrito.",
            )}
          </small>,
          document.getElementById("voice-availability") || slot,
        )}
      {notice && (
        <div className="voice-notice" role="alert">
          {notice}
          <button
            onClick={() => {
              setError("");
              call.current!.dismissError();
            }}
            aria-label={t("Close", "Cerrar")}
          >
            ×
          </button>
        </div>
      )}
      {(state.active || state.starting) && (
        <section
          className="voice-panel fixed bottom-5 right-5 z-50 rounded-2xl border p-5 shadow-xl"
          aria-label={t("Live call", "Llamada Live")}
        >
          <header className="flex items-center justify-between gap-4">
            <h2>Fénix Live</h2>
            <button type="button" onClick={() => call.current!.close()}>
              {t("Hang up", "Colgar")}
            </button>
          </header>
          <p role="status">
            <span className="dot" />
            {phases[state.phase] || state.phase}
          </p>
          <p className="muted">
            {t(
              "Tasks continue when you hang up.",
              "Las tareas continúan al colgar.",
            )}
          </p>
          <div className="voice-caption" aria-live="polite">
            {state.reply}
          </div>
          <p className="voice-transcript">{state.transcript}</p>
          {state.error && <p role="alert">{state.error}</p>}
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => call.current!.toggleMute()}
              aria-pressed={state.muted}
            >
              {state.muted
                ? t("Enable microphone", "Activar micrófono")
                : t("Mute", "Silenciar")}
            </button>
            <button type="button" onClick={() => call.current!.interrupt()}>
              {t("Interrupt speech", "Interrumpir voz")}
            </button>
          </div>
          <form
            className="mt-3 flex gap-2"
            onSubmit={(e) => {
              e.preventDefault();
              e.stopPropagation();
              call.current!.send(text);
              setText("");
            }}
          >
            <input
              aria-label={t("Message during Live", "Mensaje durante Live")}
              value={text}
              onChange={(e) => setText(e.target.value)}
              placeholder={t(
                "You can also type here",
                "También puedes escribir aquí",
              )}
            />
            <button type="submit" disabled={!text.trim() || !state.active}>
              {t("Send", "Enviar")}
            </button>
          </form>
        </section>
      )}
      {settingsOpen && (
        <div className="voice-backdrop">
          <section
            role="dialog"
            aria-modal="true"
            aria-labelledby="voice-settings-title"
            className="voice-settings rounded-2xl p-6"
          >
            <div className="flex justify-between">
              <h2 id="voice-settings-title">
                {t("Conversation settings", "Configuración de conversación")}
              </h2>
              <button
                type="button"
                onClick={() => setSettingsOpen(false)}
                aria-label={t(
                  "Close conversation settings",
                  "Cerrar configuración de conversación",
                )}
              >
                ×
              </button>
            </div>
            <p>
              {t(
                "This conversational model handles web chat, Telegram and Live while the master executes tasks independently. Browser speech requires microphone permission; recognition may use browser vendor servers.",
                "Este modelo atiende el chat web, Telegram y Live mientras el maestro ejecuta tareas de forma independiente. La voz requiere permiso del micrófono; el reconocimiento puede usar servidores del navegador.",
              )}
            </p>
            {config && (
              <form
                onSubmit={async (e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  setSaving(true);
                  setError("");
                  try {
                    const form = e.currentTarget;
                    const data = new FormData(form);
                    await api("/live/settings", {
                      provider: data.get("provider"),
                      model: data.get("model"),
                      base_url: data.get("base_url") || null,
                      api_key: data.get("api_key") || null,
                    });
                    form.reset();
                    await loadConfig();
                    setSettingsOpen(false);
                  } catch (e) {
                    setError(String(e));
                  } finally {
                    setSaving(false);
                  }
                }}
              >
                <label>
                  {t("Provider", "Proveedor")}
                  <select
                    name="provider"
                    defaultValue={config.settings.provider}
                  >
                    {providers.map((p) => (
                      <option key={p.id}>{p.id}</option>
                    ))}
                  </select>
                </label>
                <label>
                  {t("Model", "Modelo")}
                  <input
                    name="model"
                    required
                    defaultValue={config.settings.model}
                  />
                </label>
                <label>
                  API URL
                  <input
                    name="base_url"
                    type="url"
                    defaultValue={config.settings.base_url || ""}
                  />
                </label>
                <label>
                  API key
                  <input
                    name="api_key"
                    type="password"
                    autoComplete="off"
                    required={!config.configured}
                  />
                </label>
                <button
                  className="primary mt-4"
                  type="submit"
                  disabled={saving}
                >
                  {saving
                    ? t("Verifying…", "Verificando…")
                    : t("Verify and save", "Verificar y guardar")}
                </button>
              </form>
            )}
          </section>
        </div>
      )}
    </>
  );
}
