# Changelog / Historial de cambios

## 0.3.0 — 2026-09-15

### English

This release builds on the background subagents and project-scoped context shipped
in 0.2.0 on September 14, and packages the subsequent September 14–15 work.

- Editable asynchronous plans with milestones, dependencies, tracked questions,
  durable events and updates routed to the original channel.
- Adaptive plan review, specialist reviewers and evidence-based resolution of
  objections, using the configured model.
- Repository-aware coding: regex/glob search, multi-file patches, scoped
  `AGENTS.md` instructions and owner-configured LSP servers.
- Active-task compaction with durable checkpoints and recoverable source context.
- Interactive terminal input, persistent services, incremental redacted logs,
  paginated output recovery and configurable execution limits.
- Follow-up messages to existing subagents, preserving their context and IDs,
  with one delegation level and bounded retry counters.
- Deterministic on-demand tool discovery over the authorized catalog.
- Independent public web search with a background crawler, SQLite FTS5 index,
  concurrent HTTP reading and source links.
- A local Chromium browser controlled from Rust, persistent profiles, manual
  controls, an encrypted password vault, responsive viewports and a browser skill.
- Versioned English/Spanish documentation and matching 0.3.0 package metadata.

The browser model receives DOM text and geometry rather than screenshots.
Search coverage depends on the local corpus. Voice, autonomous memory learning
and automatic resumption after a crash remain future work.

### Español

Esta versión continúa los subagentes en segundo plano y el contexto por proyecto
publicados en 0.2.0 el 14 de septiembre, e incorpora el trabajo posterior del 14–15.

- Planes asíncronos editables con hitos, dependencias, preguntas vinculadas,
  eventos durables y actualizaciones entregadas al canal original.
- Revisión adaptativa de planes, especialistas y resolución de objeciones con
  evidencia, usando el modelo configurado.
- Programación con contexto del repositorio: búsquedas regex/glob, parches de
  varios archivos, instrucciones `AGENTS.md` y servidores LSP configurables.
- Compactación durante tareas activas con checkpoints y contexto recuperable.
- Entrada de terminal interactiva, servicios persistentes, logs incrementales
  redactados, recuperación por páginas y límites de ejecución configurables.
- Mensajes de seguimiento a subagentes existentes, conservando contexto e IDs,
  un nivel de delegación y contadores de intentos acotados.
- Descubrimiento determinista de herramientas bajo demanda sobre el catálogo autorizado.
- Búsqueda web pública propia con rastreador en segundo plano, índice SQLite FTS5,
  lectura HTTP concurrente y enlaces a las fuentes.
- Navegador Chromium local controlado desde Rust, perfiles persistentes, control
  manual, bóveda de contraseñas cifrada, vistas responsivas y un skill de navegación.
- Documentación en español e inglés y versiones de paquetes unificadas en 0.3.0.

El modelo recibe texto y geometría del DOM, no capturas de pantalla. La cobertura
de búsqueda depende del índice local. Voz, aprendizaje autónomo de memoria y
reanudación automática tras caídas siguen como trabajo futuro.
