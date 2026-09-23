---
name: software-implementation
description: Implement a new program, parser, interpreter or utility with a concrete runnable artifact. Crear programas
  y utilidades.
metadata:
  version: 1.5.1
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

Create the entry artifact and implement one valid input-to-output path early. Run it through the actual caller or command, then extend behavior. A plan or explanation does not deliver a requested program. For parsers and interpreters, keep parsing, representation and evaluation boundaries explicit; test a small valid case, composition and a malformed input consistent with the requested interface. For utilities, check exit status and the generated output through its consumer.

When a test fails, isolate the smallest observed discrepancy and change that path. Reuse successful work. Before completing, check the required artifact exists and run representative behavior checks under the requested invocation. Report remaining failures and checks that could not run. Do not substitute a different language, output format or command when these were specified.

Use a language-specific skill when the task hinges on ownership, concurrency, runtime types or platform semantics. Discover and load only the relevant one; a small language-neutral edit does not need every engineering skill. Use test-driven-repair for an observed defect and architecture-design for an unresolved system boundary.

For new behavior with a meaningful automated check, establish the expected result independently of the implementation and observe the missing behavior before filling it in. Preserve existing working code; use test-strategy if test sensitivity is uncertain. Prose or cosmetic edits need proportionate inspection, not an artificial test-first ceremony.

## Verification

Run the actual entrypoint on representative inputs and check the delivered artifact.

For an explicit delivery audit, delegated results or competing success signals, discover and load completion-verification. It checks the current artifact against the requested outcome and also applies to non-software deliverables. Ordinary implementation still uses the entrypoint checks above.

For a delivery governed by several requirements, also read [completion evidence](references/completion-evidence.md). Passing tests establishes only the behavior they exercise; inspect acceptance coverage separately. Route confirmed specification gaps through completion-verification, preserving valid completed work instead of restarting the plan.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.

## Español


Identifica el archivo entregable y su interfaz en la solicitud original. Ante una contradicción aparente del resumen, recupera la fuente. Crea pronto una versión mínima ejecutable y amplíala con comprobaciones funcionales. Las decisiones de implementación no añaden requisitos. Comprueba el resultado mediante su invocación o consumidor real y comunica lo pendiente.

Para comportamiento nuevo comprobable, define resultados independientes y observa el caso que falta. Para auditar una entrega, usa completion-verification; las comprobaciones normales de implementación siguen siendo parte de esta skill.

En repositorios con LSP configurado, ubica archivos y consulta símbolos antes de leer cuerpos completos. Usa referencias con contexto si necesitas comprobar impacto; resuelve nombres ambiguos y no edites desde un cuerpo truncado. Mantén las herramientas de edición actuales. Para cambios pequeños o sin LSP, basta buscar y leer archivos: no hace falta instalar infraestructura adicional.

Al implementar una especificación, conserva referencias a requisitos y ejemplos de aceptación en tareas y comprobaciones. Contrasta contratos vigentes, límites y errores entre productor y consumidor; no cambies lo pedido para justificar el código. Revisa cobertura de requisitos además del resultado de las pruebas y conserva trabajo válido al corregir pendientes.
