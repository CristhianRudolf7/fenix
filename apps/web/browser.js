// Local browser controls: credentials never pass through conversation endpoints.
let dispose = () => {};
export function stopBrowser() {
  dispose();
  dispose = () => {};
}
export async function renderBrowser({ api, t, escape, toast }) {
  stopBrowser();
  let actionQueue = Promise.resolve();
  let lastStatus = 0;
  let stopped = false,
    timer,
    objectUrl,
    busy = false,
    selected = "",
    snapshot;
  const abort = new AbortController();
  dispose = () => {
    stopped = true;
    clearTimeout(timer);
    abort.abort();
    if (objectUrl) URL.revokeObjectURL(objectUrl);
  };
  const root = document.querySelector("#page");
  root.innerHTML = `<section class="browser-panel"><h1>${t("Browser", "Navegador")}</h1>
    <p>${t("Personal profiles keep your logins after closing. Use these controls for passwords, verification and QR codes. Enable agent access when you are ready.", "Los perfiles personales conservan tus accesos al cerrar. Usa estos controles para contraseñas, verificación y códigos QR. Habilita al agente cuando estés listo.")}</p>
    <form id="browser-create" class="row"><label>${t("New profile", "Nuevo perfil")} <input name="name" required pattern="[A-Za-z0-9_-]{1,80}" placeholder="personal" maxlength="80"></label><button>${t("Create", "Crear")}</button></form>
    <div class="row"><label>${t("Profile", "Perfil")} <select id="browser-profile" aria-label="${t("Browser profile", "Perfil del navegador")}"></select></label><label><input id="browser-agent" type="checkbox"> ${t("Allow agent access", "Permitir acceso al agente")}</label><button id="browser-control">${t("Take control", "Tomar control")}</button><button id="browser-close">${t("Close profile", "Cerrar perfil")}</button></div>
    <form id="browser-open" class="row"><button type="button" id="browser-back" aria-label="${t("Back", "Atrás")}">←</button><button type="button" id="browser-reload" aria-label="${t("Reload", "Recargar")}">↻</button><input name="url" class="grow" type="url" placeholder="https://example.com" required aria-label="URL"><button>${t("Open", "Abrir")}</button></form>
    <div class="row"><label>${t("Tab", "Pestaña")} <select id="browser-tabs"></select></label><button id="browser-new-tab">+</button><button id="browser-close-tab">${t("Close tab", "Cerrar pestaña")}</button><select id="browser-preset" aria-label="${t("Screen preset", "Tamaño de pantalla")}"><option value="1280,800">${t("Desktop", "Escritorio")} 1280 × 800</option><option value="390,844">${t("Phone", "Teléfono")} 390 × 844</option><option value="768,1024">Tablet 768 × 1024</option><option value="1920,1080">1920 × 1080</option></select><input id="browser-width" aria-label="${t("Width", "Ancho")}" type="number" min="240" max="3840" value="1280"><span>×</span><input id="browser-height" aria-label="${t("Height", "Alto")}" type="number" min="240" max="2160" value="800"><button id="browser-resize">${t("Resize", "Redimensionar")}</button></div>
    <p id="browser-status" role="status"></p><div class="row"><button id="browser-dialog-accept">${t("Accept site dialog", "Aceptar diálogo del sitio")}</button><button id="browser-dialog-dismiss">${t("Dismiss site dialog", "Cancelar diálogo del sitio")}</button><button id="browser-downloads">${t("Downloads", "Descargas")}</button></div><div id="browser-download-list"></div><div class="browser-screen"><img id="browser-screen" tabindex="0" hidden alt="${t("Interactive browser screen", "Pantalla interactiva del navegador")}"></div>
    <details><summary>${t("Saved passwords", "Contraseñas guardadas")}</summary><p>${t("Encrypted locally with your master password (at least 12 characters). Fill checks the exact site and profile. It does not submit the form.", "Cifradas localmente con tu clave maestra (mínimo 12 caracteres). Completar verifica el sitio y perfil exactos. No envía el formulario.")}</p><form id="browser-vault" autocomplete="off"><label>${t("Master password", "Clave maestra")} <input name="master" type="password" minlength="12" required autocomplete="off"></label><label>${t("Username", "Usuario")} <input name="username" autocomplete="off"></label><label>${t("Site password", "Contraseña del sitio")} <input name="password" type="password" autocomplete="off"></label><div class="row"><button name="action" value="save">${t("Save for this site", "Guardar para este sitio")}</button><button name="action" value="fill">${t("Fill saved login", "Completar acceso guardado")}</button><button name="action" value="delete">${t("Delete saved login", "Eliminar acceso guardado")}</button></div></form></details>
    <p>${t("Closing frees browser memory. Site session expiry still applies. Screen emulation does not replace testing on physical devices.", "Cerrar libera la memoria del navegador. Los accesos pueden caducar según cada sitio. La emulación de pantalla no sustituye las pruebas en dispositivos reales.")}</p></section>`;
  const $ = (id) => root.querySelector(id);
  const post = (path, body) => api(path, { method: "POST", body });
  const run = (fn) => async (event) => {
    event?.preventDefault();
    try {
      await fn(event);
    } catch (error) {
      if (!stopped) toast(error.message);
    }
  };
  const command = async (tool, args = {}) => {
    if (!selected)
      throw Error(
        t(
          "Create or select a profile first",
          "Crea o selecciona un perfil primero",
        ),
      );
    return post("/browser/command", {
      tool,
      args: { ...args, profile: selected },
    });
  };
  async function refresh() {
    snapshot = await api("/browser/status");
    if (stopped) return;
    const personal = snapshot.profiles;
    const profiles = [
      ...personal.map((p) => p.name),
      ...snapshot.sessions.filter((s) => s.temporary).map((s) => s.profile),
    ];
    if (!profiles.includes(selected)) selected = profiles[0] || "";
    $("#browser-profile").innerHTML = profiles
      .map(
        (name) =>
          `<option ${name === selected ? "selected" : ""} value="${escape(name)}">${escape(name)}</option>`,
      )
      .join("");
    const profile = personal.find((p) => p.name === selected);
    $("#browser-agent").checked = !!profile?.allow_agent;
    $("#browser-agent").disabled = !profile;
    const session = snapshot.sessions.find((s) => s.profile === selected);
    $("#browser-status").textContent = !snapshot.available
      ? t(
          "Chromium is unavailable. Configure its executable in browser.json.",
          "Chromium no está disponible. Configura su ejecutable en browser.json.",
        )
      : session
        ? `${session.rss_sum_mib} MiB RSS · ${session.width} × ${session.height} · ${t("Memory budget", "Presupuesto de memoria")}: ${snapshot.config.memory_budget_mib} MiB`
        : t(
            "Profile closed. Its saved logins remain on disk.",
            "Perfil cerrado. Sus accesos guardados permanecen en disco.",
          );
    $("#browser-screen").hidden = !session;
    if (session) {
      const tabs = await command("browser_tabs");
      if (stopped) return;
      $("#browser-tabs").innerHTML = tabs.tabs
        .map(
          (tab) =>
            `<option value="${escape(tab.tab)}" ${tab.tab === tabs.active ? "selected" : ""}>${escape(tab.title || tab.url || tab.tab)}</option>`,
        )
        .join("");
    } else $("#browser-tabs").innerHTML = "";
  }
  async function manual() {
    if ($("#browser-agent").checked) {
      await post("/browser/profiles", { name: selected, allow_agent: false });
      $("#browser-agent").checked = false;
    }
  }
  function act(args) {
    const profile = selected;
    actionQueue = actionQueue
      .catch(() => {})
      .then(async () => {
        if (stopped || selected !== profile) return;
        await manual();
        return command("browser_act", args);
      });
    return actionQueue;
  }
  async function frames() {
    if (stopped) return;
    try {
      if (Date.now() - lastStatus > 5000) {
        await refresh();
        lastStatus = Date.now();
      }
      const session = snapshot?.sessions.find((s) => s.profile === selected);
      if (session && !busy) {
        busy = true;
        const requestedProfile = selected;
        const response = await fetch(
          `/api/browser/frame?profile=${encodeURIComponent(selected)}`,
          { signal: abort.signal, cache: "no-store" },
        );
        if (!response.ok) {
          await refresh();
        } else {
          const blob = await response.blob();
          if (!stopped && requestedProfile === selected) {
            const previous = objectUrl;
            objectUrl = URL.createObjectURL(blob);
            $("#browser-screen").src = objectUrl;
            if (previous) URL.revokeObjectURL(previous);
          }
        }
      }
    } catch (error) {
      if (error.name !== "AbortError" && !stopped)
        $("#browser-status").textContent = error.message;
    } finally {
      busy = false;
      if (!stopped) timer = setTimeout(frames, 750);
    }
  }
  $("#browser-create").onsubmit = run(async (event) => {
    selected = new FormData(event.target).get("name");
    await post("/browser/profiles", { name: selected, allow_agent: false });
    event.target.reset();
    await refresh();
  });
  $("#browser-profile").onchange = run(async (event) => {
    selected = event.target.value;
    await refresh();
  });
  $("#browser-agent").onchange = run(async (event) => {
    await post("/browser/profiles", {
      name: selected,
      allow_agent: event.target.checked,
    });
    await refresh();
  });
  $("#browser-control").onclick = run(async () => {
    await manual();
    await refresh();
    $("#browser-screen").focus();
  });
  $("#browser-open").onsubmit = run(async (event) => {
    await manual();
    await command("browser_open", {
      url: new FormData(event.target).get("url"),
    });
    await refresh();
  });
  $("#browser-close").onclick = run(async () => {
    await command("browser_close");
    await refresh();
  });
  $("#browser-back").onclick = run(() => act({ action: "back" }));
  $("#browser-reload").onclick = run(() => act({ action: "reload" }));
  $("#browser-tabs").onchange = run(async (event) => {
    await manual();
    await command("browser_tabs", {
      action: "select",
      tab: event.target.value,
    });
    await refresh();
  });
  $("#browser-new-tab").onclick = run(async () => {
    await manual();
    await command("browser_tabs", { action: "new" });
    await refresh();
  });
  $("#browser-close-tab").onclick = run(async () => {
    await manual();
    await command("browser_close", { tab: $("#browser-tabs").value });
    await refresh();
  });
  $("#browser-preset").onchange = (event) => {
    const [width, height] = event.target.value.split(",");
    $("#browser-width").value = width;
    $("#browser-height").value = height;
  };
  $("#browser-resize").onclick = run(async () => {
    await manual();
    await command("browser_resize", {
      width: Number($("#browser-width").value),
      height: Number($("#browser-height").value),
    });
    await refresh();
  });
  $("#browser-dialog-accept").onclick = run(() =>
    act({ action: "dialog", accept: true }),
  );
  $("#browser-dialog-dismiss").onclick = run(() =>
    act({ action: "dialog", accept: false }),
  );
  $("#browser-downloads").onclick = run(async () => {
    const result = await api(
      `/browser/downloads?profile=${encodeURIComponent(selected)}`,
    );
    $("#browser-download-list").innerHTML =
      result.files
        .map(
          (f) =>
            `<p><a href="${escape(f.url)}" download>${escape(f.id)}</a> (${f.bytes} bytes)</p>`,
        )
        .join("") ||
      t("No completed downloads", "No hay descargas completadas");
  });
  const screen = $("#browser-screen");
  function point(event) {
    const rect = screen.getBoundingClientRect();
    const session = snapshot.sessions.find((s) => s.profile === selected);
    return {
      x:
        ((event.clientX - rect.left) * screen.naturalWidth) /
        rect.width /
        (session?.dpr || 1),
      y:
        ((event.clientY - rect.top) * screen.naturalHeight) /
        rect.height /
        (session?.dpr || 1),
    };
  }
  screen.onclick = run(async (event) => {
    screen.focus();
    await act({ action: "click", ...point(event) });
  });
  screen.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      run(() =>
        act({
          action: "scroll",
          ...point(event),
          dx: event.deltaX,
          dy: event.deltaY,
        }),
      )(event);
    },
    { passive: false },
  );
  screen.onkeydown = (event) => {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "v")
      return;
    if (
      event.isComposing ||
      ["Control", "Shift", "Alt", "Meta"].includes(event.key)
    )
      return;
    event.preventDefault();
    const modifiers = [
      event.ctrlKey && "Control",
      event.altKey && "Alt",
      event.metaKey && "Meta",
      event.shiftKey && "Shift",
    ].filter(Boolean);
    run(() =>
      act(
        Array.from(event.key).length === 1 &&
          !event.ctrlKey &&
          !event.altKey &&
          !event.metaKey
          ? { action: "type", text: event.key }
          : { action: "key", key: [...modifiers, event.key].join("+") },
      ),
    )(event);
  };
  screen.onpaste = run((event) =>
    act({ action: "type", text: event.clipboardData.getData("text/plain") }),
  );
  $("#browser-vault").onsubmit = run(async (event) => {
    await manual();
    const values = Object.fromEntries(new FormData(event.target));
    try {
      await post("/browser/credentials", {
        ...values,
        action: event.submitter.value,
        profile: selected,
      });
      toast(t("Completed", "Completado"));
    } finally {
      event.target.elements.master.value = "";
      event.target.elements.password.value = "";
      values.master = "";
      values.password = "";
    }
  });
  await refresh();
  frames();
}
