# Biblioteca de skills de Fénix

[English](../en/skill-library.md)

[Comparación detallada con Superpowers y mejoras de procedimientos](superpowers-comparison.md).

La biblioteca contiene **187 skills propias**, con cuatro procedimientos nuevos de seguridad y trece entradas mejoradas en la [revisión específica de ciberseguridad](cybersecurity-skills-review.md). Las instrucciones se cargan bajo demanda.

[Ampliación basada en cuatro repositorios y pruebas ejecutables](repository-skills-comparison.md).

[Revisión de gstack: tres skills propias y ocho mejoras](gstack-review.md).

## Estructura común

```text
resources/skills/
  README.md
  README.es.md
  <capability>/
    SKILL.md
    references/    # optional, loaded when needed
    scripts/       # optional, reviewed executable helpers
    assets/        # optional templates/data
```

Cada entrada usa `name`, `description` y `metadata` con valores de texto: `version`, `category`, `tags`, `aliases`, `required_tools`, `requires`, `effects`. El cuerpo canónico define alcance, entradas, procedimiento, verificación y límites en inglés. La traducción completa al español se guarda por separado en `SKILL.es.md` para la interfaz; el archivo canónico no incluye un resumen en español. Los alias en ambos idiomas facilitan búsqueda léxica. [Plantilla](../../resources/skills/skill-authoring/references/template.md).

`required_tools` contiene herramientas nativas existentes; el motor las descubre con los permisos de la tarea. `requires` describe dependencias externas. `effects` documenta efectos esperados; no aplica permisos ni instala conexiones. La activación/desactivación se conserva en el estado del motor, no en una segunda bandera contradictoria del YAML.

## Conexiones y límites

Google Drive, Docs, Sheets, Slides, correo y calendario necesitan una conexión autorizada o la alternativa documentada. Crear sus skills no añade endpoints, OAuth ni conectores. PDF requiere un lector/conversor/OCR disponible. Word, Excel y PowerPoint locales usan Fénix Documents, sin Microsoft 365 ni OfficeCLI. El motor no calcula fórmulas ni renderiza los documentos; las skills conservan esas limitaciones.


## Cobertura

| Category | Skills |
| --- | ---: |
| business | 16 |
| coordination | 5 |
| creative | 8 |
| data | 17 |
| engineering | 48 |
| knowledge | 6 |
| office | 19 |
| operations | 20 |
| personal | 8 |
| research | 15 |
| security | 19 |
| writing | 5 |

## Catálogo

| Skill | Category | Purpose |
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
| [data-pipelines](../../resources/skills/data-pipelines/SKILL.md) | data | Build ETL or ELT pipelines with stable keys, incremental state and recoverable data flow. |
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
| [google-workspace-workflows](../../resources/skills/google-workspace-workflows/SKILL.md) | office | Coordinate Google Workspace workflows, Tasks, Contacts and Meet records through native connections. |
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
| [requirements-and-planning](../../resources/skills/requirements-and-planning/SKILL.md) | coordination | Clarify broad requests and turn agreed outcomes into editable milestones without inventing requirements. |
| [research-evidence-audit](../../resources/skills/research-evidence-audit/SKILL.md) | research | Audit manuscript claims against cited passages, result files and experiment provenance, distinguishing source identity from actual support. Auditar afirmaciones, citas y evidencia científica. |
| [reverse-engineering](../../resources/skills/reverse-engineering/SKILL.md) | engineering | Explain compiled binaries or recover undocumented formats from supplied artifacts, using static evidence and scoped dynamic analysis. Ingeniería inversa y comparación binaria. |
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

## Selección y límites

Se compararon 66 repositorios, 7.216 nombres candidatos agrupados y secciones de 201 rutas seleccionadas. Se descargaron 536 entradas para la ampliación; descargarlas no equivale a leerlas íntegramente. [Comparación por repositorio](skill-source-review.md) y [evidencia de fuentes](../skills/source-review.json).

No se copiaron cuerpos, scripts ni activos de las fuentes. Las skills de conexiones requieren herramientas autorizadas disponibles; crear instrucciones no añade OAuth, proveedores ni integraciones. Multimedia, entrenamiento ML, cloud y tareas especializadas necesitan los recursos declarados. Fénix Documents conserva sus límites: no evalúa fórmulas ni renderiza documentos.

La [guía de selección](../../resources/skills/skill-authoring/references/routing-and-boundaries.md) distingue procedimientos solapados. No obliga a cargar varias skills por tarea. Las descripciones y alias se indexan localmente; la búsqueda devuelve hasta ocho resultados, la orientación automática hasta dos resúmenes, y las instrucciones completas solo se cargan cuando hacen falta.

## Incorporación y actualización

Cargo empaqueta automáticamente cada carpeta con `SKILL.md` y sus recursos. Al arrancar una compilación actualizada, el motor instala las nuevas entradas en el directorio configurado de cada usuario. Preserva personalizaciones y activación; solo reemplaza contenido antiguo que coincide exactamente con hashes conocidos. Registra el hash distribuido antes de revisar una skill. Las referencias existentes no se sobrescriben. Esta ampliación no reinicia ni despliega el servicio público.

## Validación

```bash
cargo test -p fenix-agent --lib skills:: --locked
cargo test -p fenix-agent --test runtime skill_installation --locked
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests/skills -p 'test_*.py' -v
```

La validación usa las 187 skills reales: metadatos, herramientas existentes, contenidos distintos, instalación, preservación de personalizaciones, rutas confinadas y recursos bajo demanda. Incluye el conjunto original de consultas en inglés y español, 18 casos de Superpowers, 24 de separación de procedimientos, 65 de cuatro repositorios, 54 de OpenClaw y 62 de diez repositorios. Son regresiones de desarrollo, no un benchmark independiente del modelo. La revisión histórica de diez repositorios superó 26 pruebas nativas y 24 pruebas Python. Las comprobaciones actuales y 39 consultas adicionales están en la [revisión de ciberseguridad](cybersecurity-skills-review.md). Las revisiones anteriores registraron por separado comprobaciones de medios, animación y lecturas públicas del Hub; esas mediciones históricas no son evidencia de una nueva ejecución. No se habilitaron ni probaron integraciones externas autenticadas. No se generaron skills sintéticas de relleno.

Las incorporaciones actuales y los límites de validación constan en [la revisión de diez repositorios](ten-repository-skills-review.md), incluida la utilidad de las integraciones Composio. Las comprobaciones históricas se conservan en [la revisión de OpenClaw](openclaw-skills-review.md), [el informe de separación](workflow-skill-boundaries.md) y [la revisión de cuatro repositorios](repository-skills-comparison.md), incluido su fallo previo de recuperación.

### Comprobaciones del catálogo inicial de 142 skills — 2026-09-19

- 142/142 skills pasaron el validador de formato.
- 62/62 consultas de regresión encontraron el resultado esperado entre los tres primeros.
- Pasaron las 11 pruebas de biblioteca de skills.
- Pasó la prueba de integración de instalación, personalizaciones y activación.
- Pasaron Clippy del agente (biblioteca y tests), formato Rust y comprobación de enlaces/documentación.

## Especialista de seguridad

Los procedimientos de auditoría comparten el catálogo general y están disponibles para el maestro y los subagentes normales. Se añaden seis procedimientos propios para cambios, hallazgos, análisis estático, Rust, propiedades y correcciones. Consulta [el perfil, el catálogo y las evidencias](security-specialist.md).

La revisión de gstack añade 26 casos de descubrimiento y comprobaciones de actualización; consulta los [resultados actuales](gstack-review.md).

[Resúmenes de YouTube y extracción de subtítulos](youtube-summary.md).

[Comparación con ECC: cinco skills nuevas y trece mejoras](ecc-review.md).

## Edición privada desde la interfaz web

Skills muestra el catálogo privado de la cuenta. Al pulsar una tarjeta se abren sus instrucciones guardadas; **Editar** muestra el Markdown original completo en inglés como texto plano. Los cambios se guardan únicamente al pulsar el icono de Guardar cambios, comprobando el hash para no sobrescribir ediciones de otra pestaña. Al cerrar con cambios pendientes se pregunta si se desea seguir editando o descartarlos. Las acciones de editar y eliminar aparecen en barras de iconos separadas del Markdown. El proceso existente de traducción prepara el Markdown en español para la nueva versión. Si falla el guardado, el texto permanece en el editor para corregirlo o copiarlo.

**Eliminar** pide confirmación y retira el skill y sus archivos auxiliares de la cuenta. Un registro privado evita que los skills incluidos o la migración antigua lo vuelvan a crear al reiniciar. Las publicaciones y las copias de otros usuarios no cambian. La API privada mantiene el aislamiento por cuenta y las comprobaciones de origen, acepta solo archivos contenidos en la carpeta de skills y rechaza documentos vacíos, excesivos o con cabecera mal formada.

La edición manual abre el Markdown guardado en el idioma de la interfaz. Al guardar explícitamente, el modelo rápido traduce el otro idioma antes de publicar ambas versiones. El agente sigue usando inglés. Un fallo de traducción conserva el borrador y los documentos anteriores; los cambios concurrentes requieren reabrir el editor.
