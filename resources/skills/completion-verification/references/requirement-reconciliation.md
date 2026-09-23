# Requirement reconciliation / Conciliación de requisitos

Use when checking a software delivery against several acceptance criteria or investigating a specification mismatch. Keep the existing audit and plan authoritative. This procedure guides evidence gathering; it is not a deterministic proof of correctness.

## Compare current intent with current behavior

Read the current request/amendments and, when relevant, the project contract through `project_spec`. Resolve relevant pagination and record the source revision. Examine the delivered artifact and real consumer behavior against the acceptance observations; a diff, generated summary or checked task box is insufficient by itself.

For each material discrepancy, record the requirement/source, affected artifact/revision, observed result or missing observation, consequence and smallest next step. Distinguish:

| Finding | Meaning | Next step |
| --- | --- | --- |
| Missing | Required behavior is demonstrably absent | Add the smallest implementation task and acceptance check |
| Partial | An observed path works but a required path does not | Repair the uncovered path and its affected dependents |
| Contradictory | Observed behavior violates a current requirement | Correct the behavior; amend intent only with a valid user/source basis |
| Outside scope | Newly introduced behavior has no requirement or justified enabling role | Inspect its effects; do not delete unrelated or user-owned work automatically |
| Not verified | Available evidence cannot establish the requirement | Obtain the observation, or report the concrete access/environment blocker |

Do not call uninspected code missing, absent tests a proven bug, or a proposed design an accepted requirement. A future business metric remains unmeasured even when the measurement instrumentation works. Separate technical delivery status from that later evaluation, preserving any business commitment and its observation plan.

## Turn confirmed gaps into bounded work

First inspect existing milestones, findings and evidence for the same requirement and artifact. Reuse an open correction rather than append a duplicate. In the native plan, preserve valid IDs/history and update only affected work with current version checks. Each correction needs the source reference, observed discrepancy, affected artifact, dependency and acceptance observation; auxiliary tasks may cite the requirement they enable.

Recheck affected behavior after the correction and record evidence for the current revision. Keep failed attempts as history, not current proof. Stop when required deliverables are verified, or state exactly what remains blocked or unverified. If the same discrepancy persists without new evidence, diagnose the cause or request the missing decision instead of generating another identical task. A read-only review records recommendations; it does not acquire implementation authority. Existing implementation authorization permits bounded repairs without another routine approval round.

## Español

Contrasta requisitos vigentes y artefactos actuales mediante observaciones reales. Clasifica lo demostrado como ausente, parcial, contradictorio o añadido sin justificación; separa lo no verificado por falta de evidencia. No conviertas falta de inspección en un defecto ni métricas futuras en resultados demostrados.

Cada pendiente debe indicar requisito y revisión, artefacto, observación, consecuencia y siguiente paso. Busca correcciones existentes antes de crear otras, conserva historial e IDs válidos y modifica solo trabajo afectado con la versión vigente. Repite las comprobaciones pertinentes tras corregir. Si no aparece evidencia nueva, investiga el bloqueo en lugar de repetir tareas. Una revisión de solo lectura no autoriza cambios; una implementación ya autorizada no necesita aprobaciones rutinarias adicionales.
