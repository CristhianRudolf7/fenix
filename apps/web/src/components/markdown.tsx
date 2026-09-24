import { useMemo } from "react";
import { Marked } from "marked";
import DOMPurify from "dompurify";
import "./markdown.css";

const escapeHtml = (text: string) =>
  text.replace(
    /[&<>"']/g,
    (character) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        character
      ]!,
  );
const parser = new Marked({
  gfm: true,
  breaks: true,
  renderer: {
    // Skill instructions and model output are untrusted. Raw HTML is displayed,
    // never interpreted, and Markdown images do not make tracking requests.
    html: ({ text }) => escapeHtml(text),
    image: ({ text }) => escapeHtml(text),
    checkbox: ({ checked }) =>
      `<span class="markdown-checkbox" aria-label="${checked ? "Completed" : "Pending"}">${checked ? "☑" : "☐"}</span> `,
  },
});

/** Shared safe output for both legacy DOM views and React components. */
export function markdownHtml(source: string): string {
  const html = parser.parse(source, { async: false });
  const fragment = DOMPurify.sanitize(html, {
    RETURN_DOM_FRAGMENT: true,
    ALLOWED_TAGS: [
      "p",
      "br",
      "strong",
      "em",
      "del",
      "s",
      "h1",
      "h2",
      "h3",
      "h4",
      "h5",
      "h6",
      "blockquote",
      "ul",
      "ol",
      "li",
      "pre",
      "code",
      "hr",
      "a",
      "table",
      "thead",
      "tbody",
      "tr",
      "th",
      "td",
      "span",
    ],
    ALLOWED_ATTR: ["href", "title", "start", "class", "aria-label", "align"],
    ALLOW_DATA_ATTR: false,
  });
  for (const link of fragment.querySelectorAll("a")) {
    const href = link.getAttribute("href");
    if (!href) continue;
    try {
      const url = new URL(href, window.location.href);
      if (!["https:", "http:", "mailto:"].includes(url.protocol)) {
        link.removeAttribute("href");
        continue;
      }
      link.setAttribute("rel", "noopener noreferrer");
      if (!href.startsWith("#")) link.setAttribute("target", "_blank");
    } catch {
      link.removeAttribute("href");
    }
  }
  for (const table of fragment.querySelectorAll("table")) {
    const scroll = document.createElement("div");
    scroll.className = "markdown-table";
    scroll.tabIndex = 0;
    scroll.setAttribute("role", "region");
    scroll.setAttribute(
      "aria-label",
      document.documentElement.lang === "en"
        ? "Table, scroll horizontally"
        : "Tabla, desplazamiento horizontal",
    );
    table.replaceWith(scroll);
    scroll.append(table);
  }
  const container = document.createElement("div");
  container.append(fragment);
  return container.innerHTML;
}
export function Markdown({
  children,
  className = "",
}: {
  children: string;
  className?: string;
}) {
  const html = useMemo(() => markdownHtml(children), [children]);
  return (
    <div
      className={`markdown-content ${className}`}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
