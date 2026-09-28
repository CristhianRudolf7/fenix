import { Loading } from "../components/loading";
import { Icon } from "../components/icons";
// Owner input uses browser endpoints directly, never the agent conversation.
import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { createRoot, type Root } from "react-dom/client";

type Session = {
  profile: string;
  tab: string;
  temporary: boolean;
  width: number;
  height: number;
  dpr: number;
  rss_sum_mib: number | null;
};
type Status = {
  available: boolean;
  profiles: { name: string; allow_agent: boolean }[];
  sessions: Session[];
};
type Tab = { tab: string; title: string; url: string };
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
    throw new Error(value?.error || `Request failed (${response.status})`);
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
  const [accessRequest, setAccessRequest] = useState<boolean>();
  const [url, setUrl] = useState("");
  const [width, setWidth] = useState(1280);
  const [height, setHeight] = useState(800);
  const [files, setFiles] = useState<Download[]>([]);
  const screen = useRef<HTMLImageElement>(null);
  const keyboard = useRef<HTMLTextAreaElement>(null);
  const composing = useRef(false);
  const queue = useRef(Promise.resolve());
  const alive = useRef(true);
  const revision = useRef(0);
  const objectUrl = useRef("");
  const refresh = useRef<() => Promise<void>>(async () => {});
  const session = status?.sessions.find((s) => s.profile === selected);
  const personal = status?.profiles.find((p) => p.name === selected);

  function clearScreen() {
    if (objectUrl.current) URL.revokeObjectURL(objectUrl.current);
    objectUrl.current = "";
    setFrame("");
    if (keyboard.current) keyboard.current.value = "";
    composing.current = false;
  }
  function selectProfile(value: string) {
    revision.current++;
    setAccessRequest(undefined);
    selectedRef.current = value;
    tabRef.current = "";
    setSelected(value);
    setTab("");
    setTabs([]);
    setFiles([]);
    clearScreen();
  }
  function run(action: (profile: string, target: string) => Promise<void>) {
    const profile = selectedRef.current,
      target = tabRef.current,
      version = revision.current;
    queue.current = queue.current
      .catch(() => {})
      .then(async () => {
        if (!alive.current || version !== revision.current) return;
        try {
          setMessage("");
          await action(profile, target);
          if (alive.current) await refresh.current();
        } catch (error) {
          if (alive.current)
            setMessage(error instanceof Error ? error.message : String(error));
        }
      });
  }
  async function command(
    profile: string,
    target: string,
    tool: string,
    args: Record<string, unknown> = {},
  ) {
    if (!profile)
      throw new Error(t("Create a profile first", "Crea un perfil primero"));
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
  useEffect(() => {
    alive.current = true;
    const image = screen.current;
    const preventScroll = (event: WheelEvent) => event.preventDefault();
    image?.addEventListener("wheel", preventScroll, { passive: false });
    const abort = new AbortController();
    let timer: ReturnType<typeof setTimeout>;
    let polling = false;
    async function poll() {
      if (polling || !alive.current) return;
      polling = true;
      const version = revision.current;
      try {
        const next = await api<Status>("status", undefined, abort.signal);
        if (!alive.current || version !== revision.current) return;
        setStatus(next);
        const names = [
          ...next.profiles.map((p) => p.name),
          ...next.sessions.filter((s) => s.temporary).map((s) => s.profile),
        ];
        let profile = selectedRef.current;
        if (!names.includes(profile)) {
          profile = names[0] || "";
          selectedRef.current = profile;
          setSelected(profile);
          tabRef.current = "";
          setTab("");
          clearScreen();
        }
        const active = next.sessions.find((s) => s.profile === profile);
        if (!active) {
          setTabs([]);
          tabRef.current = "";
          setTab("");
          clearScreen();
          return;
        }
        const result = await api<{ tabs: Tab[]; active: string }>(
          "command",
          { tool: "browser_tabs", args: { profile } },
          abort.signal,
        );
        if (!alive.current || version !== revision.current) return;
        // Keep owner actions pinned to the tab displayed by this view.
        const target = result.tabs.some((t) => t.tab === tabRef.current)
          ? tabRef.current
          : result.active;
        if (target !== tabRef.current) {
          clearScreen();
          tabRef.current = target;
          setTab(target);
        }
        setTabs(result.tabs);
        if (!target) return;
        const response = await fetch(
          `/api/browser/frame?profile=${encodeURIComponent(profile)}&tab=${encodeURIComponent(target)}`,
          { signal: abort.signal, cache: "no-store" },
        );
        if (!response.ok)
          throw new Error(
            t(
              "Could not refresh the browser screen",
              "No se pudo actualizar la pantalla del navegador",
            ),
          );
        const blob = await response.blob();
        if (
          !alive.current ||
          version !== revision.current ||
          target !== tabRef.current
        )
          return;
        const previous = objectUrl.current;
        objectUrl.current = URL.createObjectURL(blob);
        setFrame(objectUrl.current);
        if (previous) URL.revokeObjectURL(previous);
      } catch (error) {
        if (!abort.signal.aborted && alive.current)
          setMessage(error instanceof Error ? error.message : String(error));
      } finally {
        polling = false;
      }
    }
    refresh.current = poll;
    async function tick() {
      await poll();
      if (alive.current) timer = setTimeout(tick, 1000);
    }
    void tick();
    return () => {
      image?.removeEventListener("wheel", preventScroll);
      alive.current = false;
      revision.current++;
      abort.abort();
      clearTimeout(timer);
      if (objectUrl.current) URL.revokeObjectURL(objectUrl.current);
      if (keyboard.current) keyboard.current.value = "";
    };
  }, []);

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
      event.key === "Process" ||
      event.key === "Dead" ||
      ["Control", "Alt", "Shift", "Meta", "AltGraph"].includes(event.key)
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
  if (!status && !message) return <Loading />;
  const profiles = [
    ...(status?.profiles.map((p) => p.name) || []),
    ...(status?.sessions.filter((s) => s.temporary).map((s) => s.profile) ||
      []),
  ];
  return (
    <section className="browser-panel">
      <h1>{t("Browser", "Navegador")}</h1>
      <p>
        {t(
          "Sign in here, then enable agent access and return to the conversation. Personal profiles keep logins after closing. Site expiration still applies.",
          "Inicia sesión aquí, habilita el acceso al agente y vuelve a la conversación. Los perfiles personales conservan los accesos al cerrar. Cada sitio puede hacerlos caducar.",
        )}
      </p>
      <form
        className="row"
        onSubmit={(event) => {
          event.preventDefault();
          const form = event.currentTarget;
          const name = String(new FormData(form).get("name"));
          run(async () => {
            await api("profiles", { name, allow_agent: false });
            selectProfile(name);
            form.reset();
          });
        }}
      >
        <label>
          {t("New profile", "Nuevo perfil")}{" "}
          <input
            name="name"
            required
            pattern="[A-Za-z0-9_-]{1,80}"
            maxLength={80}
            placeholder="personal"
          />
        </label>
        <button>{t("Create", "Crear")}</button>
      </form>
      <div className="row">
        <label>
          {t("Profile", "Perfil")}{" "}
          <select
            aria-label={t("Browser profile", "Perfil del navegador")}
            value={selected}
            onChange={(event) => selectProfile(event.target.value)}
          >
            {profiles.map((p) => (
              <option key={p}>{p}</option>
            ))}
          </select>
        </label>
        <label>
          <input
            type="checkbox"
            checked={accessRequest ?? !!personal?.allow_agent}
            disabled={!personal || accessRequest !== undefined}
            onChange={(event) => {
              const allow_agent = event.target.checked;
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
            }}
          />{" "}
          {t("Allow agent access", "Permitir acceso al agente")}
        </label>
        <button
          disabled={!personal}
          onClick={() =>
            run(async (profile) => {
              await api("profiles", { name: profile, allow_agent: false });
              keyboard.current?.focus({ preventScroll: true });
            })
          }
        >
          {t("Take control", "Tomar control")}
        </button>
        <button
          className="subtle icon-button"
          aria-label={t("Close profile", "Cerrar perfil")}
          title={t("Close profile", "Cerrar perfil")}
          disabled={!session}
          onClick={() =>
            run(async (profile) => {
              await command(profile, "", "browser_close");
              revision.current++;
              setAccessRequest(undefined);
              clearScreen();
            })
          }
        >
          <Icon name="close" />
        </button>
      </div>
      <form
        className="row"
        onSubmit={(event) => {
          event.preventDefault();
          const address = url;
          run(async (profile, target) => {
            await command(profile, target, "browser_open", { url: address });
          });
        }}
      >
        <button
          type="button"
          disabled={!session}
          aria-label={t("Back", "Atrás")}
          onClick={() => act({ action: "back" })}
        >
          ←
        </button>
        <button
          type="button"
          disabled={!session}
          aria-label={t("Reload", "Recargar")}
          onClick={() => act({ action: "reload" })}
        >
          ↻
        </button>
        <input
          className="grow"
          type="url"
          required
          aria-label="URL"
          placeholder="https://example.com"
          value={url}
          onChange={(event) => setUrl(event.target.value)}
        />
        <button disabled={!selected}>{t("Open", "Abrir")}</button>
      </form>
      <div className="row">
        <label>
          {t("Tab", "Pestaña")}{" "}
          <select
            aria-label={t("Tab", "Pestaña")}
            value={tab}
            onChange={(event) => {
              const target = event.target.value;
              run(async (profile) => {
                await command(profile, target, "browser_tabs", {
                  action: "select",
                });
                revision.current++;
                setAccessRequest(undefined);
                tabRef.current = target;
                setTab(target);
                clearScreen();
              });
            }}
          >
            {tabs.map((t) => (
              <option key={t.tab} value={t.tab}>
                {t.title || t.url || t.tab}
              </option>
            ))}
          </select>
        </label>
        <button
          disabled={!session}
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
          {t("New tab", "Nueva pestaña")}
        </button>
        <button
          className="subtle icon-button"
          aria-label={t("Close tab", "Cerrar pestaña")}
          title={t("Close tab", "Cerrar pestaña")}
          disabled={!tab}
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
        <select
          aria-label={t("Screen preset", "Tamaño de pantalla")}
          defaultValue="1280,800"
          onChange={(event) => {
            const [w, h] = event.target.value.split(",").map(Number);
            setWidth(w);
            setHeight(h);
          }}
        >
          <option value="1280,800">1280 × 800</option>
          <option value="390,844">390 × 844</option>
          <option value="768,1024">768 × 1024</option>
          <option value="1920,1080">1920 × 1080</option>
        </select>
        <input
          aria-label={t("Width", "Ancho")}
          type="number"
          min={240}
          max={3840}
          value={width}
          onChange={(event) => setWidth(Number(event.target.value))}
        />{" "}
        ×{" "}
        <input
          aria-label={t("Height", "Alto")}
          type="number"
          min={240}
          max={2160}
          value={height}
          onChange={(event) => setHeight(Number(event.target.value))}
        />
        <button
          disabled={!session}
          onClick={() => {
            const size = { width, height };
            run(async (profile, target) => {
              await command(profile, target, "browser_resize", size);
              clearScreen();
            });
          }}
        >
          {t("Resize", "Redimensionar")}
        </button>
      </div>
      {tab && (
        <p className="muted">
          {t("Current page", "Página actual")}:{" "}
          <code>{tabs.find((item) => item.tab === tab)?.url}</code>
        </p>
      )}
      <p role="status">
        {message ||
          (status?.available === false
            ? t("Chromium is unavailable", "Chromium no está disponible")
            : session
              ? `${session.width} × ${session.height}${session.rss_sum_mib === null ? "" : ` · ${session.rss_sum_mib} MiB RSS`}`
              : t("Profile closed", "Perfil cerrado"))}
      </p>
      <div className="row">
        <button
          disabled={!session}
          onClick={() => act({ action: "dialog", accept: true })}
        >
          {t("Accept site dialog", "Aceptar diálogo del sitio")}
        </button>
        <button
          disabled={!session}
          onClick={() => act({ action: "dialog", accept: false })}
        >
          {t("Dismiss site dialog", "Cancelar diálogo del sitio")}
        </button>
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
          {t("Downloads", "Descargas")}
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
      <div className="browser-screen">
        <img
          ref={screen}
          src={frame || undefined}
          hidden={!frame}
          alt={t(
            "Interactive browser screen",
            "Pantalla interactiva del navegador",
          )}
          onClick={(event) => {
            const position = point(event);
            keyboard.current?.focus({ preventScroll: true });
            act({ action: "click", ...position });
          }}
          onWheel={(event) => {
            const scale =
              event.deltaMode === 1
                ? 16
                : event.deltaMode === 2
                  ? session?.height || 800
                  : 1;
            act({
              action: "scroll",
              ...point(event),
              dx: event.deltaX * scale,
              dy: event.deltaY * scale,
            });
          }}
        />
      </div>
      <label>
        {t("Browser keyboard", "Teclado del navegador")}
        <textarea
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
      </label>
      <p className="muted">
        {t(
          "Click the screen to focus an input, then type or paste. Manual actions take control from the agent.",
          "Haz clic en un campo de la pantalla y escribe o pega. Las acciones manuales toman el control del agente.",
        )}
      </p>
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
              (event.nativeEvent as SubmitEvent).submitter as HTMLButtonElement
            )?.value;
            (form.elements.namedItem("master") as HTMLInputElement).value = "";
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
