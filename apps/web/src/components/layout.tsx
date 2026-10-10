import { ApplicationRouter, NavLink } from "./routing";
import { createRoot, type Root } from "react-dom/client";
import { flushSync } from "react-dom";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { Flame, Icon } from "./icons";
import { placeAccountControl } from "../login";
import { showExplorer } from "../workspace/store";
const root = createRoot(document.getElementById("app")!);
let pageRoot: Root | undefined;
export type Language = "es" | "en";
const translate = (language: string, en: string, es: string) =>
  language === "es" ? es : en;
interface ShellProps {
  page: string;
  language: Language;
  theme: string;
  model: string;
  labels: Record<string, string>;
}
function Preferences({
  language,
  theme,
}: {
  language: Language;
  theme: string;
}) {
  return (
    <>
      <button
        type="button"
        data-action="language"
        className="subtle"
        aria-label={translate(language, "Change language", "Cambiar idioma")}
      >
        {language === "es" ? "EN" : "ES"}
      </button>
      <button
        type="button"
        data-action="theme"
        className="subtle"
        aria-label={translate(
          language,
          "Toggle light and dark mode",
          "Cambiar modo claro y oscuro",
        )}
      >
        <Icon name={theme === "dark" ? "sun" : "moon"} />
      </button>
    </>
  );
}
function Shell({ page, language, theme, model, labels }: ShellProps) {
  const t = (en: string, es: string) => translate(language, en, es);
  const [collapsed, setCollapsed] = useState(
    localStorage.getItem("fenix.sidebar") === "collapsed",
  );
  const [mobile, setMobile] = useState(
    () => window.matchMedia("(max-width: 620px)").matches,
  );
  const [menuOpen, setMenuOpen] = useState(false);
  const menuButton = useRef<HTMLButtonElement>(null);
  const sidebar = useRef<HTMLElement>(null);
  useEffect(() => {
    const query = window.matchMedia("(max-width: 620px)");
    const update = () => {
      setMobile(query.matches);
      setMenuOpen(false);
    };
    query.addEventListener("change", update);
    return () => query.removeEventListener("change", update);
  }, []);
  useEffect(() => {
    setMenuOpen(false);
  }, [page]);
  useEffect(() => {
    if (!mobile || !menuOpen) return;
    sidebar.current?.querySelector<HTMLButtonElement>("button")?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        flushSync(() => setMenuOpen(false));
        menuButton.current?.focus();
      }
      if (event.key === "Tab") {
        const items =
          sidebar.current?.querySelectorAll<HTMLElement>("button, a[href]");
        if (!items?.length) return;
        const first = items[0],
          last = items[items.length - 1];
        if (event.shiftKey && document.activeElement === first) {
          event.preventDefault();
          last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault();
          first.focus();
        }
      }
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [mobile, menuOpen]);
  const closeMenu = () => {
    flushSync(() => setMenuOpen(false));
    menuButton.current?.focus();
  };
  return (
    <div
      className={`shell${collapsed ? " is-collapsed" : ""}${menuOpen ? " mobile-menu-open" : ""}`}
    >
      {mobile && menuOpen && (
        <button
          className="navigation-backdrop"
          aria-label={t("Close navigation", "Cerrar menú")}
          onClick={closeMenu}
        />
      )}
      <aside
        ref={sidebar}
        className="sidebar"
        id="main-sidebar"
        inert={mobile && !menuOpen}
        aria-label={t("Navigation", "Navegación")}
      >
        <div className="sidebar-top">
          <div className="brand">
            <Flame />
            <span className="brand-name">Fénix</span>
          </div>
          <div className="mobile-brand">
            <Preferences language={language} theme={theme} />
            {mobile && (
              <button
                type="button"
                className="subtle icon-button mobile-menu-close"
                aria-label={t("Close navigation", "Cerrar menú")}
                onClick={closeMenu}
              >
                <Icon name="close" />
              </button>
            )}
          </div>
        </div>
        <div className="nav-label">{t("Workspace", "Tu espacio")}</div>
        <nav
          className="nav"
          aria-label={t("Main navigation", "Navegación principal")}
        >
          {Object.entries(labels)
            .filter(([key]) => key !== "profile")
            .map(([key, label]) => (
              <NavLink
                key={key}
                onClick={() => {
                  if (mobile) closeMenu();
                }}
                to={`/${key}`}
                data-page={key}
                title={label}
                aria-label={label}
                className={page === key ? "active" : ""}
                aria-current={page === key ? "page" : undefined}
              >
                <span className="nav-icon" aria-hidden="true">
                  <Icon name={key} />
                </span>
                <span className="nav-text">{label}</span>
              </NavLink>
            ))}
        </nav>
        <div className="sidebar-bottom">
          <div id="account-controls" />
          <div className="row">
            <small className="grow sidebar-version">Fénix · v0.3.1</small>
          </div>
        </div>
      </aside>
      <main className="main" inert={mobile && menuOpen}>
        <header className="topbar">
          <div className="breadcrumb">
            <button
              ref={menuButton}
              className="subtle sidebar-toggle icon-button"
              type="button"
              aria-label={
                (mobile ? !menuOpen : collapsed)
                  ? t("Expand navigation", "Expandir menú")
                  : t("Collapse navigation", "Colapsar menú")
              }
              aria-expanded={mobile ? menuOpen : !collapsed}
              aria-controls="main-sidebar"
              onClick={() => {
                if (mobile) {
                  setMenuOpen(!menuOpen);
                  return;
                }
                setCollapsed(!collapsed);
                localStorage.setItem(
                  "fenix.sidebar",
                  collapsed ? "expanded" : "collapsed",
                );
              }}
            >
              <Icon name="menu" />
            </button>
            <span className="breadcrumb-brand">Fénix</span>
            <span className="breadcrumb-divider">/</span>
            <strong>{labels[page]}</strong>
          </div>
          <div className="topbar-actions">
            <div className="desktop-preferences">
              <Preferences language={language} theme={theme} />
            </div>
            <button
              type="button"
              className="subtle icon-button"
              aria-label={t("Open folders", "Abrir carpetas")}
              title={t("Open folders", "Abrir carpetas")}
              onClick={showExplorer}
            >
              <Icon name="projects" />
            </button>
          </div>
        </header>
        <div id="page" key={page} />
      </main>
    </div>
  );
}
export function renderShell(props: ShellProps) {
  placeAccountControl(true);
  pageRoot?.unmount();
  pageRoot = undefined;
  flushSync(() =>
    root.render(
      <ApplicationRouter>
        <Shell {...props} />
      </ApplicationRouter>,
    ),
  );
  placeAccountControl();
}
/** Existing administration controllers own only this isolated DOM boundary. */
export function renderSetupMarkup(html: string) {
  placeAccountControl(true);
  pageRoot?.unmount();
  pageRoot = undefined;
  flushSync(() =>
    root.render(
      <ApplicationRouter>
        <div dangerouslySetInnerHTML={{ __html: html }} />
      </ApplicationRouter>,
    ),
  );
  placeAccountControl();
}
export function renderChatLayout(language: Language, model: string) {
  const t = (en: string, es: string) => translate(language, en, es);
  const content: ReactNode = (
    <section className="chat-layout chat-history-loading" aria-busy="true">
      <div
        className="chat-history-skeleton"
        role="status"
        aria-label={t("Loading messages", "Cargando mensajes")}
      >
        {[0, 1, 2, 3].map((index) => (
          <div
            key={index}
            className={`chat-skeleton-message${index % 2 ? " from-user" : ""}`}
            aria-hidden="true"
          >
            <span className="chat-skeleton-avatar" />
            <div>
              <span />
              <span />
              <span />
            </div>
          </div>
        ))}
      </div>
      <div className="chat-scroll" id="chat-scroll">
        <div className="chat-inner">
          <button className="subtle load-older" data-action="older">
            {t("Earlier messages", "Mensajes anteriores")}
          </button>
          <div
            id="message-list"
            role="log"
            aria-live="polite"
            aria-relevant="additions"
          />
          <div id="live-message" aria-busy="true" />
        </div>
      </div>
      <div className="composer-area">
        <div className="composer-wrap">
          <div id="design-results" />
          <div className="activity-strip" id="activity-strip" role="status" />
          <form id="chat-form" className="composer">
            <label className="muted" htmlFor="chat-input" hidden>
              {t("Message Fénix", "Escribe a Fénix")}
            </label>
            <span id="document-upload-control" />
            <textarea
              id="chat-input"
              placeholder={t("Message Fénix…", "Escribe a Fénix…")}
              maxLength={32000}
              rows={1}
              required
            />
            <div className="composer-footer">
              <span id="voice-controls" />
              <button
                className="primary send"
                type="submit"
                aria-label={t("Send message", "Enviar mensaje")}
              >
                <Icon name="send" />
              </button>
            </div>
          </form>
          <div id="voice-availability" />
        </div>
      </div>
    </section>
  );
  pageRoot = createRoot(document.getElementById("page")!);
  flushSync(() => pageRoot!.render(content));
  window.dispatchEvent(new Event("fenix:view"));
}
