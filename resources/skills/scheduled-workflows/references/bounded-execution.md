# Bounded automated runs

Use when a scheduled task itself contains repeated attempts or an optimization/repair loop. Scheduling describes when to start; it does not supply a safe termination policy.

Record the authorized objective, unchanged acceptance criteria, meaningful progress signal, maximum attempts/time/resource budget, cancellation path and terminal outcomes. Use the owning runtime's counters and persisted run identity; a prose limit is not enforcement. A timeout or incomplete check cannot count as success. Keep user-requested completion criteria separate from optional improvements.

Before retrying, classify the failure and inspect whether the preceding effect committed. A changed observation or corrected cause can justify another attempt; rewording the same request without new evidence is not progress. Preserve failed attempts, partial results and undelivered outcomes. Cancellation can leave external effects in place and must be reconciled.

Test stopping and recovery with synthetic failure, unchanged failure, exhausted budget, cancellation and lost acknowledgement after a committed write. The task cannot improve its score by disabling checks, excluding failed cases or weakening the acceptance condition. If the acceptance criteria are themselves wrong, revise them explicitly as a requirement change and retain the earlier evidence.

Use existing workflow and execution limits; do not add a competing queue, observer daemon or global learning hook from a skill. A recurring plan is not evidence that it has been installed or run successfully.

## Español

Una programación no define por sí sola cuándo debe detenerse el trabajo. Fija resultados, límites y cancelación en el runtime existente, conserva intentos fallidos y reconcilia efectos inciertos. No relajes pruebas para hacer que un bucle parezca exitoso.
