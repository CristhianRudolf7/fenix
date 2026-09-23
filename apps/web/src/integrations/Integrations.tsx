// Pairing stays in the owner UI; no QR or credentials enter the conversation.
import { useEffect, useRef, useState } from "react";
import { createRoot, type Root } from "react-dom/client";
import { Icon } from "../components/icons";
import "./integrations.css";
import { ConnectorCatalog } from "./ConnectorCatalog";
import { ProviderIcon } from "./ProviderIcon";

type Connection = {
  id: string;
  state: string;
  profile?: string;
  agent_access?: boolean;
  session_saved?: boolean;
  qr_available?: boolean;
};
async function request<T>(
  path: string,
  signal: AbortSignal,
  post = false,
): Promise<T> {
  const response = await fetch(`/api/integrations${path}`, {
    method: post ? "POST" : "GET",
    headers: post ? { "content-type": "application/json" } : {},
    cache: "no-store",
    signal,
  });
  const value = await response.json();
  if (!response.ok)
    throw new Error(value?.error || `Request failed (${response.status})`);
  return value as T;
}
function Integrations({ language }: { language: string }) {
  const t = (en: string, es: string) => (language === "es" ? es : en);
  const [connection, setConnection] = useState<Connection>();
  const [qr, setQr] = useState("");
  const [error, setError] = useState("");
  const [pollError, setPollError] = useState("");
  const [busy, setBusy] = useState(false);
  const alive = useRef(true);
  const working = useRef(false);
  const revision = useRef(0);
  const imageUrl = useRef("");
  const poll = useRef<AbortController | undefined>(undefined);
  const command = useRef<AbortController | undefined>(undefined);
  const refresh = useRef<() => Promise<void>>(async () => {});
  function clearQr() {
    if (imageUrl.current) URL.revokeObjectURL(imageUrl.current);
    imageUrl.current = "";
    setQr("");
  }
  refresh.current = async () => {
    if (!alive.current || working.current || poll.current) return;
    const controller = new AbortController(),
      version = revision.current;
    poll.current = controller;
    const current = () =>
      alive.current &&
      version === revision.current &&
      !controller.signal.aborted;
    try {
      const data = await request<{ connections: Connection[] }>(
        "",
        controller.signal,
      );
      if (!current()) return;
      const value = data.connections.find((c) => c.id === "whatsapp");
      setConnection(value);
      setPollError("");
      if (value?.state === "awaiting_qr" && value.qr_available) {
        const response = await fetch("/api/integrations/whatsapp/qr", {
          signal: controller.signal,
          cache: "no-store",
        });
        // A QR may disappear between the status poll and image request after scanning.
        if (!response.ok) {
          if (current()) clearQr();
          return;
        }
        if (!response.headers.get("content-type")?.startsWith("image/png"))
          throw new Error(t("Invalid QR response", "Respuesta QR inválida"));
        const blob = await response.blob();
        if (!current()) return;
        clearQr();
        imageUrl.current = URL.createObjectURL(blob);
        setQr(imageUrl.current);
      } else clearQr();
    } catch (cause) {
      if (current()) {
        clearQr();
        setPollError(cause instanceof Error ? cause.message : String(cause));
      }
    } finally {
      if (poll.current === controller) poll.current = undefined;
    }
  };
  useEffect(() => {
    alive.current = true;
    let timer: ReturnType<typeof setTimeout>;
    const tick = async () => {
      await refresh.current();
      if (alive.current) timer = setTimeout(() => void tick(), 2000);
    };
    void tick();
    return () => {
      alive.current = false;
      revision.current++;
      clearTimeout(timer);
      poll.current?.abort();
      command.current?.abort();
      if (imageUrl.current) URL.revokeObjectURL(imageUrl.current);
    };
  }, []);
  async function act(action: "connect" | "pause" | "forget") {
    if (working.current) return;
    working.current = true;
    setBusy(true);
    revision.current++;
    poll.current?.abort();
    poll.current = undefined;
    clearQr();
    setError("");
    const controller = new AbortController();
    command.current = controller;
    try {
      await request(`/whatsapp/${action}`, controller.signal, true);
    } catch (cause) {
      if (alive.current)
        setError(cause instanceof Error ? cause.message : String(cause));
      return;
    } finally {
      working.current = false;
      if (alive.current) setBusy(false);
    }
    await refresh.current();
  }
  const state = connection?.state || "loading";
  const labels: Record<string, string> = {
    disconnected: t("Not connected", "Sin conectar"),
    loading: t("Opening WhatsApp…", "Abriendo WhatsApp…"),
    awaiting_qr: t("Scan to connect", "Escanea para conectar"),
    syncing: t("Syncing your session…", "Sincronizando tu sesión…"),
    ready: t("Session ready", "Sesión lista"),
    saved: t(
      "Session saved · browser closed",
      "Sesión guardada · navegador cerrado",
    ),
    needs_open: t("Tab closed", "Pestaña cerrada"),
    unexpected_page: t(
      "WhatsApp is not open in this tab",
      "WhatsApp no está abierto en esta pestaña",
    ),
    browser_incompatible: t(
      "WhatsApp requires an updated browser",
      "WhatsApp requiere actualizar el navegador",
    ),
  };
  const canClose =
    !!connection?.profile && state !== "saved" && state !== "disconnected";
  return (
    <section className="integrations-page">
      <header className="integrations-heading">
        <div className="eyebrow">
          {t("Your connected accounts", "Tus cuentas conectadas")}
        </div>
        <h1>{t("Integrations", "Integraciones")}</h1>
        <p className="muted">
          {t(
            "Connect an account so Fénix can use it when you ask.",
            "Conecta una cuenta para que Fénix pueda usarla cuando se lo pidas.",
          )}
        </p>
      </header>
      <ConnectorCatalog language={language} />
      <article
        className="integration-card"
        aria-labelledby="whatsapp-title"
        aria-busy={busy}
      >
        <div className="integration-title">
          <span className="integration-symbol">
            <ProviderIcon provider="whatsapp" />
          </span>
          <div>
            <h2 id="whatsapp-title">WhatsApp</h2>
            <p className="muted">WhatsApp Web</p>
          </div>
          <span
            className={`connection-badge ${state === "ready" ? "is-ready" : ""}`}
            role="status"
          >
            {busy
              ? t("One moment…", "Un momento…")
              : labels[state] || t("Checking session…", "Comprobando sesión…")}
          </span>
        </div>
        <p>
          {t(
            "Scan the QR with your phone. Your login is saved in your Fénix browser, so you can reopen it later.",
            "Escanea el QR con tu teléfono. Tu inicio de sesión se guarda en tu navegador de Fénix para que puedas volver a abrirlo después.",
          )}
        </p>
        {state === "awaiting_qr" && !busy && (
          <div className="integration-pairing">
            <div className="integration-qr">
              {qr ? (
                <img
                  src={qr}
                  width="260"
                  height="260"
                  alt={t("WhatsApp pairing QR", "QR para vincular WhatsApp")}
                />
              ) : (
                <span>{t("Refreshing QR…", "Actualizando QR…")}</span>
              )}
            </div>
            <div>
              <h3>{t("On your phone", "En tu teléfono")}</h3>
              <ol>
                <li>
                  {t(
                    "Open WhatsApp and go to Linked devices.",
                    "Abre WhatsApp y entra en Dispositivos vinculados.",
                  )}
                </li>
                <li>
                  {t("Choose Link a device.", "Elige Vincular un dispositivo.")}
                </li>
                <li>
                  {t(
                    "Scan this QR and wait for the session to finish syncing.",
                    "Escanea este QR y espera a que termine la sincronización.",
                  )}
                </li>
              </ol>
              <p className="muted">
                {t(
                  "The QR refreshes automatically while this page is open.",
                  "El QR se actualiza automáticamente mientras esta página esté abierta.",
                )}
              </p>
            </div>
          </div>
        )}
        {state === "ready" && (
          <p className="integration-ready">
            {connection?.agent_access
              ? t(
                  "Ready for Fénix. Tell the agent what you want to do with WhatsApp.",
                  "Listo para Fénix. Dile al agente qué quieres hacer con WhatsApp.",
                )
              : t(
                  "You have control. Enable agent access to let Fénix use this session.",
                  "Tú tienes el control. Habilita el acceso del agente para que Fénix use esta sesión.",
                )}
          </p>
        )}
        {state === "saved" && (
          <p>
            {t(
              "Your login remains on disk without keeping Chromium running. Fénix can reopen the authorized session when needed.",
              "Tu acceso permanece guardado sin mantener Chromium abierto. Fénix puede volver a abrir la sesión autorizada cuando lo necesite.",
            )}
          </p>
        )}
        {[
          "loading",
          "syncing",
          "unexpected_page",
          "browser_incompatible",
        ].includes(state) &&
          connection && (
            <p className="muted">
              {t(
                "If this takes too long or WhatsApp asks for another step, continue in Browser, then return here to enable agent access.",
                "Si tarda demasiado o WhatsApp pide otro paso, continúa en Navegador y vuelve aquí para habilitar el acceso del agente.",
              )}
            </p>
          )}
        {(error || pollError) && (
          <p className="integration-error" role="alert">
            {error || pollError}
          </p>
        )}
        <div className="integration-actions">
          <button
            type="button"
            className="primary"
            disabled={
              busy ||
              !connection ||
              (state === "ready" && !!connection.agent_access)
            }
            onClick={() => void act("connect")}
          >
            {state === "ready"
              ? t("Enable agent access", "Habilitar acceso del agente")
              : connection?.session_saved
                ? t("Open session", "Abrir sesión")
                : t("Connect WhatsApp", "Conectar WhatsApp")}
          </button>
          {canClose && (
            <button
              type="button"
              disabled={busy}
              onClick={() => void act("pause")}
            >
              {t("Close browser", "Cerrar navegador")}
            </button>
          )}
          {connection?.profile && (
            <button
              type="button"
              disabled={busy}
              onClick={() => void act("forget")}
            >
              {t("Forget session", "Olvidar sesión")}
            </button>
          )}
          {connection?.profile && (
            <a href="/browser">{t("Open Browser", "Abrir Navegador")}</a>
          )}
        </div>
        <p className="integration-footnote muted">
          {t(
            "Connecting enables Fénix to use this session after login. Messages are sent only when you ask. Closing the browser frees RAM and preserves your login; forgetting removes the local session. To unlink this device from your account, use Linked devices on your phone.",
            "Conectar habilita a Fénix para usar esta sesión después de iniciar sesión. Los mensajes se envían solo cuando se lo pidas. Cerrar el navegador libera RAM y conserva tu acceso; olvidar elimina la sesión local. Para desvincular este dispositivo de tu cuenta, usa Dispositivos vinculados en tu teléfono.",
          )}
        </p>
      </article>
    </section>
  );
}
let root: Root | undefined;
export function stopIntegrations() {
  root?.unmount();
  root = undefined;
}
export function renderIntegrations(language: string) {
  stopIntegrations();
  root = createRoot(document.getElementById("page")!);
  root.render(<Integrations language={language} />);
}
