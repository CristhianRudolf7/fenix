# Skill routing and overlap / Selección y solapamientos

Load the narrowest workflow that matches the requested outcome. Add a second skill only for a separate decision or artifact. These are selection examples, not mandatory chains.

Carga el procedimiento más específico para el resultado solicitado. Añade otro solo si resuelve una decisión o entregable distinto; estas combinaciones no son cadenas obligatorias.

| Requested outcome / Resultado | Primary skill | Optional specialist / Especialista |
| --- | --- | --- |
| Implement behavior / Implementar función | software-implementation | rust-development, python-development, typescript-development or the actual language |
| Repair observed bug / Corregir fallo | test-driven-repair | relevant language or subsystem skill |
| Choose boundaries / Diseñar arquitectura | architecture-design | database-schema-design, background-jobs, realtime-protocols |
| Change DB schema / Migrar esquema | database-migrations | backup-and-restore; SQL tuning only if a measured query bottleneck exists |
| Review a diff / Revisar cambios | code-review | security-code-review when the diff affects a concrete trust boundary |
| Handle received review comments / Atender comentarios recibidos | review-feedback | test-driven-repair for an authorized correction whose cause remains unclear |
| Audit a claimed delivery / Comprobar una entrega | completion-verification | the document, data or software skill needed to inspect that artifact |
| Diagnose agent decisions or repeated effects / Diagnosticar decisiones o efectos repetidos del agente | agent-workflow-diagnosis | local-troubleshooting only when the investigation identifies an authorized local repair |
| Decide test coverage / Elegir comprobaciones | test-strategy | browser-regression-testing, mobile-app-testing, load-and-resilience-testing |
| Inspect a site / Navegar una web | browser-navigation | no regression harness unless repeatable checks are requested |
| Answer data question / Analizar datos | data-analysis | scientific-figures for a publication artifact |
| Audit dataset / Auditar calidad | data-quality-audit | ml-dataset-preparation for train/evaluation split integrity |
| Compare randomized groups / Experimento A/B | ab-test-analysis | analytics-instrumentation if event collection is also being changed |
| Forecast future / Predecir serie | time-series-forecasting | no random train/test split across time |
| Study literature / Revisar papers | literature-review | bibliography-management for citation-file maintenance |
| Draft actual research / Redactar investigación | scientific-writing | latex-authoring for typesetting, peer-review for critique |
| Local Word/Excel/PPTX / Archivos locales | documents-word, documents-spreadsheets, documents-presentations | pdf-workflows for explicit PDF work; no implied formula calculation or rendering |
| Hosted Google document / Documento Google | google-docs, google-sheets, google-slides | google-drive for file discovery and sharing |
| Move cloud files / Transferir archivos | cloud-file-sync | google-drive if that is the connected provider |
| Compare versions / Comparar documentos | document-comparison | contract-review for legal obligations, not ordinary text differences |
| Reminder / Recordatorio | memory-and-reminders | scheduled-workflows for recurring jobs with overlap policy |
| Personal priorities / Agenda personal | personal-organization | calendar-scheduling for authorized calendar changes |
| Product priorities / Prioridades producto | roadmap-prioritization | project-status-review for execution evidence |
| Draft message / Redactar mensaje | professional-writing | email-workflows or team-messaging only when accessing that channel is needed |
| Billing code / Programar cobros | payment-and-billing | financial-modeling is for projections, invoice-and-expense-records for bookkeeping evidence |
| Hosted knowledge / Wiki remota | knowledge-workspace-connectors | knowledge-base-maintenance for local notes; connection is a prerequisite |

Do not load all options in a row. Search metadata with the user’s terms; load the selected body and only the referenced resource necessary for the task. If no connector exists, draft or process a supplied export where useful and label the result accurately.

No cargues todas las opciones de una fila. Busca mediante metadatos y abre solo el cuerpo y los recursos necesarios. Sin conexión disponible, prepara un borrador o procesa una exportación proporcionada, indicando lo que realmente se hizo.
