---
name: software-implementation
description: Implement a new program, parser, interpreter or utility with a concrete runnable artifact. Crear programas
  y utilidades.
metadata:
  version: 1.10.0
  category: engineering
  tags: software, implementation
  aliases: programación, implementar, implementa, programa, software, parser, utilidad, command-line program, CLI
  required_tools: file_read,file_write,file_edit,command_run
  requires: Available task-authorized Fénix tools
  effects: mixed
---
# Software Implementation

## Scope

Implement a new program, parser, interpreter or utility with a concrete runnable artifact. Crear programas y utilidades.

## Inputs

Requested artifact, interface, language and constraints.

## Workflow

### Establish the interface

Identify the requested artifact, entry command, inputs, outputs and constraints from the original request and existing callers. Keep exact paths and invocation syntax. If a summary appears inconsistent, retrieve the original before asking about the apparent conflict. Ask about unresolved requirements; an implementation choice with several valid solutions is not itself a missing requirement.

When implementing from a specification or multi-step plan, carry its existing requirement IDs/source references and acceptance examples into the affected tasks and checks. Compare the current contract with the plan before relying on either; use requirements-and-planning for material contradictions. Preserve specified validation limits, error semantics and compatibility at both producer and consumer boundaries. Do not silently rewrite a requirement to match convenient implementation behavior.

### Build and check a small working slice

When navigating an existing codebase with a configured language server, locate candidate files first, then discover `code_query`. Prefer `symbol_outline` or `find_symbol` before `read_symbol` to load only the relevant function/class. Resolve ambiguity explicitly; do not guess an overload. Use `references` with `include_context` when impact is unclear. Read enough surrounding code before editing with the existing file tools, and never use a truncated symbol body as complete patch context. Small, obvious edits can use file search/read directly; an unavailable LSP is not a reason to install a new service or block the task.

Use tools already exposed directly; discovery is for missing capabilities. Once the relevant files are understood, make a coherent multi-file patch instead of spending a model turn on each file. Keep edits and their verification ordered; do not delete an existing file in one turn merely to recreate it in the next.

Create the entry artifact and implement one valid input-to-output path early. Run it through the actual caller or command, then extend behavior. A plan or explanation does not deliver a requested program. For parsers and interpreters, keep parsing, representation and evaluation boundaries explicit; test a small valid case, composition and a malformed input consistent with the requested interface. For utilities, check exit status and the generated output through its consumer.

When a test fails, isolate the smallest observed discrepancy and change that path. Reuse successful work. Before completing, check the required artifact exists and run representative behavior checks under the requested invocation. Report remaining failures and checks that could not run. Do not substitute a different language, output format or command when these were specified.

Use a language-specific skill when the task hinges on ownership, concurrency, runtime types or platform semantics. Discover and load only the relevant one; a small language-neutral edit does not need every engineering skill. Use test-driven-repair for an observed defect and architecture-design for an unresolved system boundary.

For new behavior with a meaningful automated check, establish the expected result independently of the implementation and observe the missing behavior before filling it in. Preserve existing working code; use test-strategy if test sensitivity is uncertain. Prose or cosmetic edits need proportionate inspection, not an artificial test-first ceremony.

## Focused execution checks

Do not turn unspecified optional/edge behavior into an indispensable owner requirement. For a precise objective, use a conventional reversible choice within the requested scope and record consequential assumptions; ask for genuine missing intent or conflicts. Continue independently defined work. Never report completed while admitting that required final execution or verification remains pending. Repair authorized missing dependencies and rerun the relevant checks after the last edit.

## Verification

Run the actual entrypoint on representative inputs and check the delivered artifact. Choose checks from explicit requirements and plausible failures introduced by this change. Reuse existing tests and tool evidence first; add only a missing case that could change acceptance. An import or successful exit is insufficient when the request concerns observable behavior. Batch related checks in one command when practical, then stop once the relevant evidence passes; do not invent requirements, arbitrary coverage targets or unrelated edge cases.

For an explicit delivery audit, delegated results or competing success signals, discover and load completion-verification. It checks the current artifact against the requested outcome and also applies to non-software deliverables. Ordinary implementation still uses the entrypoint checks above.

For a delivery governed by several requirements, also read [completion evidence](references/completion-evidence.md). Passing tests establishes only the behavior they exercise; inspect acceptance coverage separately. Route confirmed specification gaps through completion-verification, preserving valid completed work instead of restarting the plan.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.

## Differential verification

A self-authored differential test needs validated fixtures, matching effective paths and identical starting data. A mismatched or truncated fixture is not evidence of a code defect. Preserve the failed receipt; repair the fixture and rerun the comparison. If the command changes, task_contract resolve_check requires the original requirement ID, a source-grounded reason and a fresh passing comparison receipt. Do not dismiss a failed requested check or relabel unfinished work as completed.

Valida las pruebas propias antes de diagnosticar un defecto: entradas, rutas efectivas y datos iniciales equivalentes. Conserva el fallo y ejecuta la comparación corregida. No descartes comprobaciones solicitadas ni cambies a «completado» si queda trabajo relevante pendiente.

Vincula las comprobaciones al requisito y al riesgo del cambio. Reutiliza pruebas y resultados existentes, añade solo casos relevantes que falten y agrupa comprobaciones económicas. Una importación no demuestra el comportamiento solicitado. No inventes requisitos ni prolongues pruebas sin evidencia nueva.

Use the current runtime autonomy preference for implementation choices. Regardless of mode, clarify genuinely unresolved objective/scope/acceptance questions at intake before dependent changes. During execution, resolve ordinary technical uncertainty from evidence and repair installable dependencies within authorization. An autonomous preference never changes the requirements, approval policy or completion audits. A coherent diagnosis/repair/check loop should not end just because an edit failed. Use a patch error's file and candidate lines to narrow the repair, then rerun the relevant check.

En español: la autonomía decide quién resuelve detalles técnicos; las dudas reales sobre el objetivo inicial siempre se consultan. No cambia permisos ni requisitos ni auditorías. Repara dependencias y parches dentro del encargo, sin abandonar tras un error recuperable.

No conviertas detalles de borde no especificados en requisitos bloqueantes. Conserva las preguntas sobre intención o conflictos reales y avanza en lo independiente. No declares completado mientras falten comprobaciones requeridas de la versión final.
