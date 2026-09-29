import { serviceError } from "../components/service-error";
import { Icon } from "../components/icons";
import { Markdown } from "../components/markdown";
import { useEffect, useRef, useState } from "react";
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
type Category = { id: string; labels: Record<string, string> };
type SkillCard = { name: string; title: string; description: string };
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
    throw new Error(
      serviceError(value?.error || `Request failed (${response.status})`),
    );
  return value as T;
}
const markdownBody = (source: string) =>
  source.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n/, "").trim();
function Skills({ language }: { language: string }) {
  const t = (en: string, es: string) => (language === "es" ? es : en);
  const [skills, setSkills] = useState<Skill[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [localized, setLocalized] = useState<Record<string, SkillCard>>({});
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [dialog, setDialog] = useState<{
    skill: Skill;
    title: string;
    content?: string;
    error?: string;
    revision?: string;
    mode?: "read" | "edit" | "delete";
  } | null>(null);
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [discardOpen, setDiscardOpen] = useState(false);
  const modal = useRef<HTMLDivElement>(null);
  const opening = useRef(0);
  const skillTitle = (skill: Skill) =>
    localized[skill.name]?.title || skill.name.replaceAll("-", " ");
  useEffect(() => {
    let active = true;
    setLoading(true);
    void Promise.all([
      api<Skill[]>("/skills"),
      api<Category[]>("/skill-categories"),
    ])
      .then(async ([list, taxonomy]) => {
        if (!active) return;
        setSkills(list);
        setCategories(taxonomy);
        setLocalized({});
        if (language === "es") {
          const cards: SkillCard[] = [];
          for (let offset = 0; offset < list.length; offset += 256) {
            const result = await api<{ cards: SkillCard[] }>(
              "/skill-localizations",
              {
                language,
                names: list
                  .slice(offset, offset + 256)
                  .map((skill) => skill.name),
              },
            );
            if (!active) return;
            cards.push(...result.cards);
          }
          setLocalized(
            Object.fromEntries(cards.map((card) => [card.name, card])),
          );
        }
      })
      .catch((e) => {
        if (active) setError(serviceError(e));
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
      opening.current++;
    };
  }, [language]);
  useEffect(() => {
    if (language !== "es" || loading) return;
    const missing = skills
      .filter((skill) => !localized[skill.name])
      .map((skill) => skill.name);
    if (!missing.length) return;
    let active = true;
    const timer = window.setTimeout(() => {
      void api<{ cards: SkillCard[] }>("/skill-localizations", {
        language,
        names: missing.slice(0, 256),
      })
        .then(({ cards }) => {
          if (active)
            setLocalized((previous) => ({
              ...previous,
              ...Object.fromEntries(cards.map((card) => [card.name, card])),
            }));
        })
        .catch(() => {
          if (active) setLocalized((previous) => ({ ...previous }));
        });
    }, 15000);
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [language, loading, skills, localized]);
  function dismiss() {
    setDirty(false);
    setDiscardOpen(false);
    opening.current++;
    setDialog(null);
  }
  function close() {
    if (saving) return;
    if (discardOpen) {
      setDiscardOpen(false);
      return;
    }
    if (dirty) {
      setDiscardOpen(true);
      return;
    }
    dismiss();
  }
  useEffect(() => {
    if (!discardOpen) return;
    const previous = document.activeElement as HTMLElement | null;
    modal.current
      ?.querySelector<HTMLButtonElement>(".skill-discard-keep")
      ?.focus();
    return () => {
      if (previous?.isConnected) previous.focus();
    };
  }, [discardOpen]);
  async function open(skill: Skill, mode: "read" | "edit" | "delete" = "read") {
    const request = ++opening.current;
    const title = skillTitle(skill);
    setDirty(false);
    setDialog({ skill, title, mode });
    try {
      const result = await api<{ content: string; revision?: string }>(
        mode === "read"
          ? `/skills/${encodeURIComponent(skill.name)}?language=${language}`
          : `/skills/${encodeURIComponent(skill.name)}/source${mode === "edit" ? `?language=${language}` : ""}`,
      );
      if (opening.current === request)
        setDialog({
          skill,
          title,
          mode,
          content: result.content,
          revision: result.revision,
        });
    } catch (e) {
      if (opening.current === request)
        setDialog({ skill, title, mode, error: serviceError(e) });
    }
  }
  const closeRef = useRef(close);
  closeRef.current = close;
  const dialogOpen = dialog !== null;
  useEffect(() => {
    if (!dialogOpen) return;
    const previous = document.activeElement as HTMLElement;
    modal.current?.querySelector<HTMLButtonElement>("button")?.focus();
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        closeRef.current();
      }
      if (event.key !== "Tab") return;
      const elements = Array.from(
        (
          modal.current?.querySelector<HTMLElement>('[role="alertdialog"]') ||
          modal.current
        )?.querySelectorAll<HTMLElement>(
          'button:not(:disabled), a[href], textarea, input, [tabindex="0"]',
        ) || [],
      ).filter((element) => element.getClientRects().length);
      const first = elements[0],
        last = elements.at(-1);
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last?.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first?.focus();
      }
    }
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("keydown", onKey);
      if (previous?.isConnected) previous.focus();
    };
  }, [dialogOpen]);
  const filtered = skills.filter(
    (skill) =>
      (!category || skill.category === category) &&
      `${skill.name} ${skill.description} ${localized[skill.name]?.title || ""} ${localized[skill.name]?.description || ""}`
        .toLowerCase()
        .includes(query.toLowerCase()),
  );
  return (
    <div className="content skill-exchange">
      <header className="skills-hero">
        <h1>Skills</h1>
      </header>
      <div className="skills-filters">
        <input
          aria-label={t("Search skills", "Buscar skills")}
          placeholder={t("Find a skill…", "Encuentra un skill…")}
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
        <select
          aria-label={t("Category", "Categoría")}
          value={category}
          onChange={(event) => setCategory(event.target.value)}
        >
          <option value="">
            {t("All categories", "Todas las categorías")}
          </option>
          {categories.map((item) => (
            <option key={item.id} value={item.id}>
              {item.labels[language] || item.labels.en || item.id}
            </option>
          ))}
        </select>
      </div>
      {error && (
        <p role="alert" className="form-status error">
          {error}
        </p>
      )}
      {loading ? (
        <Loading />
      ) : (
        <div className="grid">
          {filtered.map((skill) => (
            <article
              key={skill.name}
              className="card skill-card skill-card-clickable"
            >
              <button
                className="skill-card-open"
                aria-label={`${t("Open", "Abrir")} ${skillTitle(skill)}`}
                onClick={() => void open(skill)}
              />
              <div className="skill-title">
                <h2>{skillTitle(skill)}</h2>
                <span className="badge">v{skill.version}</span>
              </div>
              <span className="eyebrow">
                {categories.find((item) => item.id === skill.category)?.labels[
                  language
                ] || skill.category}
              </span>
              <p className="muted">
                {localized[skill.name]?.description || skill.description}
              </p>
              <div className="skill-card-controls">
                <label className="toggle">
                  <input
                    type="checkbox"
                    checked={skill.enabled}
                    disabled={busy !== null}
                    onChange={async (event) => {
                      const enabled = event.target.checked;
                      setBusy(skill.name);
                      setError("");
                      setSkills((list) =>
                        list.map((item) =>
                          item.name === skill.name
                            ? { ...item, enabled }
                            : item,
                        ),
                      );
                      try {
                        await api(
                          `/skills/${encodeURIComponent(skill.name)}`,
                          { enabled },
                          "PUT",
                        );
                      } catch (e) {
                        setSkills((list) =>
                          list.map((item) =>
                            item.name === skill.name
                              ? { ...item, enabled: skill.enabled }
                              : item,
                          ),
                        );
                        setError(serviceError(e));
                      } finally {
                        setBusy(null);
                      }
                    }}
                  />
                  {t("Enabled", "Activo")}
                </label>
                <div className="skill-management-actions">
                  <button
                    className="skill-action-button"
                    title={t("Edit skill", "Editar skill")}
                    aria-label={t("Edit", "Editar")}
                    onClick={() => void open(skill, "edit")}
                  >
                    <Icon name="edit" />
                  </button>
                  <button
                    className="skill-action-button skill-delete-button"
                    title={t("Delete skill", "Eliminar skill")}
                    aria-label={t("Delete", "Eliminar")}
                    onClick={() => void open(skill, "delete")}
                  >
                    <Icon name="trash" />
                  </button>
                </div>
              </div>
            </article>
          ))}
          {!filtered.length && (
            <p className="muted">
              {t("No skills found.", "No se encontraron skills.")}
            </p>
          )}
        </div>
      )}
      {dialog && (
        <div
          className="skill-modal"
          onClick={(event) => {
            if (event.target === event.currentTarget) close();
          }}
        >
          <div
            ref={modal}
            role="dialog"
            aria-modal="true"
            aria-labelledby="skill-dialog-title"
            className="card"
          >
            <div className="skill-dialog-main" inert={discardOpen || undefined}>
              <div className="skill-dialog-header">
                <div className="skill-dialog-heading">
                  <span className="eyebrow">
                    {dialog.mode === "edit"
                      ? t("EDIT SKILL", "EDITAR SKILL")
                      : dialog.mode === "delete"
                        ? t("YOUR LIBRARY", "TU BIBLIOTECA")
                        : t("INSTRUCTIONS", "INSTRUCCIONES")}
                  </span>
                  <h2 id="skill-dialog-title">{dialog.title}</h2>
                </div>
                <div className="skill-dialog-actions">
                  {(!dialog.mode || dialog.mode === "read") && (
                    <button
                      className="skill-action-button"
                      aria-label={t("Edit", "Editar")}
                      title={t("Edit skill", "Editar skill")}
                      onClick={() => void open(dialog.skill, "edit")}
                      disabled={dialog.content === undefined}
                    >
                      <Icon name="edit" />
                    </button>
                  )}
                  <button
                    className="subtle icon-button"
                    aria-label={t("Close", "Cerrar")}
                    onClick={close}
                    disabled={saving}
                  >
                    <Icon name="close" />
                  </button>
                </div>
              </div>
              <div className="skill-reader">
                {dialog.error ? (
                  <p role="alert">{dialog.error}</p>
                ) : dialog.content === undefined ? (
                  <Loading />
                ) : dialog.mode === "edit" ? (
                  <SkillEditor
                    key={dialog.skill.name}
                    name={dialog.skill.name}
                    content={dialog.content}
                    revision={dialog.revision!}
                    language={language}
                    onDirty={setDirty}
                    onSaving={setSaving}
                    onSaved={() => {
                      void api<Skill[]>("/skills")
                        .then(setSkills)
                        .catch((e) => setError(serviceError(e)));
                      setLocalized((previous) => {
                        const next = { ...previous };
                        delete next[dialog.skill.name];
                        return next;
                      });
                    }}
                  />
                ) : dialog.mode === "delete" ? (
                  <div className="skill-delete-confirm">
                    <p>
                      {t(
                        "Delete this skill from your library? Published versions and other users’ copies will stay unchanged.",
                        "¿Eliminar este skill de tu biblioteca? Las versiones publicadas y las copias de otros usuarios no cambiarán.",
                      )}
                    </p>
                    <div className="actions">
                      <button
                        disabled={saving}
                        className="skill-delete-confirm-button"
                        onClick={async () => {
                          setSaving(true);
                          try {
                            await api(
                              `/skills/${encodeURIComponent(dialog.skill.name)}/source`,
                              { revision: dialog.revision },
                              "DELETE",
                            );
                            setSkills((list) =>
                              list.filter(
                                (item) => item.name !== dialog.skill.name,
                              ),
                            );
                            opening.current++;
                            setDialog(null);
                          } catch (e) {
                            setDialog((value) =>
                              value
                                ? { ...value, error: serviceError(e) }
                                : null,
                            );
                          } finally {
                            setSaving(false);
                          }
                        }}
                      >
                        {saving
                          ? t("Deleting…", "Eliminando…")
                          : t("Delete skill", "Eliminar skill")}
                      </button>
                      <button disabled={saving} onClick={close}>
                        {t("Cancel", "Cancelar")}
                      </button>
                    </div>
                  </div>
                ) : (
                  <Markdown>{markdownBody(dialog.content)}</Markdown>
                )}
              </div>
            </div>
            {discardOpen && (
              <div className="skill-discard-overlay">
                <section
                  className="skill-discard-panel"
                  role="alertdialog"
                  aria-modal="true"
                  aria-labelledby="skill-discard-title"
                  aria-describedby="skill-discard-description"
                >
                  <span className="skill-discard-icon">
                    <Icon name="edit" />
                  </span>
                  <h3 id="skill-discard-title">
                    {t("Leave without saving?", "¿Salir sin guardar?")}
                  </h3>
                  <p id="skill-discard-description">
                    {t(
                      "You have unsaved changes. If you close this window, they will be lost.",
                      "Tienes cambios sin guardar. Si cierras esta ventana, se perderán.",
                    )}
                  </p>
                  <div className="skill-discard-actions">
                    <button
                      className="skill-action-button skill-discard-keep"
                      onClick={() => setDiscardOpen(false)}
                    >
                      {t("Keep editing", "Seguir editando")}
                    </button>
                    <button
                      className="skill-action-button skill-delete-confirm-button"
                      onClick={dismiss}
                    >
                      {t("Discard changes", "Descartar cambios")}
                    </button>
                  </div>
                </section>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
function SkillEditor({
  name,
  content,
  revision,
  language,
  onDirty,
  onSaving,
  onSaved,
}: {
  name: string;
  content: string;
  revision: string;
  language: string;
  onDirty: (value: boolean) => void;
  onSaving: (value: boolean) => void;
  onSaved: () => void;
}) {
  const t = (en: string, es: string) => (language === "es" ? es : en);
  const [text, setText] = useState(content);
  const [saved, setSaved] = useState(content);
  const [version, setVersion] = useState(revision);
  const [working, setWorking] = useState(false);
  const [error, setError] = useState("");
  const callbacks = useRef({ onSaved, onDirty, onSaving });
  callbacks.current = { onSaved, onDirty, onSaving };
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  useEffect(() => {
    callbacks.current.onDirty(text !== saved);
    const warn = (event: BeforeUnloadEvent) => {
      if (text !== saved) {
        event.preventDefault();
        event.returnValue = "";
      }
    };
    window.addEventListener("beforeunload", warn);
    return () => window.removeEventListener("beforeunload", warn);
  }, [text, saved]);
  const saveLock = useRef(false);
  async function save() {
    if (text === saved || saveLock.current) return;
    saveLock.current = true;
    const snapshot = text;
    setWorking(true);
    setError("");
    callbacks.current.onSaving(true);
    try {
      const result = await api<{ revision: string }>(
        `/skills/${encodeURIComponent(name)}/source`,
        { content: snapshot, revision: version, language },
        "PUT",
      );
      if (mounted.current) {
        setSaved(snapshot);
        setVersion(result.revision);
        callbacks.current.onDirty(false);
        callbacks.current.onSaved();
      }
    } catch (e) {
      if (mounted.current) setError(serviceError(e));
    } finally {
      saveLock.current = false;
      if (mounted.current) {
        setWorking(false);
        callbacks.current.onSaving(false);
      }
    }
  }
  return (
    <div className="skill-editor">
      <p className="muted">
        {t(
          "Edit the English instructions. Saving also translates the Spanish version.",
          "Edita las instrucciones en español. Al guardar también se traduce la versión en inglés.",
        )}
      </p>
      <textarea
        aria-label={t("Skill instructions", "Instrucciones del skill")}
        spellCheck={false}
        readOnly={working}
        value={text}
        onChange={(event) => {
          setText(event.target.value);
          callbacks.current.onDirty(event.target.value !== saved);
          setError("");
        }}
      />
      {error && (
        <p role="alert" className="skill-editor-error">
          {error}
        </p>
      )}
      <div className="skill-editor-footer">
        <div
          role="status"
          className="skill-save-status"
          data-dirty={text !== saved}
        >
          <Icon
            name={working ? "refresh" : text === saved ? "check" : "edit"}
          />
          <span>
            {working
              ? t("Saving…", "Guardando…")
              : text === saved
                ? t("All changes saved", "Todos los cambios guardados")
                : t("Unsaved changes", "Cambios sin guardar")}
          </span>
        </div>
        <button
          className="primary skill-action-button skill-save-button"
          aria-label={t("Save changes", "Guardar cambios")}
          title={t("Save changes", "Guardar cambios")}
          aria-busy={working}
          disabled={working || text === saved}
          onClick={() => void save()}
        >
          <Icon name={working ? "refresh" : "save"} />
        </button>
      </div>
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
