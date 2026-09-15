---
name: software-implementation
description: Implement a new program, parser, interpreter or utility with a concrete runnable artifact. Crear programas y utilidades.
version: 1.0.0
required_tools: file_read,file_write,file_edit,command_run
---

# Software Implementation

## Establish the interface

Identify the requested artifact, entry command, inputs, outputs and constraints from the original request and existing callers. Keep exact paths and invocation syntax. If a summary appears inconsistent, retrieve the original before asking about the apparent conflict. Ask about unresolved requirements; an implementation choice with several valid solutions is not itself a missing requirement.

## Build and check a small working slice

Create the entry artifact and implement one valid input-to-output path early. Run it through the actual caller or command, then extend behavior. A plan or explanation does not deliver a requested program. For parsers and interpreters, keep parsing, representation and evaluation boundaries explicit; test a small valid case, composition and a malformed input consistent with the requested interface. For utilities, check exit status and the generated output through its consumer.

When a test fails, isolate the smallest observed discrepancy and change that path. Reuse successful work. Before completing, check the required artifact exists and run representative behavior checks under the requested invocation. Report remaining failures and checks that could not run. Do not substitute a different language, output format or command when these were specified.

## Español

Identifica el archivo entregable y su interfaz en la solicitud original. Ante una contradicción aparente del resumen, recupera la fuente. Crea pronto una versión mínima ejecutable y amplíala con comprobaciones funcionales. Las decisiones de implementación no añaden requisitos. Comprueba el resultado mediante su invocación o consumidor real y comunica lo pendiente.
