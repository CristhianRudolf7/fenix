---
name: completion-verification
description: Check whether delivered artifacts or delegated work actually meet requirements before accepting a completion claim, especially with missing or stale evidence. Verificar entregas y finalización.
metadata:
  version: 1.1.0
  category: coordination
  tags: completion, verification, delivery, evidence, requirements
  aliases: verify delivery, marked complete, worker done, stale tests, entrega, finalización, ya terminó, cumplió lo pedido, documentos entregados, specification gaps, incumplimientos, requisitos pendientes
  required_tools: task_inspect,file_read,command_status,tool_search
  requires: Requested outcome and access to the delivered artifacts and relevant evidence
  effects: mixed
---
# Completion Verification

## Scope

Verify a claimed delivery against the owner's requirements, including code, documents, data and worker handoffs. Use when checks are missing, stale, contradictory or span several artifacts. Routine straightforward work can use its own skill's verification; this is not a mandatory extra stage for every response.

## Inputs

Original request and accepted changes, artifact paths or identities, current revision and existing checks with their scope. For a worker, obtain its delivered files and actual tool results, not only its final summary.

## Workflow

1. Recover the authoritative requirements. Map each material outcome to a current artifact and the observation needed to establish it. Preserve explicit exclusions and separate assumptions from requirements.
2. Read existing checks and compare their code, input and environment with the delivered state. Tests that passed before an affecting final edit are stale for that behavior. Reuse evidence for unchanged parts; rerun only affected or required checks.
3. Discover the appropriate consumer tools within the task's permissions. Inspect documents with their document skill, data with a relevant analysis skill and software through its actual entrypoint. A file existing, build succeeding or port listening does not prove the requested behavior.
4. Check foreground command completion and recover omitted output where needed. Name failed, skipped or unavailable checks. For delegated work, inspect the artifact and acceptance evidence; a worker saying it is done is insufficient.
5. Report each material requirement as verified, failed or not verified, with its artifact/check and limitation. Distinguish prepared, integrated, uploaded and deployed states. Address authorized remaining work or identify its concrete blocker; do not mark the whole task complete while required outcomes remain unverified.

## Verification

The verdict follows observed evidence for the current delivery. A narrow check cannot justify an entire suite passing. State which requirements remain unverified and why; an independently verified artifact can satisfy a requirement even when the worker's report is incomplete.

When a software delivery has multiple acceptance criteria or a mismatch with its specification, load [requirement reconciliation](references/requirement-reconciliation.md). Classify observed gaps separately from unavailable evidence, and attach bounded corrective work to the existing plan. A successful audit cannot establish post-launch business outcomes that have not been measured.

## Limits

Checking delivery does not authorize publishing, sending messages, merging, deletion or an unsafe rerun of external effects. Tests and consumer checks can themselves write files or trigger actions; inspect their side effects before running them. Do not introduce new approval stages, test suites or reviewers merely to fill a checklist.

## Español

Comprueba si lo entregado cumple todo lo pedido mediante requisitos originales, archivos actuales y resultados observados. Si el agente dice que ya terminó, verifica el entregable. Si las pruebas pasaron antes del último cambio, repite lo afectado. Distingue existencia, compilación, funcionamiento, integración y publicación. Informa requisitos verificados, fallidos o sin verificar, conservando las limitaciones. Aplica la comprobación propia de cada documento o sistema sin repetir efectos externos ni añadir aprobaciones rutinarias.

Para entregas de software con varios criterios o discrepancias, consulta la referencia de conciliación. Separa incumplimientos observados de evidencia no disponible; registra correcciones acotadas en el plan vigente. Una auditoría técnica no demuestra resultados comerciales aún no medidos.
