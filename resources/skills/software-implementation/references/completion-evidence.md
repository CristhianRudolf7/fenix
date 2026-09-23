# Completion evidence / Evidencia de finalización

Load when delivery spans several requirements, worker reports or different verification signals.

Use the existing task contract, plan and tool events as the record. For each material claim, identify the artifact or revision, command or inspection, observed result and coverage limit. A separate report file is useful only when the task needs a durable handoff.

| Claim | Evidence to inspect | What it does not establish |
| --- | --- | --- |
| The artifact exists | Read the exact output path and relevant contents | Its behavior or installation |
| The build passes | Completed build command and exit status | Runtime correctness |
| A defect is repaired | Original reproduction passes; regression detects the defect | All unrelated behavior |
| Tests pass | Completed named test command, result and scope | Tests that were not run |
| A service works | Readiness plus an actual request through its consumer | All requests or deployment health |
| A worker finished | Changed artifact, requirement coverage and attributable checks | Completion inferred from the worker's wording |
| Work is integrated | Combined revision and relevant integration checks | Remote publication or deployment |

Evidence remains usable while the relevant code, inputs and environment are unchanged. After an affecting edit, rerun the affected checks; do not rerun an expensive unchanged suite merely to make the evidence appear in the latest message. When shared files can change concurrently, compare their state with the checked revision before using earlier results.

Read the command's final status, not only its last friendly log line. Recover omitted output through command_output or task_inspect when it matters. Record pre-existing failures separately; a partial pass cannot be described as a clean full suite. Test results and requirement coverage are separate questions.

Finish with the delivered behavior, observed checks, unresolved items and actual integration state. Existing authorization remains valid; this checklist is not a new permission gate, a required reviewer, or permission to merge, publish or delete.

## Español

Asocia cada afirmación con el archivo o revisión, la comprobación, su resultado y límites. Existencia, compilación, pruebas, disponibilidad del servicio e integración son estados distintos. Reutiliza evidencia válida mientras código, entradas y entorno relevantes no cambien; repite lo afectado por nuevas ediciones. Recupera salidas truncadas y separa fallos previos. Un resumen de un subagente no demuestra el resultado. Informa comportamiento entregado, comprobaciones, pendientes y estado real sin crear aprobaciones adicionales.
