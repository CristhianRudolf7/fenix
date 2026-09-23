# Root-cause investigation / Investigación de causa raíz

Use for an unexplained failure, a flaky reproduction or a repair that did not work.

1. Record the exact input, command, working directory, relevant version and observed failure. Separate setup failure from the behavior under investigation. Check recent changes and compare the same input in a working path or environment.
2. Trace the first incorrect value or state backward to its producer. Across boundaries, inspect sanitized input/output shape, ordering and configuration presence. Do not dump environment variables, credentials or complete private payloads.
3. State a falsifiable hypothesis: which cause predicts which observation? Choose the smallest diagnostic that distinguishes it from an alternative. Change one explanatory factor; record the result before another edit.
4. For timing failures, wait for the event or state the assertion needs with a bounded deadline and diagnostic output. Increasing a sleep hides ordering defects. Preserve deliberate timing tests such as debounce intervals. For suite-only failures, narrow the predecessor tests and shared state using a controlled order and isolated data.
5. Preserve a regression case that fails on the defect and passes after the focused fix. Check affected consumers. Distinguish a mitigated symptom from an established cause when external systems cannot be reproduced.

If attempted fixes keep exposing different coupled failures, reassess the interface or architecture before another speculative patch. Preserve useful changes and the hypothesis log in existing task evidence. Escalate only the unresolved decision or missing evidence; do not invent an arbitrary retry allowance or override the runtime's limits.

For a test added after an implementation already exists, establish its sensitivity in a disposable copy or with a narrowly controlled mutation. Restore that copy and rerun the check. Do not delete valid implementation or alter the user's unrelated work to recreate a test-first sequence.

## Español

Registra entrada, comando, entorno y fallo exactos. Distingue preparación defectuosa de error del producto. Sigue el primer dato incorrecto hasta su origen y compara con un caso válido sin exponer secretos. Formula una hipótesis comprobable y cambia un factor cada vez. En fallos intermitentes espera una condición con plazo; en fallos de conjunto investiga orden y estado compartido. Conserva una regresión que detecte el defecto y pase tras corregirlo. Si ya existe la implementación, comprueba la sensibilidad en una copia aislada. Revisa el diseño cuando las correcciones revelen acoplamientos; no borres trabajo válido ni reinicies límites de ejecución.
