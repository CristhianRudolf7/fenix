import { Icon } from "../components/icons";
import { Markdown } from "../components/markdown";
import { useEffect, useState } from "react";
import { createRoot, type Root } from "react-dom/client";
import { Loading } from "../components/loading";
import "./skills.css";

type Skill = {
  name: string;
  description: string;
  version: string;
  category: string;
  enabled: boolean;
};
type Bundle = {
  name: string;
  files: Record<string, string>;
  executable: string[];
};
type Publication = {
  id: string;
  name: string;
  category: string;
  status: string;
  public: boolean;
  repaired: boolean;
  source_hash: string;
  reviewed_hash?: string;
};
type Report = {
  reason?: string;
  repair?: {
    explanation: string;
    edits: { file: string; before: string; after: string }[];
  };
  [key: string]: unknown;
};
async function api<T>(
  path: string,
  body?: unknown,
  method = "POST",
): Promise<T> {
  const response = await fetch(`/api${path}`, {
    method: body === undefined ? "GET" : method,
    headers: body === undefined ? {} : { "content-type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const value = await response.json().catch(() => null);
  if (!response.ok)
    throw new Error(value?.error || `Request failed (${response.status})`);
  return value as T;
}
type Category = {
  id: string;
  labels: Record<string, string>;
  aliases: string[];
};
type SkillCard = { name: string; title: string; description: string };
const markdownBody = (source: string) =>
  source.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n/, "").trim();
async function publishedSpanish(
  bundle: Bundle,
): Promise<{ title: string; description: string; content: string } | null> {
  const document = bundle.files["SKILL.es.md"];
  if (!document) return null;
  try {
    const header = document.match(/^---\n([\s\S]*?)\n---/);
    if (!header) return null;
    const metadata = JSON.parse(header[1]);
    const digest = await crypto.subtle.digest(
      "SHA-256",
      new TextEncoder().encode(bundle.files["SKILL.md"]),
    );
    const hash = Array.from(new Uint8Array(digest))
      .map((v) => v.toString(16).padStart(2, "0"))
      .join("");
    if (
      metadata.source_hash !== `v1-es-${hash}` ||
      typeof metadata.title !== "string" ||
      typeof metadata.description !== "string"
    )
      return null;
    return {
      title: metadata.title,
      description: metadata.description,
      content: document,
    };
  } catch {
    return null;
  }
}
const readableName = (name: string) => name.replaceAll("-", " ");
const statusEs: Record<string, string> = {
  queued: "En cola",
  reviewing: "Revisando",
  approved: "Revisado",
  needs_review: "Necesita revisión",
  failed: "Revisión incompleta",
  cancelled: "Cancelado",
};
function Skills({ language }: { language: string }) {
  const t = (en: string, es: string) => (language === "es" ? es : en);
  const [skills, setSkills] = useState<Skill[]>([]);
  const [mine, setMine] = useState<Publication[]>([]);
  const [items, setItems] = useState<Publication[]>([]);
  const [tab, setTab] = useState("mine");
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("");
  const [categories, setCategories] = useState<Category[]>([]);
  const [localized, setLocalized] = useState<Record<string, SkillCard>>({});
  const [translating, setTranslating] = useState(false);
  const [translationError, setTranslationError] = useState("");
  const [translationRetry, setTranslationRetry] = useState(0);
  const categoryLabel = (id: string) =>
    categories.find((c) => c.id === id)?.labels[language] || id;
  const [offset, setOffset] = useState(0);
  const [busy, setBusy] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [configured, setConfigured] = useState(false);
  const [dialog, setDialog] = useState<{
    title: string;
    text?: string;
    report?: Report | null;
  } | null>(null);
  const [installation, setInstallation] = useState<Publication | null>(null);
  const [installName, setInstallName] = useState("");
  useEffect(() => {
    if (!dialog && !installation) return;
    const previous = document.activeElement as HTMLElement;
    const modal = document.querySelector<HTMLElement>(".skill-modal");
    modal?.querySelector<HTMLElement>("button, input")?.focus();
    const key = (event: KeyboardEvent) => {
      if (event.key === "Escape" && (dialog || !busy)) {
        setDialog(null);
        setInstallation(null);
      }
      if (event.key === "Tab") {
        const items = Array.from(
          modal?.querySelectorAll<HTMLElement>(
            'button:not(:disabled), input:not(:disabled), a[href], [tabindex="0"]',
          ) || [],
        ).filter((el) => el.getClientRects().length);
        if (event.shiftKey && document.activeElement === items[0]) {
          event.preventDefault();
          items.at(-1)?.focus();
        } else if (!event.shiftKey && document.activeElement === items.at(-1)) {
          event.preventDefault();
          items[0]?.focus();
        }
      }
    };
    document.addEventListener("keydown", key);
    return () => {
      document.removeEventListener("keydown", key);
      if (previous?.isConnected) previous.focus();
    };
  }, [dialog, installation]);
  async function publications() {
    const all: Publication[] = [];
    for (let page = 0; page < 20; page++) {
      const result = await api<{ items: Publication[] }>(
        `/skill-exchange/mine?offset=${page * 50}`,
      );
      all.push(...result.items);
      if (result.items.length < 50) break;
    }
    setMine(all);
  }
  useEffect(() => {
    let active = true;
    void Promise.all([
      api<Skill[]>("/skills"),
      api<Category[]>("/skill-categories"),
      api<{ configured: boolean }>("/skill-exchange/status"),
    ])
      .then(([list, taxonomy, status]) => {
        if (active) {
          setSkills(list);
          setCategories(taxonomy);
          setConfigured(status.configured);
        }
      })
      .catch((e: Error) => {
        if (active) setError(e.message);
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    void publications().catch((e: Error) => {
      if (active) setError(e.message);
    });
    return () => {
      active = false;
    };
  }, []);
  useEffect(() => {
    let active = true;
    setLocalized({});
    setTranslationError("");
    if (language !== "es" || tab === "reviews") {
      setTranslating(false);
      return;
    }
    const source = tab === "market" ? items : skills;
    if (!source.length) {
      setTranslating(false);
      return;
    }
    setTranslating(true);
    void (async () => {
      try {
        const cards: SkillCard[] = [];
        if (tab === "market") {
          for (const item of items) {
            const bundle = await api<Bundle>(
              `/skill-exchange/publications/${item.id}/bundle`,
            );
            const display = await publishedSpanish(bundle);
            if (display)
              cards.push({
                name: item.name,
                title: display.title,
                description: display.description,
              });
          }
        } else {
          for (let offset = 0; offset < source.length; offset += 256) {
            const result = await api<{ cards: SkillCard[] }>(
              "/skill-localizations",
              {
                language,
                names: source.slice(offset, offset + 256).map((s) => s.name),
              },
            );
            cards.push(...result.cards);
          }
        }
        if (active)
          setLocalized(Object.fromEntries(cards.map((c) => [c.name, c])));
      } catch {
        if (active)
          setTranslationError(
            "No se pudieron cargar las versiones en español.",
          );
      } finally {
        if (active) setTranslating(false);
      }
    })();
    return () => {
      active = false;
    };
  }, [language, tab, skills, items, translationRetry]);
  useEffect(() => {
    if (
      language !== "es" ||
      tab !== "mine" ||
      translating ||
      !skills.length ||
      skills.every((s) => localized[s.name])
    )
      return;
    const timer = window.setInterval(
      () => setTranslationRetry((v) => v + 1),
      15000,
    );
    return () => window.clearInterval(timer);
  }, [language, tab, translating, skills, localized]);
  const pending = mine.some((p) => ["queued", "reviewing"].includes(p.status));
  useEffect(() => {
    if (!pending) return;
    const timer = window.setInterval(() => {
      void publications().catch((e: Error) => setError(e.message));
    }, 5000);
    return () => window.clearInterval(timer);
  }, [pending]);
  useEffect(() => {
    if (tab !== "market") return;
    let active = true;
    const timer = window.setTimeout(() => {
      setLoading(true);
      void api<{ items: Publication[] }>(
        `/skill-exchange/catalog?q=${encodeURIComponent(query)}&category=${encodeURIComponent(category)}&offset=${offset}`,
      )
        .then((v) => {
          if (active) setItems(v.items);
        })
        .catch((e: Error) => {
          if (active) setError(e.message);
        })
        .finally(() => {
          if (active) setLoading(false);
        });
    }, 250);
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [tab, query, category, offset]);
  async function run(key: string, work: () => Promise<void>) {
    if (busy) return;
    setBusy(key);
    setError("");
    setNotice("");
    try {
      await work();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(null);
    }
  }
  async function publish(skill: Skill) {
    const bundle = await api<Bundle>(
      `/skills/${encodeURIComponent(skill.name)}/bundle`,
    );
    const publication = await api<Publication>("/skill-exchange/publications", {
      bundle,
    });
    // A withdrawn approved version can be made public without spending more tokens.
    if (publication.status === "approved" && !publication.public)
      await api(`/skill-exchange/publications/${publication.id}/visibility`, {
        public: true,
      });
    if (publication.status === "cancelled") {
      await api(`/skill-exchange/publications/${publication.id}/retry`, {});
    }
    await publications();
    setNotice(
      t(
        "Publication requested. Your private original stays unchanged.",
        "Publicación solicitada. Tu original privado se conserva intacto.",
      ),
    );
  }
  const label = (p: Publication) =>
    p.public
      ? t("Public", "Público")
      : language === "es"
        ? statusEs[p.status] || p.status
        : p.status.replaceAll("_", " ");
  const filtered = skills.filter(
    (s) =>
      (!category || s.category === category) &&
      `${s.name} ${s.description} ${localized[s.name]?.title || ""} ${localized[s.name]?.description || ""}`
        .toLowerCase()
        .includes(query.toLowerCase()),
  );
  return (
    <div className="content skill-exchange">
      <header className="skills-hero">
        <h1>Skills</h1>
      </header>
      <div className="row skills-tabs" role="tablist" aria-label="Skills">
        <button
          role="tab"
          aria-selected={tab === "mine"}
          onClick={() => {
            setTab("mine");
            setQuery("");
            setLoading(false);
          }}
        >
          {t("My skills", "Mis skills")} <small>{skills.length}</small>
        </button>
        <button
          role="tab"
          aria-selected={tab === "market"}
          onClick={() => {
            setTab("market");
            setQuery("");
            setOffset(0);
          }}
        >
          {t("Marketplace", "Marketplace")}
        </button>
        <button
          role="tab"
          aria-selected={tab === "reviews"}
          onClick={() => {
            setTab("reviews");
            setLoading(false);
          }}
        >
          {t("Publications", "Publicaciones")}
          {pending ? " · ●" : ""}
        </button>
      </div>
      {error && (
        <p role="alert" className="form-status error">
          {error}
        </p>
      )}
      {notice && (
        <p role="status" className="form-status">
          {notice}
        </p>
      )}
      {tab === "mine" && !configured && (
        <p className="note">
          {t(
            "Publication review is not available yet. Your private skills remain available.",
            "La revisión de publicaciones aún no está disponible. Tus skills privados siguen disponibles.",
          )}
        </p>
      )}
      {tab !== "reviews" && (
        <div className="skills-filters">
          <input
            aria-label={t("Search skills", "Buscar skills")}
            placeholder={t("Find a skill…", "Encuentra un skill…")}
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setOffset(0);
            }}
          />
          <select
            aria-label={t("Category", "Categoría")}
            value={category}
            onChange={(e) => {
              setCategory(e.target.value);
              setOffset(0);
            }}
          >
            <option value="">
              {t("All categories", "Todas las categorías")}
            </option>
            {categories.map((c) => (
              <option key={c.id} value={c.id}>
                {c.labels[language] || c.labels.en}
              </option>
            ))}
          </select>
        </div>
      )}
      {translating && (
        <p className="muted" role="status">
          {t("Loading saved documents…", "Cargando documentos guardados…")}
        </p>
      )}
      {translationError && (
        <p role="status">
          {translationError}{" "}
          <button
            className="subtle"
            onClick={() => setTranslationRetry((v) => v + 1)}
          >
            {t("Retry", "Reintentar")}
          </button>
        </p>
      )}
      {loading || translating ? (
        <Loading
          label={t("Finding possibilities…", "Descubriendo posibilidades…")}
        />
      ) : tab === "mine" ? (
        <div className="grid">
          {filtered.map((skill) => {
            const versions = mine.filter((p) => p.name === skill.name);
            const published = versions.find((p) => p.public);
            const waiting = versions.find((p) =>
              ["queued", "reviewing"].includes(p.status),
            );
            return (
              <article key={skill.name} className="card skill-card">
                <div className="skill-title">
                  <h2>
                    {localized[skill.name]?.title ||
                      (language === "es"
                        ? "Versión en español pendiente"
                        : readableName(skill.name))}
                  </h2>
                  <span className="badge">v{skill.version}</span>
                </div>
                <span className="eyebrow">{categoryLabel(skill.category)}</span>
                <p className="muted">
                  {localized[skill.name]?.description ||
                    (language === "es"
                      ? "La versión en español se guarda automáticamente al crear o actualizar este skill."
                      : skill.description)}
                </p>
                <div className="row spread">
                  <label className="toggle">
                    <input
                      type="checkbox"
                      checked={skill.enabled}
                      disabled={!!busy}
                      onChange={(e) => {
                        const enabled = e.target.checked;
                        void run(skill.name, async () => {
                          await api(
                            `/skills/${encodeURIComponent(skill.name)}`,
                            { enabled },
                            "PUT",
                          );
                          setSkills((list) =>
                            list.map((s) =>
                              s.name === skill.name ? { ...s, enabled } : s,
                            ),
                          );
                        });
                      }}
                    />
                    {t("Enabled", "Activo")}
                  </label>
                  <span className="badge">
                    {published
                      ? t("Public version", "Versión pública")
                      : t("Private", "Privado")}
                  </span>
                </div>
                <div className="actions">
                  <button
                    disabled={
                      !!busy || (language === "es" && !localized[skill.name])
                    }
                    onClick={() =>
                      void run(skill.name, async () => {
                        const result = await api<{ content: string }>(
                          `/skills/${encodeURIComponent(skill.name)}?language=${language}`,
                        );
                        setDialog({
                          title:
                            localized[skill.name]?.title ||
                            readableName(skill.name),
                          text: markdownBody(result.content),
                        });
                      })
                    }
                  >
                    {t("Read instructions", "Ver instrucciones")}
                  </button>
                  <button
                    disabled={!!busy || !!waiting || !configured}
                    onClick={() => void run(skill.name, () => publish(skill))}
                  >
                    {waiting
                      ? t("Under review…", "En revisión…")
                      : published
                        ? t("Review latest changes", "Revisar cambios actuales")
                        : t("Make public", "Hacer público")}
                  </button>
                  {(published || waiting) && (
                    <button
                      disabled={!!busy}
                      onClick={() =>
                        void run(skill.name, async () => {
                          await api(
                            `/skill-exchange/publications/${(published || waiting)!.id}/visibility`,
                            { public: false },
                          );
                          await publications();
                        })
                      }
                    >
                      {t("Keep private", "Mantener privado")}
                    </button>
                  )}
                </div>
              </article>
            );
          })}
        </div>
      ) : tab === "market" ? (
        <>
          <div className="grid">
            {items.map((item) => (
              <article className="card skill-card" key={item.id}>
                <span className="eyebrow">{categoryLabel(item.category)}</span>
                <h2>
                  {localized[item.name]?.title ||
                    (language === "es"
                      ? "Versión en español no disponible"
                      : readableName(item.name))}
                </h2>
                <p className="muted">
                  {t(
                    "Version reviewed by Fénix Sentinel. Install your own private copy.",
                    "Versión revisada por Fénix Sentinel. Instala tu propia copia privada.",
                  )}
                </p>
                <span className="badge">
                  {item.repaired
                    ? t("Reviewed & corrected", "Revisado y corregido")
                    : t("Reviewed", "Revisado")}
                </span>
                <div className="actions">
                  <button
                    disabled={!!busy}
                    onClick={() =>
                      void run(item.id, async () => {
                        const b = await api<Bundle>(
                          `/skill-exchange/publications/${item.id}/bundle`,
                        );
                        const preview =
                          language === "es"
                            ? await publishedSpanish(b)
                            : { content: b.files["SKILL.md"] };
                        if (!preview)
                          throw new Error(
                            "La publicación aún no incluye una versión en español guardada.",
                          );
                        setDialog({
                          title:
                            localized[item.name]?.title ||
                            readableName(item.name),
                          text: markdownBody(preview.content),
                        });
                      })
                    }
                  >
                    {t("Preview", "Vista previa")}
                  </button>
                  <button
                    className="primary"
                    onClick={() => {
                      setInstallation(item);
                      setInstallName(item.name);
                    }}
                  >
                    {t("Install", "Instalar")}
                  </button>
                </div>
              </article>
            ))}
          </div>
          {!items.length && (
            <div className="card empty-state">
              <h2>
                {t(
                  "Room for the next great skill.",
                  "Un espacio para tu próxima habilidad.",
                )}
              </h2>
              <p>
                {t(
                  "No reviewed publications match this search yet.",
                  "Todavía no hay publicaciones revisadas que coincidan con esta búsqueda.",
                )}
              </p>
            </div>
          )}
          <div className="actions">
            <button
              disabled={offset === 0}
              onClick={() => setOffset(Math.max(0, offset - 24))}
            >
              {t("Previous", "Anterior")}
            </button>
            <button
              disabled={items.length < 24}
              onClick={() => setOffset(offset + 24)}
            >
              {t("Next", "Siguiente")}
            </button>
          </div>
        </>
      ) : (
        <div className="grid">
          {mine.map((p) => (
            <article className="card" key={p.id}>
              <h2>{p.name}</h2>
              <span className="badge" data-status={p.status}>
                {label(p)}
              </span>
              <p className="muted">
                {p.repaired
                  ? t(
                      "A corrected copy was saved. The original was preserved.",
                      "Se guardó una copia corregida. El original se conservó.",
                    )
                  : t(
                      "Each content version is reviewed once. Repeated clicks reuse the same review.",
                      "Cada versión se revisa una vez. Los clics repetidos reutilizan la misma revisión.",
                    )}
              </p>
              <div className="actions">
                <button
                  disabled={!!busy}
                  onClick={() =>
                    void run(p.id, async () => {
                      const data = await api<{ report: Report | null }>(
                        `/skill-exchange/publications/${p.id}`,
                      );
                      setDialog({ title: p.name, report: data.report });
                    })
                  }
                >
                  {t("View review", "Ver revisión")}
                </button>
                {p.status === "failed" && (
                  <button
                    disabled={!!busy}
                    onClick={() =>
                      void run(p.id, async () => {
                        await api(
                          `/skill-exchange/publications/${p.id}/retry`,
                          {},
                        );
                        await publications();
                      })
                    }
                  >
                    {t("Retry review", "Reintentar revisión")}
                  </button>
                )}
                {p.status === "approved" && !p.public && (
                  <button
                    disabled={!!busy}
                    onClick={() =>
                      void run(p.id, async () => {
                        await api(
                          `/skill-exchange/publications/${p.id}/visibility`,
                          { public: true },
                        );
                        await publications();
                      })
                    }
                  >
                    {t("Publish reviewed copy", "Publicar copia revisada")}
                  </button>
                )}
                {(p.public || ["queued", "reviewing"].includes(p.status)) && (
                  <button
                    disabled={!!busy}
                    onClick={() =>
                      void run(p.id, async () => {
                        await api(
                          `/skill-exchange/publications/${p.id}/visibility`,
                          { public: false },
                        );
                        await publications();
                      })
                    }
                  >
                    {t("Keep private", "Mantener privado")}
                  </button>
                )}
              </div>
            </article>
          ))}
          {!mine.length && (
            <p>
              {t(
                "Your publication reviews will appear here.",
                "Tus revisiones de publicación aparecerán aquí.",
              )}
            </p>
          )}
        </div>
      )}
      {dialog && (
        <div
          className="skill-modal"
          role="dialog"
          aria-modal="true"
          aria-label={dialog.title}
          onClick={(event) => {
            if (event.target === event.currentTarget) setDialog(null);
          }}
        >
          <section className="card">
            <div className="row spread">
              <h2>{dialog.title}</h2>
              <button
                type="button"
                className="subtle icon-button"
                aria-label={t("Close", "Cerrar")}
                onClick={() => setDialog(null)}
              >
                <Icon name="close" />
              </button>
            </div>
            {dialog.text !== undefined ? (
              <Markdown className="skill-reader">{dialog.text}</Markdown>
            ) : (
              <>
                <Markdown>
                  {dialog.report?.reason ||
                    t("Review is pending.", "La revisión está pendiente.")}
                </Markdown>
                {dialog.report?.repair && (
                  <>
                    <Markdown>{dialog.report.repair.explanation}</Markdown>
                    {dialog.report.repair.edits.map((edit, i) => (
                      <div key={i}>
                        <h3>{edit.file}</h3>
                        <small>{t("Before", "Antes")}</small>
                        <pre className="skill-before">{edit.before}</pre>
                        <small>
                          {t("Proposed correction", "Corrección propuesta")}
                        </small>
                        <pre className="skill-after">{edit.after}</pre>
                      </div>
                    ))}
                  </>
                )}
                <details>
                  <summary>{t("Full report", "Informe completo")}</summary>
                  <pre>{JSON.stringify(dialog.report, null, 2)}</pre>
                </details>
              </>
            )}
          </section>
        </div>
      )}
      {installation && (
        <div
          className="skill-modal"
          role="dialog"
          aria-modal="true"
          aria-label={t("Install private copy", "Instalar copia privada")}
          onClick={(event) => {
            if (event.target === event.currentTarget && !busy)
              setInstallation(null);
          }}
        >
          <form
            className="card"
            onSubmit={(e) => {
              e.preventDefault();
              void run(installation.id, async () => {
                const b = await api<Bundle>(
                  `/skill-exchange/publications/${installation.id}/bundle`,
                );
                await api("/skills/install", { bundle: b, name: installName });
                setSkills(await api<Skill[]>("/skills"));
                setInstallation(null);
                setNotice(
                  t(
                    "Private copy installed. Existing skills were not overwritten.",
                    "Copia privada instalada. Se conservaron los skills existentes.",
                  ),
                );
              });
            }}
          >
            <h2>{t("Make it yours.", "Hazlo tuyo.")}</h2>
            <label>
              {t("Private skill name", "Nombre del skill privado")}
              <input
                autoFocus
                required
                pattern="[A-Za-z0-9_-]+"
                maxLength={80}
                value={installName}
                onChange={(e) => setInstallName(e.target.value)}
              />
            </label>
            <p>
              {t(
                "Choose another name if this skill already exists.",
                "Elige otro nombre si este skill ya existe.",
              )}
            </p>
            {error && <p role="alert">{error}</p>}
            <div className="actions">
              <button type="submit" className="primary" disabled={!!busy}>
                {t("Install copy", "Instalar copia")}
              </button>
              <button type="button" onClick={() => setInstallation(null)}>
                {t("Cancel", "Cancelar")}
              </button>
            </div>
          </form>
        </div>
      )}
    </div>
  );
}
let root: Root | undefined;
export function stopSkills() {
  root?.unmount();
  root = undefined;
}
export function renderSkillsPage(language: string) {
  stopSkills();
  root = createRoot(document.getElementById("page")!);
  root.render(<Skills language={language} />);
}
