# Fénix capability map and YC assessment / Mapa de capacidades y valoración para YC

Date / Fecha: 2026-09-27. Scope: current working tree, including uncommitted code. Static inspection of tool registries, implementation paths, startup wiring, skills metadata and supporting documentation; not a line-by-line security audit, live-account test or deployment certification. No product code was changed and no tests were rerun for this assessment. Historical test results are not presented as fresh results.

Alcance: inventario estático del código actual, incluidas modificaciones locales; no certifica la instancia desplegada ni la calidad de cada tarea. Se revisaron los puntos de entrada, módulos y documentación indicados, no cada línea de todo el repositorio. La biblioteca de skills se inventarió por metadatos: no se releyeron aquí los 187 procedimientos íntegros.

## Assessment / Valoración

Yes: the proposed subscription assistant with persistent private workspaces is a credible YC-category business. This is a judgment about fit and possibility, not a prediction of admission or a probability estimate. Product breadth alone cannot establish customer demand, retention, differentiation, founder strength, distribution or profitable unit economics. MVP incompleteness is deliberately excluded from this judgment.

Sí: el asistente por suscripción con un espacio persistente por usuario encaja como propuesta potencial para YC. La amplitud funcional aporta una base de producto, pero no prueba demanda, retención, diferenciación, capacidad del equipo, distribución ni margen. No penalizo lo pendiente del MVP. Una primera propuesta comercial plausible es resolver trabajo recurrente de pequeños equipos de servicios entre correo, documentos y datos; es una hipótesis que requiere clientes, no una conclusión demostrada por el repositorio.

Official references / Fuentes oficiales:

- [YC application guidance](https://www.ycombinator.com/howtoapply): clarity, founder achievements and specific insight into differentiation.
- [YC FAQ](https://www.ycombinator.com/faq): early-stage applications are welcome and similar portfolio companies are not an automatic exclusion.
- [Tasklet](https://www.ycombinator.com/companies/tasklet-2): YC lists an agent product with persistent cloud work and connections.
- [Rowboat](https://www.ycombinator.com/companies/rowboat-labs): YC lists a coworker with work memory and connected work surfaces.

These portfolio descriptions establish category precedent and overlap, not Fénix's superiority or its likelihood of admission. Las descripciones de estas empresas muestran precedentes y competencia; no demuestran superioridad de Fénix.

## Capability families / Familias funcionales

“Implemented” below means a relevant code or packaged procedure path exists. External accounts, models, binaries and operator configuration may still be required. Most specialized skills orchestrate tools rather than introduce their own executable engine.

| Family / Familia | Scope and limits / Alcance y límites | Evidence / Evidencia |
| --- | --- | --- |
| Conversation / Conversación | Continuous chat, projects, task traces, bilingual UI, optional owner Telegram and browser dictation/Live. Live observes and forwards owner requests; it is not an unrestricted execution agent. | [Source](../../crates/fenix-agent/src/conversation.rs) · [Guide](../en/dictation.md) |
| Projects / Proyectos | Fixed per-task project bindings, aliases, briefs, scoped context, requirements with stable IDs and versioned amendments. | [Source](../../crates/fenix-agent/src/project_specs.rs) · [Guide](../en/project-contracts.md) |
| Autonomy / Autonomía | Owner-sourced task/project/account preferences; autonomous implementation choices preserve requirement clarification and actual permission boundaries. | [Source](../../crates/fenix-agent/src/autonomy.rs) · [Guide](../en/conversational-autonomy.md) |
| Plans and delegation / Planes y delegación | Background workers, dependency milestones, revisions, steering, bounded retries, independent plan reviews and completion audits. | [Source](../../crates/fenix-agent/src/planning.rs) · [Guide](../en/planning.md) |
| Completion controls / Controles de finalización | Task contracts, evidence ledger, observed tool receipts, explicit completion/blocked outcomes, required-check gates and evidence-based stop handling. These are not a proof of arbitrary natural-language correctness. | [Source](../../crates/fenix-agent/src/completion_gate.rs) · [Guide](../en/completion-audits.md) |
| Adaptive memory / Memoria adaptativa | Sourced facts, candidates, validity/revisions, scoped entities and relation schemas, lexical/relational retrieval, optional embeddings, background capture and contradiction review. | [Source](../../crates/fenix-agent/src/memory.rs) · [Guide](../en/adaptive-memory.md) |
| Files and coding / Archivos y programación | Confined reads/writes/edits, glob/regex searches, multi-file patches, repository guidance, configurable LSP, Git worktree coordination. | [Source](../../crates/fenix-agent/src/files.rs) · [Guide](../en/coding-harness.md) |
| Execution / Ejecución | Shell, interactive terminal input, cancellation, incremental logs, persistent processes and bounded command outputs. Available environment determines supported programs. | [Source](../../crates/fenix-agent/src/processes.rs) · [Guide](../en/execution-harness.md) |
| History and recovery / Historial y recuperación | Private before/after file versions with conflict-aware restore; conservative restart admission from complete tool boundaries; context compaction and recoverable evidence. No arbitrary external-effect rollback or RAM restoration. | [Source](../../crates/fenix-agent/src/restart_recovery.rs) · [Guide](../en/harness-continuity.md) |
| Tool composition / Composición de herramientas | Declarative references, conditions, loops and projections over authorized tools, up to 50 calls without intermediate model turns. Not a transaction or a general interpreter. | [Source](../../crates/fenix-agent/src/tool_program.rs) · [Guide](../en/harness-continuity.md) |
| Schedules and workflows / Horarios y flujos | One-off/interval/cron tasks, durable versioned workflows, connector/browser/script/prompt/condition/wait/notify nodes, scoped outputs and checkpoints. | [Source](../../crates/fenix-agent/src/workflows/mod.rs) · [Guide](../en/workflows-and-connectors.md) |
| Web research / Investigación web | Own crawler/index and lexical ranking, bounded discovery, source reads, provenance, freshness, caching and concurrent request coordination. Corpus-dependent, not universal internet coverage. | [Source](../../crates/fenix-web/src/search.rs) · [Guide](../en/web-search.md) |
| Interactive browser / Navegador interactivo | Chromium profiles, DOM actions, forms, uploads/downloads, frames/tabs, wait/check conditions, viewport tests, screenshots and owner-authorized saved logins. | [Source](../../crates/fenix-browser/src/tools.rs) · [Guide](../en/browser.md) |
| Connections / Conexiones | Google Workspace across 11 services; Microsoft Graph, Slack, GitHub, IMAP/SMTP, PostgreSQL and SQLite. Owner Telegram and WhatsApp Web routes also exist. Credentials/scopes remain necessary. | [Source](../../crates/fenix-connectors/src/lib.rs) · [Guide](../en/google-workspace.md) |
| Local Office / Ofimática local | Create/read/edit ordinary DOCX/XLSX/PPTX, bounded extraction and hash-checked modifications. Formula strings are not calculated; imported Office layout is not faithfully rendered. | [Source](../../crates/fenix-documents/src/lib.rs) · [Guide](../en/skills-and-documents.md) |
| PDF and OCR / PDF y OCR | 14 helper operation families including assembly, split, rotate, crop, watermark, numbering, metadata, compression, text-form fill, image conversion and raster redaction; CPU OCR via prepared dependencies. | [Source](../../resources/skills/pdf-workflows/SKILL.md) · [Guide](../en/pdf-toolkit.md) |
| Knowledge library / Biblioteca | Account/project-scoped import, list/search/read/export of sources with recoverable citations and document context. | [Source](../../crates/fenix-agent/src/library/mod.rs) · [Guide](../en/document-library.md) |
| Visual deliverables / Entregables visuales | 72 template examples, HTML previews, Mermaid diagrams, charts, presentations, documents, tables, images and video; brand metadata, revisions and exports within format limits. | [Source](../../resources/designs/catalog.json) · [Guide](../en/design.md) |
| Workspace viewer / Visor | Shared right panel with files, library and design tabs; PDF page navigation/zoom; Markdown, HTML, images/video and extracted Office contents. Agent performs file/PDF edits. | [Source](../../crates/fenix-agent/src/workspace_view.rs) · [Guide](../en/workspace-viewer.md) |
| Skills / Habilidades | 187 bundled procedures, demand-loaded search and resources, enable/disable controls, hash-based stock upgrades that preserve customizations. Specialized skills are not all native integrations. | [Source](../../crates/fenix-agent/src/skills.rs) · [Guide](../en/skill-library.md) |
| Procedural learning / Aprendizaje de procedimientos | Selective evidence-backed candidate generation, model review, automatic private activation, bounded learned-skill updates, rollback, pinning and idle archival. Not model-weight training or measured universal improvement. | [Source](../../crates/fenix-agent/src/skill_learning.rs) · [Guide](../en/harness-continuity.md) |
| Marketplace / Mercado de skills | Immutable reviewed public snapshots, search/install private copies, publication revisions/withdrawal and Sentinel checks with bounded model-assisted corrections. No guaranteed malware detection or demonstrated network effect. | [Source](../../apps/server/src/skill_guard/mod.rs) · [Guide](../en/skill-marketplace.md) |
| Security and isolation / Seguridad y aislamiento | Account sessions/revocation, per-account controller/state, project scope enforcement, credential broker, gVisor default execution policy, resource/network limits, encrypted file checkpoints and specialist security profile. Deployment-dependent, not an escape-resistance certification. | [Source](../../scripts/fenix_execution/policy.py) · [Guide](../en/execution-isolation.md) |
| Models, efficiency and evaluation / Modelos, eficiencia y evaluación | 17 configured provider adapters, five protocols, usage/context accounting, optional Jev decision layer with fallback/shadow mode and native evaluation harness. No guaranteed cheaper inference or superior benchmark score. | [Source](../../crates/fenix-providers/src/catalog.rs) · [Guide](../en/decision-layer.md) |

## Recent additions verified in source / Adiciones recientes comprobadas en código

The current source goes beyond earlier README descriptions of five skills and no restart recovery. Important additions include automatic procedural learning/lifecycle, private file history, safe-boundary recovery, tool programs, evidence-based completion/stop gates, conversational autonomy, richer temporal/relational memory and optional typed decisions. Existing Google Workspace/PDF/visual/workflow capabilities remain part of the product. These are observed capabilities, not all attributed to a single commit or conversation.

El código actual supera las descripciones antiguas del README sobre cinco skills y ausencia de recuperación. Destacan aprendizaje privado de procedimientos, historial reversible, recuperación en límites seguros, composición de herramientas, controles de finalización/bloqueo, autonomía conversacional, memoria temporal/relacional y decisiones tipadas opcionales. El número de skills o controles no demuestra por sí solo mayor inteligencia.

## Declared tool inventory / Inventario de herramientas declaradas

82 unique names: 72 in the agent registry plus 10 browser names. This is the union across roles, not the simultaneously available catalog of one task; profile/project authorization and on-demand discovery further restrict access. Actions within one tool are not counted as separate tools.

- `ask_user`
- `autonomy_manage`
- `code_query`
- `command_cancel`
- `command_input`
- `command_list`
- `command_output`
- `command_run`
- `command_status`
- `completion_audit_read`
- `connector_call`
- `design_import`
- `design_list`
- `design_present`
- `design_publish`
- `design_read`
- `design_templates`
- `diagram_edit`
- `document_create`
- `document_edit`
- `document_extract`
- `document_read`
- `environment_inspect`
- `execution_status`
- `file_edit`
- `file_history`
- `file_patch`
- `file_read`
- `file_write`
- `files_glob`
- `files_list`
- `files_search`
- `history_read`
- `history_search`
- `integration_call`
- `knowledge_manage`
- `library_export`
- `library_import`
- `library_list`
- `library_read`
- `library_search`
- `memory_context`
- `memory_manage`
- `memory_search`
- `plan_evidence`
- `plan_manage`
- `plan_review_read`
- `project_manage`
- `project_spec`
- `request_approval`
- `schedule_manage`
- `security_scan_read`
- `skill_learning`
- `skill_list`
- `skill_load`
- `skill_search`
- `subagent_cancel`
- `subagent_finish`
- `subagent_send`
- `subagent_spawn`
- `subagent_status`
- `task_contract`
- `task_finish`
- `task_inspect`
- `tool_program`
- `tool_search`
- `web_read`
- `web_search`
- `workflow_manage`
- `workspace_checkpoint`
- `workspace_open`
- `worktree_manage`

Browser / Navegador:

- `browser_connection`
- `browser_open`
- `browser_observe`
- `browser_act`
- `browser_resize`
- `browser_inspect`
- `browser_close`
- `browser_wait`
- `browser_tabs`
- `browser_check`

## Skill inventory / Inventario de skills

187 packaged skills, not 187 independently validated end-to-end products. Specialized domains such as CAD, ML, media, home automation and research depend on their declared tools, inputs and authorized connections.

| Category / Categoría | Count / Cantidad |
| --- | ---: |
| business | 16 |
| coordination | 5 |
| creative | 8 |
| data | 17 |
| engineering | 48 |
| knowledge | 6 |
| office | 20 |
| operations | 20 |
| personal | 8 |
| research | 15 |
| security | 19 |
| writing | 5 |

| Skill | Category | Declared scope / Alcance declarado |
| --- | --- | --- |
| [3d-printing-preparation](../../resources/skills/3d-printing-preparation/SKILL.md) | engineering | Inspect mesh printability and prepare slicer-generated G-code using an actual printer, material and process profile. Preparar impresión 3D y revisar G-code. |
| [ab-test-analysis](../../resources/skills/ab-test-analysis/SKILL.md) | data | Design or analyze randomized product experiments with assignment integrity and decision uncertainty. |
| [agent-security-evaluation](../../resources/skills/agent-security-evaluation/SKILL.md) | security | Evaluate agent, RAG and MCP security through controlled prompt-injection, memory, identity and tool-effect tests. Evaluar seguridad del agente con canarios y efectos simulados. |
| [agent-workflow-diagnosis](../../resources/skills/agent-workflow-diagnosis/SKILL.md) | operations | Investigate an agent session that repeated work, missed a skill, ignored instructions or spent excessive time or tokens, using task traces. Diagnosticar ejecuciones del agente. |
| [analytics-instrumentation](../../resources/skills/analytics-instrumentation/SKILL.md) | data | Define and verify product events, funnels and retention metrics with consistent identities and denominators. |
| [api-contract-design](../../resources/skills/api-contract-design/SKILL.md) | engineering | Design or change API contracts with explicit inputs, errors, authorization, pagination and compatibility. |
| [api-integration](../../resources/skills/api-integration/SKILL.md) | engineering | Integrate external APIs with schema validation, pagination, retry discipline and duplicate-effect prevention. |
| [api-security-testing](../../resources/skills/api-security-testing/SKILL.md) | security | Test API authorization, protected properties and session boundaries with controlled requests and verified effects. Pruebas BOLA, IDOR, asignación masiva y WebSocket entre usuarios. |
| [architecture-design](../../resources/skills/architecture-design/SKILL.md) | engineering | Choose system boundaries and design tradeoffs from concrete workloads and failure modes. |
| [audio-production](../../resources/skills/audio-production/SKILL.md) | creative | Produce spoken narration, podcasts or mixed audio from a script using available TTS and audio tools, verifying the actual rendered file. Producción de voz y audio. |
| [audio-transcription](../../resources/skills/audio-transcription/SKILL.md) | creative | Transcribe or prepare audio with timestamps, uncertainty and source-linked corrections. |
| [authentication-integration](../../resources/skills/authentication-integration/SKILL.md) | engineering | Implement login, sessions or OAuth with correct identity binding, token handling and server-side authorization. |
| [automation-audit](../../resources/skills/automation-audit/SKILL.md) | operations | Inventory configured automations, jobs, hooks and connectors; verify their actual state and identify duplicate effects or missing coverage. Auditar automatizaciones existentes. |
| [background-jobs](../../resources/skills/background-jobs/SKILL.md) | engineering | Design durable job execution with leases, idempotency, retries and explicit partial effects. |
| [backup-and-restore](../../resources/skills/backup-and-restore/SKILL.md) | operations | Create and verify backups or restore data with consistency, collision and recovery-time checks. |
| [bibliography-management](../../resources/skills/bibliography-management/SKILL.md) | research | Verify and maintain BibTeX or reference lists with stable identifiers and correct document linkage. |
| [brand-and-design-tokens](../../resources/skills/brand-and-design-tokens/SKILL.md) | creative | Define or reconcile reusable brand tokens and component states without replacing established identity. |
| [browser-ml-inference](../../resources/skills/browser-ml-inference/SKILL.md) | data | Implement client-side or JavaScript model inference with Transformers.js, ONNX, WebGPU or WASM and verified device fallback. Inferencia de modelos en el navegador. |
| [browser-navigation](../../resources/skills/browser-navigation/SKILL.md) | operations | Navigate rendered websites, use persistent personal logins with owner access, and test responsive pages using the local browser. Navegar sitios, usar sesiones personales autorizadas y probar diseño responsivo. |
| [browser-regression-testing](../../resources/skills/browser-regression-testing/SKILL.md) | engineering | Create reproducible browser regression checks for user flows and diagnose flaky interactions. |
| [cache-design](../../resources/skills/cache-design/SKILL.md) | engineering | Design or diagnose caches with explicit keys, freshness, invalidation and stampede behavior. |
| [cad-drawing-and-fabrication](../../resources/skills/cad-drawing-and-fabrication/SKILL.md) | engineering | Prepare and inspect 2D DXF cutting drawings, fabrication layers and supplier-specific handoffs with explicit units and geometric checks. Planos DXF y corte láser. |
| [calendar-scheduling](../../resources/skills/calendar-scheduling/SKILL.md) | office | Find availability and create, reschedule or cancel calendar events with correct timezones and recurrence scope. |
| [cloud-cost-review](../../resources/skills/cloud-cost-review/SKILL.md) | operations | Explain cloud or API spending using actual usage, billing dimensions and defensible alternatives. |
| [cloud-deployment](../../resources/skills/cloud-deployment/SKILL.md) | operations | Deploy an application to the selected cloud while preserving account, region, identity and rollback scope. |
| [cloud-file-sync](../../resources/skills/cloud-file-sync/SKILL.md) | office | Transfer or reconcile local and cloud files with stable identity, conflict detection and resumable verification. |
| [code-health-review](../../resources/skills/code-health-review/SKILL.md) | engineering | Report repository check health from its configured type checker, linter and tests, with explicit failures, missing coverage and comparable history. Informe de salud del código. |
| [code-review](../../resources/skills/code-review/SKILL.md) | engineering | Review a concrete code diff against requirements and repository conventions, reporting actionable defects with evidence. Revisar diferencias de código sin modificar archivos. |
| [competitive-research](../../resources/skills/competitive-research/SKILL.md) | business | Compare products and market alternatives with current evidence and explicit comparison criteria. |
| [completion-verification](../../resources/skills/completion-verification/SKILL.md) | coordination | Check whether delivered artifacts or delegated work actually meet requirements before accepting a completion claim, especially with missing or stale evidence. Verificar entregas y finalización. |
| [container-workflows](../../resources/skills/container-workflows/SKILL.md) | operations | Build and diagnose containers with reproducible images, correct mounts and graceful process lifecycle. |
| [content-calendar](../../resources/skills/content-calendar/SKILL.md) | creative | Plan and prepare a coherent publishing calendar with real assets, owners and channel requirements. |
| [contract-review](../../resources/skills/contract-review/SKILL.md) | business | Summarize and compare contract terms, obligations and ambiguities for an informed review. |
| [conversation-recap](../../resources/skills/conversation-recap/SKILL.md) | knowledge | Recover decisions, progress and pending work from the continuous conversation with references. |
| [conversion-optimization](../../resources/skills/conversion-optimization/SKILL.md) | business | Diagnose conversion drop-off in landing pages, forms, pricing or onboarding and propose measurable improvements. CRO, abandono de formularios y activación. |
| [cpp-development](../../resources/skills/cpp-development/SKILL.md) | engineering | Implement C or C++ changes involving memory ownership, lifetime, ABI and native build behavior. |
| [crm-workflows](../../resources/skills/crm-workflows/SKILL.md) | office | Maintain CRM contacts, companies and deals with identity resolution and auditable field updates. |
| [customer-support](../../resources/skills/customer-support/SKILL.md) | business | Triage support cases and draft accurate responses using account context and known evidence. |
| [dashboard-design](../../resources/skills/dashboard-design/SKILL.md) | data | Create decision-focused dashboards with traceable metrics, comparable periods and readable visual encodings. |
| [data-analysis](../../resources/skills/data-analysis/SKILL.md) | research | Clean and analyze tabular datasets with reproducible transformations, checked units and honest statistical conclusions. |
| [data-pipelines](../../resources/skills/data-pipelines/SKILL.md) | data | Merge and normalize JSON CSV Parquet datasets, deduplicate users by source priority, report conflicts; build ETL/ELT pipelines with stable keys and recoverable data flow. |
| [data-quality-audit](../../resources/skills/data-quality-audit/SKILL.md) | data | Audit dataset completeness, validity, joins and lineage before analysis or ingestion. |
| [database-migrations](../../resources/skills/database-migrations/SKILL.md) | engineering | Apply schema and data migrations with compatibility, restartability and verified recovery. |
| [database-schema-design](../../resources/skills/database-schema-design/SKILL.md) | engineering | Design relational schemas with explicit identities, constraints, tenancy and query-driven indexes. |
| [dependency-and-secret-audit](../../resources/skills/dependency-and-secret-audit/SKILL.md) | security | Inspect dependency risk and accidental secret exposure using manifests, lockfiles and available scanners. |
| [desktop-automation](../../resources/skills/desktop-automation/SKILL.md) | operations | Operate native desktop applications through available accessibility or screen-control tools with fresh window evidence and verified saved outputs. Automatización de escritorio. |
| [detection-engineering](../../resources/skills/detection-engineering/SKILL.md) | security | Author, tune or evaluate security detection rules against actual telemetry, benign negatives and backend field mappings. Reglas Sigma, SIEM y YARA-L. |
| [developer-experience-review](../../resources/skills/developer-experience-review/SKILL.md) | engineering | Evaluate onboarding and usability of a developer-facing API, CLI, SDK or documentation, from first setup to a verified useful result. Auditar experiencia del desarrollador. |
| [diagram-authoring](../../resources/skills/diagram-authoring/SKILL.md) | creative | Create accurate flow, sequence, architecture or entity diagrams from concrete system evidence. |
| [document-comparison](../../resources/skills/document-comparison/SKILL.md) | office | Compare document versions for substantive changes while preserving structure and evidence references. |
| [documents-presentations](../../resources/skills/documents-presentations/SKILL.md) | office | Create PowerPoint PPTX decks with titled slides and bullet text; read and revise existing slide text with native Fenix Documents. |
| [documents-spreadsheets](../../resources/skills/documents-spreadsheets/SKILL.md) | office | Create and inspect Excel XLSX workbooks, tables of data and formulas; update individual cells while preserving existing workbook parts with native Fenix Documents. |
| [documents-word](../../resources/skills/documents-word/SKILL.md) | office | Create Word DOCX reports with headings, paragraphs and tables; read and edit existing document text using native Fenix Documents. |
| [dotnet-development](../../resources/skills/dotnet-development/SKILL.md) | engineering | Implement C# and .NET changes with correct dependency lifetimes, async I/O and persistence boundaries. |
| [email-workflows](../../resources/skills/email-workflows/SKILL.md) | office | Search email, triage threads and draft or send replies within the user-authorized scope. |
| [employee-onboarding](../../resources/skills/employee-onboarding/SKILL.md) | business | Prepare role-specific onboarding plans with ownership, access dependencies and observable milestones. |
| [engineering-retrospective](../../resources/skills/engineering-retrospective/SKILL.md) | coordination | Review engineering delivery, rework and bottlenecks over a defined period using repository, PR and incident evidence. Retrospectiva de ingeniería. |
| [environment-and-dependencies](../../resources/skills/environment-and-dependencies/SKILL.md) | engineering | Prepare development environments and diagnose dependency, build and installation failures. Entornos y dependencias. |
| [experiment-and-benchmark-design](../../resources/skills/experiment-and-benchmark-design/SKILL.md) | research | Design reproducible experiments and fair model or harness benchmarks with quality, latency, token and cost accounting. |
| [feature-rollouts](../../resources/skills/feature-rollouts/SKILL.md) | operations | Introduce and retire feature flags with stable targeting, measured rollout and tested fallback. |
| [feed-monitoring](../../resources/skills/feed-monitoring/SKILL.md) | knowledge | Inspect RSS or Atom subscriptions, compare feed snapshots and prepare sourced digests while preserving read and delivery state. Feeds, novedades y seguimiento de blogs. |
| [financial-modeling](../../resources/skills/financial-modeling/SKILL.md) | business | Build transparent budgets, cash-flow forecasts and financial scenarios with auditable assumptions. |
| [forms-and-surveys](../../resources/skills/forms-and-surveys/SKILL.md) | office | Design and process forms or surveys with clear questions, valid branching and interpretable responses. |
| [frontend-design-and-accessibility](../../resources/skills/frontend-design-and-accessibility/SKILL.md) | engineering | Build or improve web interfaces with consistent components, responsive states and accessible interaction. |
| [game-development](../../resources/skills/game-development/SKILL.md) | engineering | Build or repair playable games with explicit simulation timing, input, collision, pause/restart and target-platform checks. Desarrollo de videojuegos. |
| [geospatial-analysis](../../resources/skills/geospatial-analysis/SKILL.md) | data | Analyze geographic data with correct coordinate systems, spatial joins and location privacy. |
| [git-recovery-and-change-management](../../resources/skills/git-recovery-and-change-management/SKILL.md) | engineering | Inspect Git history and changes, recover content and preserve existing work. Git historial recuperación cambios. |
| [git-worktree-coordination](../../resources/skills/git-worktree-coordination/SKILL.md) | engineering | Isolate concurrent repository changes with Git worktrees and evidence-based integration. |
| [go-development](../../resources/skills/go-development/SKILL.md) | engineering | Implement Go services and tools with bounded goroutine lifetimes, context propagation and explicit errors. |
| [google-docs](../../resources/skills/google-docs/SKILL.md) | office | Read and edit Google Docs while preserving tabs, structure, formatting and existing content. |
| [google-drive](../../resources/skills/google-drive/SKILL.md) | office | Find, organize, export and share files in Google Drive through an available authorized connection. |
| [google-sheets](../../resources/skills/google-sheets/SKILL.md) | office | Analyze and update Google Sheets with explicit ranges, typed values and formula preservation. |
| [google-slides](../../resources/skills/google-slides/SKILL.md) | office | Create or revise Google Slides decks using existing layouts, object IDs and the requested visual template. |
| [google-workspace-workflows](../../resources/skills/google-workspace-workflows/SKILL.md) | office | Coordinate cross-service Google Workspace workflows and manage Google Tasks, Contacts or Meet records through authorized native connections. |
| [grant-proposals](../../resources/skills/grant-proposals/SKILL.md) | research | Prepare funding proposals aligned with the actual call, evidence and a feasible budget. |
| [hiring-and-interviews](../../resources/skills/hiring-and-interviews/SKILL.md) | business | Prepare role criteria, structured interview guides and evidence-based candidate summaries. |
| [huggingface-hub](../../resources/skills/huggingface-hub/SKILL.md) | data | Work with Hugging Face models, datasets, paper pages, repository files and authorized Jobs using the installed CLI or supported APIs. Hub, datasets y trabajos de Hugging Face. |
| [identity-access-review](../../resources/skills/identity-access-review/SKILL.md) | security | Review account, role and service-identity permissions against actual resource needs. |
| [image-workflows](../../resources/skills/image-workflows/SKILL.md) | creative | Create, edit or prepare images using available generation and local processing tools with clear asset provenance. |
| [incident-response](../../resources/skills/incident-response/SKILL.md) | security | Triage an operational or security incident, preserve evidence and document recovery within authorized scope. |
| [infrastructure-as-code](../../resources/skills/infrastructure-as-code/SKILL.md) | operations | Review and apply infrastructure-as-code changes with state ownership and replacement awareness. |
| [inventory-planning](../../resources/skills/inventory-planning/SKILL.md) | business | Plan stock replenishment by SKU and location using inventory position, demand uncertainty, lead times and ordering constraints. Planificar existencias y reposición. |
| [invoice-and-expense-records](../../resources/skills/invoice-and-expense-records/SKILL.md) | office | Extract invoice and receipt fields, organize expense evidence and reconcile amounts without making payments. |
| [issue-tracker-workflows](../../resources/skills/issue-tracker-workflows/SKILL.md) | office | Read and update issues, boards and project trackers without duplicate tickets or false status changes. |
| [jvm-development](../../resources/skills/jvm-development/SKILL.md) | engineering | Implement Java or Kotlin changes across transactions, nullability, coroutines and JVM service boundaries. |
| [knowledge-base-maintenance](../../resources/skills/knowledge-base-maintenance/SKILL.md) | knowledge | Organize and retrieve local notes with provenance, stable links and explicit handling of duplicates or contradictions. |
| [knowledge-workspace-connectors](../../resources/skills/knowledge-workspace-connectors/SKILL.md) | office | Maintain Notion, Confluence or similar hosted knowledge pages while preserving native structure and access. |
| [kubernetes-operations](../../resources/skills/kubernetes-operations/SKILL.md) | operations | Inspect and change Kubernetes workloads with explicit context, rollout and recovery evidence. |
| [latex-authoring](../../resources/skills/latex-authoring/SKILL.md) | research | Create and repair LaTeX documents with reproducible builds and inspected PDF output. |
| [learning-and-tutoring](../../resources/skills/learning-and-tutoring/SKILL.md) | personal | Teach a topic with focused explanations, worked examples and feedback on demonstrated understanding. |
| [legacy-migration](../../resources/skills/legacy-migration/SKILL.md) | engineering | Migrate legacy code or frameworks incrementally while preserving observable behavior and rollback options. |
| [literature-review](../../resources/skills/literature-review/SKILL.md) | research | Find and compare scholarly papers, verify citations and synthesize methods, evidence and research gaps. |
| [llm-cost-optimization](../../resources/skills/llm-cost-optimization/SKILL.md) | engineering | Diagnose LLM API spending and reduce cost per successful task with measured usage, bounded retries, caching and quality-preserving routing. Optimizar costes de inferencia LLM. |
| [load-and-resilience-testing](../../resources/skills/load-and-resilience-testing/SKILL.md) | operations | Evaluate service capacity and failure recovery with representative workloads and controlled fault injection. |
| [local-file-work](../../resources/skills/local-file-work/SKILL.md) | operations | Inspect, create or edit local files and verify the requested result. |
| [local-troubleshooting](../../resources/skills/local-troubleshooting/SKILL.md) | operations | Diagnose local service, command or tool failures using files and managed commands, and verify authorized repairs. Diagnosticar un servidor que no arranca o un comando que falla. |
| [marketing-content](../../resources/skills/marketing-content/SKILL.md) | writing | Draft marketing pages, launch posts and campaigns grounded in real product capabilities and audience needs. |
| [mathematical-computation](../../resources/skills/mathematical-computation/SKILL.md) | research | Solve symbolic or numerical calculations with explicit domains, units and independent checks. |
| [mcp-integration](../../resources/skills/mcp-integration/SKILL.md) | engineering | Build or review MCP tool/resource integrations with precise schemas, lifecycle and authorization boundaries. |
| [meal-planning](../../resources/skills/meal-planning/SKILL.md) | personal | Plan ordinary meals and shopping quantities from preferences, pantry, time and dietary constraints. |
| [meeting-notes](../../resources/skills/meeting-notes/SKILL.md) | office | Turn meeting notes or transcripts into sourced decisions, action items and a useful agenda when requested. |
| [memory-and-reminders](../../resources/skills/memory-and-reminders/SKILL.md) | knowledge | Retrieve scoped memory, review candidates, maintain evidence-backed entities and relationships, and manage requested reminders. |
| [ml-dataset-preparation](../../resources/skills/ml-dataset-preparation/SKILL.md) | data | Prepare training and evaluation datasets with provenance, deduplication and split integrity. |
| [ml-demo-apps](../../resources/skills/ml-demo-apps/SKILL.md) | data | Build and verify Gradio machine-learning demos and Hugging Face Spaces, including LoRA and ZeroGPU deployment constraints. Demos Gradio y Spaces. |
| [ml-inference-deployment](../../resources/skills/ml-inference-deployment/SKILL.md) | data | Serve local or remote models with resource sizing, tokenizer compatibility and latency-quality checks. |
| [ml-training](../../resources/skills/ml-training/SKILL.md) | data | Train or fine-tune models with explicit resource use, reproducible configuration and held-out evaluation. |
| [mobile-app-development](../../resources/skills/mobile-app-development/SKILL.md) | engineering | Implement native or cross-platform mobile features with platform lifecycle and state restoration constraints. |
| [mobile-app-testing](../../resources/skills/mobile-app-testing/SKILL.md) | engineering | Verify mobile app flows on actual devices or emulators with lifecycle, permissions and input behavior. |
| [model-selection](../../resources/skills/model-selection/SKILL.md) | data | Compare deployable machine-learning models for a concrete task, hardware and license, including weight and KV-cache memory estimates. Elegir modelos y estimar VRAM. |
| [network-diagnostics](../../resources/skills/network-diagnostics/SKILL.md) | operations | Diagnose DNS, routing, TCP, TLS and proxy failures by separating network layers. |
| [notification-triage](../../resources/skills/notification-triage/SKILL.md) | operations | Consolidate noisy notifications into actionable alerts or digests with explicit ownership, deduplication and delivery evidence. Clasificar y reducir notificaciones duplicadas. |
| [observability-and-slos](../../resources/skills/observability-and-slos/SKILL.md) | operations | Instrument useful logs, metrics and traces and define service objectives tied to user-visible reliability. |
| [package-build-and-install](../../resources/skills/package-build-and-install/SKILL.md) | engineering | Build packages, wheels and indexes; verify a fresh consumer installation. Paquetes instalación distribución. |
| [parametric-cad](../../resources/skills/parametric-cad/SKILL.md) | engineering | Create, edit and inspect dimensioned 3D CAD parts or assemblies with editable parameters and verified STEP or mesh exports. Modelado CAD paramétrico y piezas 3D. |
| [patent-research](../../resources/skills/patent-research/SKILL.md) | research | Research patent publications, families and official record status with reproducible searches and precise identifiers. Investigar patentes y antecedentes técnicos. |
| [payment-and-billing](../../resources/skills/payment-and-billing/SKILL.md) | engineering | Implement payment and subscription workflows with verified events, idempotency and reconciled financial state. |
| [pdf-workflows](../../resources/skills/pdf-workflows/SKILL.md) | office | Read PDFs with CPU OCR; merge, split, reorder, rotate, crop, watermark, number, compress, fill text forms, render and redact with Fenix's own PDF helper. Publish PDFs in the chat preview panel. |
| [peer-review](../../resources/skills/peer-review/SKILL.md) | research | Assess a manuscript’s claims, methodology and evidence with constructive, bounded findings. |
| [performance-diagnosis](../../resources/skills/performance-diagnosis/SKILL.md) | engineering | Measure and diagnose latency, memory, CPU or throughput regressions before optimizing the relevant bottleneck. |
| [personal-organization](../../resources/skills/personal-organization/SKILL.md) | personal | Organize personal commitments into a realistic plan using existing tasks, dates and preferences. |
| [phishing-triage](../../resources/skills/phishing-triage/SKILL.md) | security | Assess suspicious messages, links or attachments without activating their content. |
| [pricing-analysis](../../resources/skills/pricing-analysis/SKILL.md) | business | Evaluate pricing models using customer value, unit economics and explicit usage scenarios. |
| [privacy-and-compliance](../../resources/skills/privacy-and-compliance/SKILL.md) | business | Map data practices and compliance evidence into concrete gaps without claiming certification. |
| [privacy-redaction](../../resources/skills/privacy-redaction/SKILL.md) | security | Prepare shareable copies by removing sensitive content while preserving usable evidence. |
| [process-and-service-management](../../resources/skills/process-and-service-management/SKILL.md) | operations | Start, inspect, test and stop background processes and persistent HTTP services. Procesos servicios puertos. |
| [process-improvement](../../resources/skills/process-improvement/SKILL.md) | business | Analyze repeated work and design a simpler process with measurable handoffs and exceptions. |
| [procurement-and-vendors](../../resources/skills/procurement-and-vendors/SKILL.md) | business | Compare suppliers and renewal decisions using requirements, total cost and contract evidence. |
| [product-discovery](../../resources/skills/product-discovery/SKILL.md) | business | Evaluate product opportunities and assumptions before committing to implementation. |
| [professional-writing](../../resources/skills/professional-writing/SKILL.md) | writing | Draft and revise reports, proposals, business messages and prose while preserving facts and the author’s intent. |
| [project-status-review](../../resources/skills/project-status-review/SKILL.md) | coordination | Summarize project progress and open commitments from task evidence without mixing contexts or creating duplicate work. |
| [purchase-comparison](../../resources/skills/purchase-comparison/SKILL.md) | personal | Compare purchase options by fit, total cost, compatibility and current evidence. |
| [python-development](../../resources/skills/python-development/SKILL.md) | engineering | Implement Python behavior with correct environments, resource lifetime, exceptions and async execution. |
| [rag-evaluation](../../resources/skills/rag-evaluation/SKILL.md) | data | Design and evaluate retrieval-augmented workflows against grounded questions and access boundaries. |
| [realtime-protocols](../../resources/skills/realtime-protocols/SKILL.md) | engineering | Implement WebSocket, SSE or streamed events with ordering, reconnect and cancellation semantics. |
| [release-and-ci](../../resources/skills/release-and-ci/SKILL.md) | engineering | Prepare releases and CI changes with reproducible checks, accurate versions and a verifiable artifact. |
| [repository-navigation](../../resources/skills/repository-navigation/SKILL.md) | engineering | Orient in unfamiliar repositories; discover entrypoints, instructions, build and test commands. Repositorio, estructura, pruebas. |
| [requirements-and-planning](../../resources/skills/requirements-and-planning/SKILL.md) | coordination | Clarify broad requests, detect ambiguous or conflicting requirements, and turn agreed outcomes into editable milestones without inventing requirements. |
| [research-evidence-audit](../../resources/skills/research-evidence-audit/SKILL.md) | research | Audit manuscript claims against cited passages, result files and experiment provenance, distinguishing source identity from actual support. Auditar afirmaciones, citas y evidencia científica. |
| [reverse-engineering](../../resources/skills/reverse-engineering/SKILL.md) | engineering | Analyze binaries, undocumented formats and authorized black-box interfaces from evidence. Ingeniería inversa y caja negra. |
| [review-feedback](../../resources/skills/review-feedback/SKILL.md) | engineering | Assess received code-review or PR comments and apply valid corrections when authorized, checking questionable suggestions against current behavior. Resolver comentarios del revisor. |
| [roadmap-prioritization](../../resources/skills/roadmap-prioritization/SKILL.md) | business | Prioritize initiatives using explicit outcomes, dependencies, effort and uncertainty. |
| [robot-description](../../resources/skills/robot-description/SKILL.md) | engineering | Author or validate URDF robot structure, SRDF planning semantics and SDF simulation descriptions with explicit frames, units and inertials. Robots, articulaciones y Gazebo. |
| [rust-development](../../resources/skills/rust-development/SKILL.md) | engineering | Implement Rust changes involving ownership, async cancellation, traits or unsafe boundaries. |
| [sales-opportunities](../../resources/skills/sales-opportunities/SKILL.md) | business | Prepare opportunity briefs, proposals and RFP responses grounded in actual capabilities and customer needs. |
| [scheduled-workflows](../../resources/skills/scheduled-workflows/SKILL.md) | operations | Configure recurring tasks with timezone, overlap, missed-run and duplicate-delivery behavior. |
| [scientific-figures](../../resources/skills/scientific-figures/SKILL.md) | research | Produce reproducible scientific plots with truthful scales, uncertainty and accessible labels. |
| [scientific-writing](../../resources/skills/scientific-writing/SKILL.md) | research | Draft scientific manuscripts from actual methods and results with traceable claims and reproducibility details. |
| [security-artifact-triage](../../resources/skills/security-artifact-triage/SKILL.md) | security | Inspect suspicious attachments and archives without activating payloads; preserve static evidence and parsing limits. Analizar archivos sospechosos y macros sin ejecutarlos. |
| [security-change-review](../../resources/skills/security-change-review/SKILL.md) | security | Audit a patch for changed trust boundaries, reachable callers and security regressions. |
| [security-code-review](../../resources/skills/security-code-review/SKILL.md) | security | Review application code for concrete authorization, injection, data exposure and isolation defects. |
| [security-finding-validation](../../resources/skills/security-finding-validation/SKILL.md) | security | Verify suspected vulnerabilities, challenge false positives and find reachable variants. |
| [security-fix-validation](../../resources/skills/security-fix-validation/SKILL.md) | security | Validate a security patch against the original failure, related variants and legitimate behavior. |
| [security-property-testing](../../resources/skills/security-property-testing/SKILL.md) | security | Build invariant and state-transition tests for security boundaries, parsers and access policies. |
| [security-rust-boundaries](../../resources/skills/security-rust-boundaries/SKILL.md) | security | Audit Rust unsafe and FFI boundaries, logical races, cancellation and externally reachable failure paths. |
| [security-static-analysis](../../resources/skills/security-static-analysis/SKILL.md) | security | Run existing local security analyzers and interpret SARIF or Semgrep output with explicit coverage gaps. |
| [seo-review](../../resources/skills/seo-review/SKILL.md) | writing | Audit website discoverability and content structure with evidence from pages, metadata and available search data. |
| [shell-automation](../../resources/skills/shell-automation/SKILL.md) | engineering | Write reliable shell automation with correct quoting, exit handling and explicit file effects. |
| [simulation-modeling](../../resources/skills/simulation-modeling/SKILL.md) | research | Build and validate simulations with explicit assumptions, random inputs and sensitivity analysis. |
| [skill-authoring](../../resources/skills/skill-authoring/SKILL.md) | knowledge | Create, improve or evaluate a Fénix skill using discoverable metadata and baseline comparisons, including pressure scenarios. Crear y evaluar skills frente a una línea base. |
| [skill-source-review](../../resources/skills/skill-source-review/SKILL.md) | security | Review a proposed skill, its scripts, dependencies and installers before adoption. Revisar skills y sus dependencias antes de incorporarlas. |
| [smart-home-control](../../resources/skills/smart-home-control/SKILL.md) | personal | Inspect or control connected lights, thermostats and home scenes using exact device identities and observed state changes. Domótica y dispositivos del hogar. |
| [software-implementation](../../resources/skills/software-implementation/SKILL.md) | engineering | Implement a new program, parser, interpreter or utility with a concrete runnable artifact. Crear programas y utilidades. |
| [sql-analysis-and-optimization](../../resources/skills/sql-analysis-and-optimization/SKILL.md) | engineering | Analyze SQL, preserve query semantics and measure database performance. SQL consultas equivalencia rendimiento. |
| [storage-and-archive-work](../../resources/skills/storage-and-archive-work/SKILL.md) | operations | Inspect storage use and safely extract, package or reorganize archives with content verification. |
| [structured-llm-output](../../resources/skills/structured-llm-output/SKILL.md) | data | Implement structured model outputs with schema validation and bounded repair of invalid responses. |
| [systematic-review](../../resources/skills/systematic-review/SKILL.md) | research | Conduct a protocol-based systematic literature review with reproducible search, screening records, exclusion reasons and optional meta-analysis. Revisión sistemática y PRISMA. |
| [task-delegation](../../resources/skills/task-delegation/SKILL.md) | coordination | Coordinate bounded subagent tasks with clear outcomes, isolated project context and evidence-based integration. |
| [team-messaging](../../resources/skills/team-messaging/SKILL.md) | office | Search or send team messages with correct channels, threads, recipients and duplicate-send handling. |
| [technical-documentation](../../resources/skills/technical-documentation/SKILL.md) | writing | Write developer guides, reference documentation and architecture decision records grounded in actual code. |
| [test-driven-repair](../../resources/skills/test-driven-repair/SKILL.md) | engineering | Diagnose root causes of software bugs, flaky tests and repeated failed fixes, then verify a focused repair. Depurar causa raíz y pruebas intermitentes. |
| [test-strategy](../../resources/skills/test-strategy/SKILL.md) | engineering | Select meaningful unit, integration and end-to-end checks from behavior and failure risk. |
| [text-processing-and-parsing](../../resources/skills/text-processing-and-parsing/SKILL.md) | engineering | Implement and test parsers, regular expressions and text transformations. Regex fechas registros texto. |
| [threat-modeling](../../resources/skills/threat-modeling/SKILL.md) | security | Map assets, trust boundaries and misuse scenarios into prioritized security controls and open questions. |
| [time-series-forecasting](../../resources/skills/time-series-forecasting/SKILL.md) | data | Forecast time-dependent data with leakage-free backtesting and explicit uncertainty. |
| [translation-and-localization](../../resources/skills/translation-and-localization/SKILL.md) | writing | Translate and localize content while preserving meaning, terminology, placeholders and formatting. |
| [travel-planning](../../resources/skills/travel-planning/SKILL.md) | personal | Build a feasible travel itinerary from current options, user constraints and explicit booking status. |
| [typescript-development](../../resources/skills/typescript-development/SKILL.md) | engineering | Implement TypeScript changes with sound runtime boundaries, discriminated states and compatible module exports. |
| [user-research](../../resources/skills/user-research/SKILL.md) | business | Plan and synthesize customer interviews or usability research without leading questions or invented personas. |
| [video-editing](../../resources/skills/video-editing/SKILL.md) | creative | Edit or transcode video with explicit time ranges, stream mapping and verified audio/video output. |
| [vulnerability-management](../../resources/skills/vulnerability-management/SKILL.md) | security | Triage vulnerability findings into evidence-backed remediation, ownership, deadlines and verified closure. Priorizar CVE con KEV, EPSS y exposición; gestionar excepciones. |
| [wearable-data-analysis](../../resources/skills/wearable-data-analysis/SKILL.md) | personal | Summarize supplied or connected wearable sleep, activity and training records with unit, session-identity and missing-day checks. Análisis de datos de actividad y sueño. |
| [weather-briefing](../../resources/skills/weather-briefing/SKILL.md) | personal | Retrieve and explain current weather, hourly forecasts and official alerts for a resolved location and time window. Pronóstico meteorológico y condiciones locales. |
| [web-animation](../../resources/skills/web-animation/SKILL.md) | engineering | Implement or repair web motion with scoped timelines, scroll triggers, lifecycle cleanup and reduced-motion behavior, including GSAP. Animaciones web, parallax y ScrollTrigger. |
| [web-data-extraction](../../resources/skills/web-data-extraction/SKILL.md) | data | Extract structured records from specified web pages with pagination, schema validation and row-level provenance. Extracción web a CSV o JSON. |
| [web-presentations](../../resources/skills/web-presentations/SKILL.md) | creative | Build browser-based slide presentations with readable layouts, keyboard navigation and export boundaries. |
| [web-research](../../resources/skills/web-research/SKILL.md) | research | Search the internet and investigate recent news, Hacker News discussions, user experiences and comparisons with original sources and Fenix's own search engine. |
| [youtube-summary](../../resources/skills/youtube-summary/SKILL.md) | knowledge | Summarize a YouTube video from its URL using available captions, with timestamp links and explicit coverage limits. Resumir videos de YouTube mediante subtítulos. |

## Product boundaries relevant to positioning / Límites para describir el producto

- The default hosted sandbox policy is gVisor, not a dedicated hardware VM per customer; optional Kata requires suitable infrastructure. Local mode has different boundaries.
- Office editing is not a complete replacement for every Word/Excel/PowerPoint feature; formula calculation and exact imported layouts are not provided by the native engine.
- Meet reads accessible existing transcripts; it does not automatically join and record every Meet/Zoom/Teams call. YouTube summarization depends on available captions.
- Skills and optional integrations cannot be advertised as active account connections without configuration and observed access. Jev is optional and does not establish measured whole-task savings.
- Private learned procedures are not foundation-model training; recovery does not undo arbitrary external effects.
- A subscription proposition is commercially coherent, but subscription revenue, retention and margin are not inferable from this source inventory.

El espacio de trabajo por usuario, las tareas recurrentes, la memoria y las conexiones sostienen la propuesta de suscripción. Su valor defendible tendría que demostrarse mediante resultados repetidos para un segmento concreto. Las plantillas, cantidad de skills o elección de Rust no bastan como ventaja competitiva por sí solas. Costes de inferencia, ejecución, almacenamiento y soporte deben relacionarse con trabajo cobrado; no se estimó un margen sin datos.
