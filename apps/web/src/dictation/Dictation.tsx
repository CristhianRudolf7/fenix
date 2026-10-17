import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Icon } from "../components/icons";
import { recognitionAvailable, recognizer } from "./speech";

export function Dictation() {
  const [language, setLanguage] = useState(
    document.documentElement.lang || "es",
  );
  const [slot, setSlot] = useState<HTMLElement | null>(null);
  const [dictating, setDictating] = useState(false);
  const [blocked, setBlocked] = useState(true);
  const [error, setError] = useState("");
  const recognition = useRef<ReturnType<typeof recognizer> | undefined>(
    undefined,
  );
  const t = (en: string, es: string) => (language === "es" ? es : en);

  useEffect(() => {
    const stop = () => {
      recognition.current?.dispose();
      recognition.current = undefined;
      setDictating(false);
    };
    const availability = () => {
      const unavailable =
        document.querySelector<HTMLTextAreaElement>("#chat-input")?.readOnly !==
        false;
      setBlocked(unavailable);
      if (unavailable) stop();
    };
    const view = () => {
      stop();
      setError("");
      setSlot(document.getElementById("dictation-controls"));
      setLanguage(document.documentElement.lang || "es");
      availability();
    };
    const submit = (event: Event) => {
      if ((event.target as HTMLElement)?.id === "chat-form") stop();
    };
    const edit = (event: Event) => {
      if (event.isTrusted && (event.target as HTMLElement)?.id === "chat-input")
        stop();
    };
    window.addEventListener("fenix:model-availability", availability);
    window.addEventListener("fenix:view", view);
    window.addEventListener("fenix:stop-dictation", stop);
    window.addEventListener("pagehide", stop);
    document.addEventListener("submit", submit, true);
    document.addEventListener("input", edit);
    view();
    return () => {
      window.removeEventListener("fenix:model-availability", availability);
      window.removeEventListener("fenix:view", view);
      window.removeEventListener("fenix:stop-dictation", stop);
      window.removeEventListener("pagehide", stop);
      document.removeEventListener("submit", submit, true);
      document.removeEventListener("input", edit);
      recognition.current?.dispose();
      recognition.current = undefined;
    };
  }, []);

  const dictate = () => {
    if (dictating) {
      recognition.current?.dispose();
      recognition.current = undefined;
      setDictating(false);
      return;
    }
    const input = document.querySelector<HTMLTextAreaElement>("#chat-input");
    if (!input || input.readOnly) return;
    setError("");
    const prefix = input.value.trimEnd();
    let failed = false;
    const failure = (message: string) => {
      failed = true;
      setError(
        message === "not-allowed" || message === "service-not-allowed"
          ? t(
              "Allow microphone access to dictate. You can still type.",
              "Permite el acceso al micrófono para dictar. Puedes seguir escribiendo.",
            )
          : t(
              "Dictation is unavailable. Please try again or type your message.",
              "No se pudo dictar. Inténtalo de nuevo o escribe tu mensaje.",
            ),
      );
      setDictating(false);
    };
    try {
      recognition.current?.dispose();
      recognition.current = recognizer(
        language === "es" ? "es-PE" : "en-US",
        () => {},
        () => {},
        failure,
        {
          restart: false,
          onStopped: () => setDictating(false),
          onTranscript: (value) => {
            if (!input.isConnected || input.readOnly) return;
            input.value = [prefix, value].filter(Boolean).join(" ");
            input.dispatchEvent(new Event("input", { bubbles: true }));
          },
        },
      );
      recognition.current.start();
      if (!failed) setDictating(true);
    } catch {
      failure("unavailable");
    }
  };

  const supported = recognitionAvailable();
  return (
    <>
      {slot
        ? createPortal(
            <div className="dictation-actions flex items-center gap-2">
              <button
                type="button"
                onClick={dictate}
                disabled={!supported || blocked}
                aria-pressed={dictating}
                aria-label={t("Dictate message", "Dictar mensaje")}
                title={t("Dictate, review, then send", "Dicta, revisa y envía")}
              >
                <Icon name={dictating ? "stop" : "mic"} />
              </button>
            </div>,
            slot,
          )
        : null}
      {!supported && slot
        ? createPortal(
            <small className="muted dictation-availability-note">
              {t(
                "Dictation is unavailable in this browser. You can type your message.",
                "El dictado no está disponible en este navegador. Puedes escribir tu mensaje.",
              )}
            </small>,
            document.getElementById("dictation-availability") || slot,
          )
        : null}
      {error ? (
        <div className="dictation-notice" role="alert">
          {error}
          <button
            type="button"
            onClick={() => setError("")}
            aria-label={t("Close", "Cerrar")}
          >
            ×
          </button>
        </div>
      ) : null}
    </>
  );
}
