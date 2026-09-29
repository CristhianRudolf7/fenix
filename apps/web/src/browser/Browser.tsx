import { serviceError } from "../components/service-error";
import { Loading } from "../components/loading";
import { Icon } from "../components/icons";
// Owner input uses account-scoped browser endpoints, never the agent conversation.
import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { createRoot, type Root } from "react-dom/client";

type Session = {
  profile: string;
  tab: string;
  temporary: boolean;
  suspended?: boolean;
  width: number;
  height: number;
  dpr: number;
};
type Status = {
  available: boolean;
  profiles: { name: string; allow_agent: boolean }[];
  sessions: Session[];
};
type Tab = { tab: string; title: string; url: string };
type View = { profile: string; tabs: Tab[]; active: string };
type Download = { id: string; bytes: number; url: string };
async function api<T>(
  path: string,
  body?: unknown,
  signal?: AbortSignal,
): Promise<T> {
  const response = await fetch(`/api/browser/${path}`, {
    method: body === undefined ? "GET" : "POST",
    headers: body === undefined ? {} : { "content-type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
    signal,
  });
  const value = await response.json();
  if (!response.ok)
    throw new Error(
      serviceError(value?.error || `Request failed (${response.status})`),
    );
  return value as T;
}

function Browser({ language }: { language: string }) {
  const t = (en: string, es: string) => (language === "es" ? es : en);
  const [status, setStatus] = useState<Status>();
  const [selected, setSelected] = useState("");
  const selectedRef = useRef("");
  const [tabs, setTabs] = useState<Tab[]>([]);
  const tabRef = useRef("");
  const [tab, setTab] = useState("");
  const [frame, setFrame] = useState("");
  const [message, setMessage] = useState("");
  const [opening, setOpening] = useState(true);
  const [closed, setClosed] = useState(false);
  const closedRef = useRef(false);
  const needsOpen = useRef(true);
  const [accessRequest, setAccessRequest] = useState<boolean>();
  const [url, setUrl] = useState("");
  const addressFocused = useRef(false);
  const [files, setFiles] = useState<Download[]>([]);
  const screen = useRef<HTMLImageElement>(null);
  const stage = useRef<HTMLDivElement>(null);
  const keyboard = useRef<HTMLTextAreaElement>(null);
  const composing = useRef(false);
  const suppressClick = useRef(false);
  const lastHover = useRef(0);
  const pointer = useRef<{ x: number; y: number; kind: string } | undefined>(
    undefined,
  );
  const queue = useRef(Promise.resolve());
  const busy = useRef(false);
  const alive = useRef(true);
  const revision = useRef(0);
  const mutations = useRef(0);
  const objectUrl = useRef("");
  const lastInteraction = useRef(0);
  const metadataAt = useRef(0);
  const refresh = useRef<() => Promise<void>>(async () => {});
  const session = status?.sessions.find((s) => s.profile === selected);
  const personal = status?.profiles.find((p) => p.name === selected);
  const profiles =
    status?.profiles.filter((p) => !p.name.startsWith("integration-")) || [];

  function clearScreen() {
    if (objectUrl.current) URL.revokeObjectURL(objectUrl.current);
    objectUrl.current = "";
    setFrame("");
    if (keyboard.current) keyboard.current.value = "";
    composing.current = false;
    pointer.current = undefined;
  }
  function openProfile(value = selectedRef.current) {
    revision.current++;
    setAccessRequest(undefined);
    selectedRef.current = value;
    tabRef.current = "";
    setSelected(value);
    setTab("");
    setTabs([]);
    setFiles([]);
    setMessage("");
    setOpening(true);
    setClosed(false);
    closedRef.current = false;
    needsOpen.current = true;
    metadataAt.current = 0;
    clearScreen();
    void refresh.current();
  }
  function run(action: (profile: string, target: string) => Promise<void>) {
    const profile = selectedRef.current,
      target = tabRef.current,
      version = revision.current;
    lastInteraction.current = Date.now();
    queue.current = queue.current
      .catch(() => {})
      .then(async () => {
        if (!alive.current || version !== revision.current) return;
        busy.current = true;
        mutations.current++;
        try {
          setMessage("");
          await action(profile, target);
        } catch {
          if (alive.current)
            setMessage(
              t(
                "That action could not finish. Check the page before trying again.",
                "No se pudo completar esa acción. Revisa la página antes de volver a intentarlo.",
              ),
            );
        } finally {
          busy.current = false;
          metadataAt.current = 0;
          if (alive.current) await refresh.current();
        }
      });
  }
  async function command(
    profile: string,
    target: string,
    tool: string,
    args: Record<string, unknown> = {},
  ) {
    if (!profile) throw new Error("Browser is not ready");
    await api("command", {
      tool,
      args: { profile, ...(target ? { tab: target } : {}), ...args },
    });
  }
  function act(args: Record<string, unknown>) {
    run(async (profile, target) => {
      await command(profile, target, "browser_act", args);
    });
  }
  function handoff(allow_agent: boolean) {
    setAccessRequest(allow_agent);
    run(async (profile) => {
      try {
        await api("profiles", { name: profile, allow_agent });
        setStatus(
          (current) =>
            current && {
              ...current,
              profiles: current.profiles.map((p) =>
                p.name === profile ? { ...p, allow_agent } : p,
              ),
            },
        );
      } finally {
        setAccessRequest(undefined);
      }
    });
  }
  function selectTab(target: string) {
    run(async (profile) => {
      await command(profile, target, "browser_tabs", { action: "select" });
      revision.current++;
      setAccessRequest(undefined);
      tabRef.current = target;
      setTab(target);
      clearScreen();
    });
  }
  useEffect(() => {
    alive.current = true;
    let abort = new AbortController();
    let timer: ReturnType<typeof setTimeout>;
    let polling = false;
    let tickGeneration = 0;
    async function poll() {
      if (
        polling ||
        busy.current ||
        !alive.current ||
        document.hidden ||
        closedRef.current
      )
        return;
      polling = true;
      const version = revision.current;
      const mutation = mutations.current;
      const signal = abort.signal;
      const stale = () =>
        signal.aborted ||
        !alive.current ||
        version !== revision.current ||
        mutation !== mutations.current;
      try {
        if (needsOpen.current) {
          const view = await api<View>(
            "view",
            { profile: selectedRef.current || undefined },
            signal,
          );
          if (stale()) return;
          needsOpen.current = false;
          selectedRef.current = view.profile;
          setSelected(view.profile);
          tabRef.current = view.active;
          setTab(view.active);
          setTabs(view.tabs);
          metadataAt.current = 0;
        }
        const profile = selectedRef.current;
        if (Date.now() - metadataAt.current > 2500) {
          const next = await api<Status>("status", undefined, signal);
          if (stale()) return;
          setStatus(next);
          const active = next.sessions.find((s) => s.profile === profile);
          if (!active || active.suspended) {
            // An idle browser may have been saved to disk. Resume without replaying navigation.
            needsOpen.current = true;
            clearScreen();
            return;
          }
          const result = await api<View>(
            "command",
            { tool: "browser_tabs", args: { profile } },
            signal,
          );
          if (stale()) return;
          const target = result.tabs.some((item) => item.tab === tabRef.current)
            ? tabRef.current
            : result.active;
          if (target !== tabRef.current) {
            clearScreen();
            tabRef.current = target;
            setTab(target);
          }
          setTabs(result.tabs);
          metadataAt.current = Date.now();
          if (!addressFocused.current) {
            const address =
              result.tabs.find((item) => item.tab === target)?.url || "";
            setUrl(address === "about:blank" ? "" : address);
          }
        }
        const target = tabRef.current;
        if (!target) return;
        const response = await fetch(
          `/api/browser/frame?profile=${encodeURIComponent(profile)}&tab=${encodeURIComponent(target)}&format=jpeg`,
          { signal, cache: "no-store" },
        );
        if (!response.ok) throw new Error("Frame unavailable");
        const blob = await response.blob();
        if (stale() || target !== tabRef.current) return;
        const previous = objectUrl.current;
        objectUrl.current = URL.createObjectURL(blob);
        setFrame(objectUrl.current);
        setOpening(false);
        if (previous) URL.revokeObjectURL(previous);
      } catch {
        if (!stale()) {
          setOpening(false);
          setMessage(
            t(
              "The browser could not connect. You can try opening it again.",
              "No se pudo conectar con el navegador. Puedes volver a abrirlo.",
            ),
          );
          // Do not continuously launch/retry a failed browser.
          closedRef.current = true;
          setClosed(true);
          clearScreen();
        }
      } finally {
        polling = false;
      }
    }
    refresh.current = poll;
    async function tick(generation = tickGeneration) {
      await poll();
      if (alive.current && !document.hidden && generation === tickGeneration)
        timer = setTimeout(
          () => tick(generation),
          Date.now() - lastInteraction.current < 4000 ? 200 : 1000,
        );
    }
    function visibility() {
      tickGeneration++;
      clearTimeout(timer);
      abort.abort();
      abort = new AbortController();
      if (!document.hidden) {
        metadataAt.current = 0;
        void tick();
      }
    }
    document.addEventListener("visibilitychange", visibility);
    void tick();
    return () => {
      alive.current = false;
      revision.current++;
      abort.abort();
      clearTimeout(timer);
      document.removeEventListener("visibilitychange", visibility);
      if (objectUrl.current) URL.revokeObjectURL(objectUrl.current);
      if (keyboard.current) keyboard.current.value = "";
    };
  }, []);

  // Only resize when the owner has control. Watching Fenix must not revoke its access.
  useEffect(() => {
    const container = stage.current;
    if (!container || !session || personal?.allow_agent || closed) return;
    let timer: ReturnType<typeof setTimeout>;
    const resize = () => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        if (document.hidden) return;
        if (
          composing.current ||
          pointer.current ||
          Date.now() - lastInteraction.current < 500
        ) {
          resize();
          return;
        }
        const width = Math.max(
          240,
          Math.min(1920, Math.floor(container.clientWidth)),
        );
        const height = Math.max(
          240,
          Math.min(1080, Math.floor(container.clientHeight)),
        );
        if (width === session.width && height === session.height) return;
        run(async (profile, target) => {
          await command(profile, target, "browser_resize", {
            width,
            height,
            dpr: 1,
          });
          clearScreen();
        });
      }, 300);
    };
    const observer = new ResizeObserver(resize);
    observer.observe(container);
    resize();
    return () => {
      observer.disconnect();
      clearTimeout(timer);
    };
  }, [
    selected,
    tab,
    session?.width,
    session?.height,
    personal?.allow_agent,
    closed,
  ]);

  useEffect(() => {
    const image = screen.current;
    let timer: ReturnType<typeof setTimeout> | undefined;
    let pending:
      | { x: number; y: number; dx: number; dy: number; version: number }
      | undefined;
    function scroll(event: WheelEvent) {
      event.preventDefault();
      const scale =
        event.deltaMode === 1
          ? 16
          : event.deltaMode === 2
            ? session?.height || 800
            : 1;
      const previous =
        pending?.version === revision.current ? pending : undefined;
      pending = {
        ...point(event),
        dx: (previous?.dx || 0) + event.deltaX * scale,
        dy: (previous?.dy || 0) + event.deltaY * scale,
        version: revision.current,
      };
      if (timer !== undefined) return;
      timer = setTimeout(() => {
        timer = undefined;
        if (
          pending &&
          pending.version === revision.current &&
          !document.hidden
        ) {
          const { version: _, ...position } = pending;
          act({ action: "scroll", ...position });
        }
        pending = undefined;
      }, 50);
    }
    image?.addEventListener("wheel", scroll, { passive: false });
    return () => {
      image?.removeEventListener("wheel", scroll);
      clearTimeout(timer);
    };
  }, [session?.height, session?.dpr]);

  function point(event: { clientX: number; clientY: number }) {
    const image = screen.current!;
    const rect = image.getBoundingClientRect();
    return {
      x:
        ((event.clientX - rect.left) * image.naturalWidth) /
        rect.width /
        (session?.dpr || 1),
      y:
        ((event.clientY - rect.top) * image.naturalHeight) /
        rect.height /
        (session?.dpr || 1),
    };
  }
  function sendText(input: HTMLTextAreaElement) {
    if (composing.current) return;
    const text = input.value;
    input.value = "";
    if (text) act({ action: "type", text });
  }
  function keyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (
      event.nativeEvent.isComposing ||
      composing.current ||
      [
        "Process",
        "Dead",
        "Control",
        "Alt",
        "Shift",
        "Meta",
        "AltGraph",
      ].includes(event.key)
    )
      return;
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "v")
      return;
    if (
      Array.from(event.key).length === 1 &&
      ((!event.ctrlKey && !event.metaKey && !event.altKey) ||
        event.getModifierState("AltGraph"))
    )
      return;
    event.preventDefault();
    const modifiers = [
      event.ctrlKey && "Control",
      event.altKey && "Alt",
      event.metaKey && "Meta",
      event.shiftKey && "Shift",
    ].filter(Boolean);
    act({ action: "key", key: [...modifiers, event.key].join("+") });
  }
  const allowAgent = accessRequest ?? !!personal?.allow_agent;
  return (
    <section className="browser-panel browser-workspace">
      <header className="browser-heading">
        <div>
          <h1>{t("Your browser", "Tu navegador")}</h1>
          <p>
            {t(
              "Browse and sign in. Your sessions stay here for next time.",
              "Navega e inicia sesión. Tus accesos quedan guardados para la próxima vez.",
            )}
          </p>
        </div>
        <button
          className="subtle browser-handoff"
          disabled={
            !personal || accessRequest !== undefined || opening || closed
          }
          onClick={() => handoff(!allowAgent)}
        >
          <Icon name={allowAgent ? "user" : "wave"} />
          {allowAgent
            ? t("Take control", "Tomar control")
            : t("Let Fenix continue", "Dejar continuar a Fénix")}
        </button>
      </header>
      <div className="browser-window">
        <div className="browser-tabs-bar">
          <div
            role="tablist"
            aria-label={t("Browser tabs", "Pestañas del navegador")}
          >
            {tabs.map((item) => (
              <button
                key={item.tab}
                role="tab"
                aria-selected={item.tab === tab}
                onClick={() => selectTab(item.tab)}
                title={item.title || item.url}
              >
                {item.title ||
                  (item.url === "about:blank"
                    ? t("New tab", "Nueva pestaña")
                    : item.url)}
              </button>
            ))}
          </div>
          <button
            className="subtle icon-button"
            aria-label={t("New tab", "Nueva pestaña")}
            disabled={!session || closed}
            onClick={() =>
              run(async (profile) => {
                await command(profile, "", "browser_tabs", { action: "new" });
                revision.current++;
                setAccessRequest(undefined);
                tabRef.current = "";
                clearScreen();
              })
            }
          >
            +
          </button>
          <button
            className="subtle icon-button"
            aria-label={t("Close tab", "Cerrar pestaña")}
            disabled={!tab || closed}
            onClick={() =>
              run(async (profile, target) => {
                await command(profile, target, "browser_close");
                revision.current++;
                setAccessRequest(undefined);
                tabRef.current = "";
                clearScreen();
              })
            }
          >
            <Icon name="close" />
          </button>
        </div>
        <form
          className="browser-navigation"
          onSubmit={(event) => {
            event.preventDefault();
            const value = url.trim();
            if (!value) return;
            const address = /^[a-z][a-z\d+.-]*:/i.test(value)
              ? value
              : `https://${value}`;
            addressFocused.current = false;
            run(async (profile, target) => {
              await command(profile, target, "browser_open", { url: address });
            });
          }}
        >
          <button
            type="button"
            className="subtle icon-button"
            aria-label={t("Back", "Atrás")}
            disabled={!tab || closed}
            onClick={() => act({ action: "back" })}
          >
            <span className="browser-back">
              <Icon name="arrow" />
            </span>
          </button>
          <button
            type="button"
            className="subtle icon-button"
            aria-label={t("Forward", "Adelante")}
            disabled={!tab || closed}
            onClick={() => act({ action: "forward" })}
          >
            <Icon name="arrow" />
          </button>
          <button
            type="button"
            className="subtle icon-button"
            aria-label={t("Reload", "Recargar")}
            disabled={!tab || closed}
            onClick={() => act({ action: "reload" })}
          >
            <Icon name="refresh" />
          </button>
          <input
            aria-label={t("Website address", "Dirección web")}
            placeholder={t("Enter a website", "Escribe una dirección web")}
            value={url}
            disabled={opening || closed}
            autoComplete="off"
            spellCheck={false}
            onFocus={() => {
              addressFocused.current = true;
            }}
            onBlur={() => {
              addressFocused.current = false;
            }}
            onChange={(event) => setUrl(event.target.value)}
          />
          <button
            className="subtle icon-button"
            aria-label={t("Go", "Ir")}
            disabled={opening || closed}
          >
            <Icon name="arrow" />
          </button>
        </form>
        <div ref={stage} className="browser-screen">
          {!frame && (
            <div className="browser-placeholder">
              {opening ? (
                <>
                  <Loading />
                  <p>{t("Opening your browser…", "Abriendo tu navegador…")}</p>
                </>
              ) : (
                <>
                  <Icon name="browser" />
                  <p>
                    {message ||
                      t(
                        "Your logins are saved. Open the browser to continue.",
                        "Tus accesos están guardados. Abre el navegador para continuar.",
                      )}
                  </p>
                  <button onClick={() => openProfile()}>
                    {t("Open browser", "Abrir navegador")}
                  </button>
                </>
              )}
            </div>
          )}
          <img
            ref={screen}
            src={frame || undefined}
            hidden={!frame}
            draggable={false}
            alt={t(
              "Interactive browser screen",
              "Pantalla interactiva del navegador",
            )}
            onPointerDown={(event) => {
              if (event.button !== 0) return;
              suppressClick.current = false;
              pointer.current = { ...point(event), kind: event.pointerType };
              event.currentTarget.setPointerCapture(event.pointerId);
            }}
            onPointerCancel={() => {
              pointer.current = undefined;
              suppressClick.current = true;
            }}
            onPointerUp={(event) => {
              const start = pointer.current;
              pointer.current = undefined;
              if (!start || event.button !== 0) return;
              const end = point(event);
              if (Math.hypot(end.x - start.x, end.y - start.y) > 8) {
                suppressClick.current = true;
                if (start.kind === "touch")
                  act({
                    action: "scroll",
                    x: start.x,
                    y: start.y,
                    dx: start.x - end.x,
                    dy: start.y - end.y,
                  });
                else
                  act({
                    action: "drag",
                    x: start.x,
                    y: start.y,
                    to_x: end.x,
                    to_y: end.y,
                  });
              }
            }}
            onClick={(event) => {
              if (suppressClick.current) {
                suppressClick.current = false;
                return;
              }
              keyboard.current?.focus({ preventScroll: true });
              act({
                action: "click",
                ...point(event),
                click_count: Math.max(1, Math.min(3, event.detail)),
              });
            }}
            onPointerMove={(event) => {
              if (
                event.pointerType !== "mouse" ||
                event.buttons ||
                allowAgent ||
                busy.current ||
                Date.now() - lastHover.current < 300
              )
                return;
              lastHover.current = Date.now();
              act({ action: "hover", ...point(event) });
            }}
            onContextMenu={(event) => {
              event.preventDefault();
              act({ action: "right_click", ...point(event) });
            }}
          />
          <textarea
            className="browser-keyboard"
            aria-label={t("Browser keyboard", "Teclado del navegador")}
            ref={keyboard}
            rows={1}
            disabled={!frame}
            autoComplete="off"
            autoCorrect="off"
            autoCapitalize="off"
            spellCheck={false}
            onKeyDown={keyDown}
            onCompositionStart={() => {
              composing.current = true;
            }}
            onCompositionEnd={(event) => {
              composing.current = false;
              sendText(event.currentTarget);
            }}
            onInput={(event) => {
              if (!(event.nativeEvent as InputEvent).isComposing)
                sendText(event.currentTarget);
            }}
          />
        </div>
        <footer className="browser-footer">
          <span role="status">
            {message ||
              (opening
                ? t("Connecting…", "Conectando…")
                : closed
                  ? t("Browser closed", "Navegador cerrado")
                  : allowAgent
                    ? t(
                        "Fenix can use this browser",
                        "Fénix puede usar este navegador",
                      )
                    : t("You are in control", "Tú tienes el control"))}
          </span>
          <button
            className="subtle"
            disabled={!frame}
            onClick={() => keyboard.current?.focus({ preventScroll: true })}
          >
            {t("Keyboard", "Teclado")}
          </button>
        </footer>
      </div>
      <details className="browser-options">
        <summary>
          {t("Saved sessions and options", "Sesiones guardadas y opciones")}
        </summary>
        <div className="row">
          {profiles.length > 1 && (
            <label>
              {t("Saved session", "Sesión guardada")}
              <select
                aria-label={t("Saved session", "Sesión guardada")}
                value={selected}
                onChange={(event) => openProfile(event.target.value)}
              >
                {profiles.map((p) => (
                  <option key={p.name}>{p.name}</option>
                ))}
              </select>
            </label>
          )}
          <button
            disabled={!selected}
            onClick={() =>
              run(async (profile) => {
                const result = await api<{ files: Download[] }>(
                  `downloads?profile=${encodeURIComponent(profile)}`,
                );
                if (selectedRef.current === profile) setFiles(result.files);
              })
            }
          >
            <Icon name="download" />
            {t("Downloads", "Descargas")}
          </button>
          <button
            disabled={!session || closed}
            onClick={() => act({ action: "dialog", accept: true })}
          >
            {t("Accept site dialog", "Aceptar diálogo del sitio")}
          </button>
          <button
            disabled={!session || closed}
            onClick={() => act({ action: "dialog", accept: false })}
          >
            {t("Dismiss site dialog", "Cancelar diálogo del sitio")}
          </button>
          <button
            disabled={!session || closed}
            onClick={() =>
              run(async (profile) => {
                await command(profile, "", "browser_close");
                revision.current++;
                setAccessRequest(undefined);
                closedRef.current = true;
                setClosed(true);
                setTabs([]);
                setTab("");
                tabRef.current = "";
                clearScreen();
              })
            }
          >
            {t("Close browser", "Cerrar navegador")}
          </button>
        </div>
        {files.map((file) => (
          <p key={file.id}>
            <a href={file.url} download>
              {file.id}
            </a>{" "}
            ({file.bytes} bytes)
          </p>
        ))}
        <details>
          <summary>{t("Saved passwords", "Contraseñas guardadas")}</summary>
          <p>
            {t(
              "Encrypted locally. Filling checks the exact site and does not submit the form.",
              "Cifradas localmente. Completar verifica el sitio exacto y no envía el formulario.",
            )}
          </p>
          <form
            autoComplete="off"
            onSubmit={(event) => {
              event.preventDefault();
              const form = event.currentTarget;
              const values = Object.fromEntries(new FormData(form));
              const action = (
                (event.nativeEvent as SubmitEvent)
                  .submitter as HTMLButtonElement
              )?.value;
              (form.elements.namedItem("master") as HTMLInputElement).value =
                "";
              (form.elements.namedItem("password") as HTMLInputElement).value =
                "";
              run(async (profile, target) => {
                try {
                  await api("credentials", {
                    ...values,
                    action,
                    profile,
                    tab: target,
                  });
                } finally {
                  values.master = "";
                  values.password = "";
                }
              });
            }}
          >
            <label>
              {t("Master password", "Clave maestra")}{" "}
              <input
                name="master"
                type="password"
                minLength={12}
                required
                autoComplete="off"
              />
            </label>
            <label>
              {t("Username", "Usuario")}{" "}
              <input name="username" autoComplete="off" />
            </label>
            <label>
              {t("Site password", "Contraseña del sitio")}{" "}
              <input name="password" type="password" autoComplete="off" />
            </label>
            <div className="row">
              <button value="save">
                {t("Save for this site", "Guardar para este sitio")}
              </button>
              <button value="fill">
                {t("Fill saved login", "Completar acceso guardado")}
              </button>
              <button value="delete">
                {t("Delete saved login", "Eliminar acceso guardado")}
              </button>
            </div>
          </form>
        </details>{" "}
      </details>
    </section>
  );
}
let root: Root | undefined;
export function stopBrowser() {
  root?.unmount();
  root = undefined;
}
export function renderBrowser(language: string) {
  stopBrowser();
  root = createRoot(document.getElementById("page")!);
  root.render(<Browser language={language} />);
}
