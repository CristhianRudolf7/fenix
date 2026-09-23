# Agent workflow diagnosis / Diagnóstico del trabajo del agente

Use for a specific complaint about repeated work, a missed skill, ignored requirements, unsupported completion, elapsed time or token usage.

Resolve the task, project and relevant interval from the request and existing evidence. Ask only for scope that remains missing. Start with task_inspect and history_search; retrieve bounded pages and the exact tool results needed. A large transcript or one huge event should be narrowed before reading. Preserve original events and distinguish the owner, coordinator, worker and tool as separate sources of statements.

Build the relevant sequence: original requirement and changes, skill candidates/load events if recorded, plan/revisions, commands, approval or clarification waits, worker reports and final checks. Cite task/event/checkpoint references for every material finding. Current installed skill text is not proof of which version a past task loaded. Missing load events mean unknown coverage unless the retained trace is known to be complete.

Distinguish these causes before proposing a correction:

| Evidence | Candidate explanation |
| --- | --- |
| Applicable skill absent from search results | Discovery wording, enabled state or indexing |
| Skill loaded but required step absent in a complete trace | Instruction application or conflicting context |
| Tool rejected the call or returned executed=false | Runtime, schema or permission boundary; no completed effect |
| Successful effect repeated after compaction | Recovery or provenance failure; inspect checkpoint and task contract |
| Check failed but final report claims success | Evidence interpretation or reporting failure |
| Time spent awaiting an answer or external command | Waiting, not necessarily model reasoning or redundant work |

Count repeated completed effects separately from legitimate reads, retries with new evidence and status checks that observed progress. Sum token usage only where provider usage is present, report missing records and avoid double-counting parent/worker summaries. Overlapping worker durations do not add up to wall-clock latency. Estimate cost only with an attributable model, price and date; otherwise report usage and leave cost unknown.

Propose the smallest correction supported by the trace and an independent reproduction. A skill rewrite cannot repair a deterministic engine defect by itself. Apply changes only within the requested repair scope; diagnosis alone stays read-only. No automatic transcript export, issue posting, retrospective sweep or mandatory analyst agents.

## Español

Delimita tarea, proyecto e intervalo y reconstruye requisitos, skills, herramientas, esperas y comprobaciones con referencias a eventos. Distingue descubrimiento fallido, instrucciones omitidas, rechazo del runtime y afirmaciones sin evidencia. La versión instalada hoy no prueba cuál se usó antes. Cuenta efectos repetidos por separado de lecturas y reintentos justificados; no sumes tiempos concurrentes como latencia ni inventes costes cuando faltan datos. Propón una corrección verificable sin exportar trazas ni modificar el agente si solo se pidió diagnóstico.
