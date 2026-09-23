---
name: software-implementation
description: Implement a new program, parser, interpreter or utility with a concrete runnable artifact. Crear programas
  y utilidades.
metadata:
  version: 1.2.0
  category: engineering
  tags: software, implementation
  aliases: programación, implementar, programa, software
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

### Build and check a small working slice

Create the entry artifact and implement one valid input-to-output path early. Run it through the actual caller or command, then extend behavior. A plan or explanation does not deliver a requested program. For parsers and interpreters, keep parsing, representation and evaluation boundaries explicit; test a small valid case, composition and a malformed input consistent with the requested interface. For utilities, check exit status and the generated output through its consumer.

When a test fails, isolate the smallest observed discrepancy and change that path. Reuse successful work. Before completing, check the required artifact exists and run representative behavior checks under the requested invocation. Report remaining failures and checks that could not run. Do not substitute a different language, output format or command when these were specified.

Use a language-specific skill when the task hinges on ownership, concurrency, runtime types or platform semantics. Discover and load only the relevant one; a small language-neutral edit does not need every engineering skill. Use test-driven-repair for an observed defect and architecture-design for an unresolved system boundary.

## Verification

Run the actual entrypoint on representative inputs and check the delivered artifact.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.

## Español


Identifica el archivo entregable y su interfaz en la solicitud original. Ante una contradicción aparente del resumen, recupera la fuente. Crea pronto una versión mínima ejecutable y amplíala con comprobaciones funcionales. Las decisiones de implementación no añaden requisitos. Comprueba el resultado mediante su invocación o consumidor real y comunica lo pendiente.
