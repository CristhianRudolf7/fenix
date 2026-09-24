---
name: package-build-and-install
description: Build packages, wheels and indexes; verify a fresh consumer installation. Paquetes instalación distribución.
metadata:
  version: 1.3.0
  category: engineering
  tags: package, build, and, install
  aliases: paquete, distribución, wheel, instalación limpia
  required_tools: command_run,command_status,file_read,file_write
  requires: Available task-authorized Fénix tools
  effects: mixed
---
# Package Build And Install

## Scope

Build packages, wheels and indexes; verify a fresh consumer installation. Paquetes instalación distribución.

## Inputs

Package source, declared build backend, artifact and consumer environment.

## Workflow

Keep diagnosis, dependency repair, source changes, installation and consumer verification in one end-to-end execution assignment when they share one environment and form a serial feedback loop. A plan can track these as checklist items without creating a new worker and audit for each step. Delegate separately only when work is genuinely independent or a distinct deliverable warrants it. Preserve the final completion audit for substantial work.

Inspect the packaging layout and build metadata. Build the distributable using the declared backend. Inspect the produced artifact. If serving an index, verify the actual package-specific endpoint and links using the relevant packaging standard, not just the root listing. Keep the service supervised with persistent=true. Test installation from a separate clean environment using the requested source; a message such as Requirement already satisfied does not exercise that source. Import and use the installed package from outside the source checkout. Record the artifact, source URL and fresh-install result. Do not hard-code benchmark package names or hidden expected output.

## Completion and recovery

Keep the requested consumer environment authoritative. If the user requires a global installation or a fixed dependency version, verify that exact interpreter/environment and preserve that version; an isolated environment is only an additional diagnostic, not a substitute. Inspect build and runtime dependencies before choosing installation flags. Using --no-deps does not make missing runtime dependencies optional. A wheel built in isolation may differ from an in-place build; verify the installed artifact really contains and loads the requested native modules.

Treat missing build/runtime dependencies and ordinary compiler/import failures as actionable diagnostics, not terminal blockers. Within the authorized installation scope, identify and install the missing compatible dependency, retry the failed check, and follow the next observed error. Do not reinstall an unchanged artifact as a substitute for resolving a missing dependency. Do not change pinned versions or broaden permissions to make a test pass. If authorization or a real external prerequisite is missing, ask a concrete question instead of ending with unfinished implementation advice.

Keep a compact ledger of the target interpreter, preserved versions, completed build/install commands, latest failing check, and next repair. Recover these facts after compaction; do not repeat completed installs or reload the same procedure merely because the transcript was summarized. For builds/installations that are still running, use command_status with a meaningful wait rather than repeated one-second polls.

## Verification

From outside the source checkout, import and exercise the installed package in the requested consumer environment. Run the user's example and applicable existing tests, respecting explicit exclusions. Verify both native extension loading and actual Python-facing behavior. A successful build/install log does not establish that the requested use case works. Continue fixing actionable failures before finalizing; report the exact checks and unresolved limitations without claiming completion of unfinished work.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.

## Español


Construye el artefacto y pruébalo fuera del repositorio en el intérprete y entorno solicitados, conservando versiones fijadas. Corrige dependencias faltantes y errores reparables antes de terminar; reinstalar sin resolver la causa no basta. Ejecuta el ejemplo del usuario y las pruebas pertinentes. Conserva el último error y el siguiente paso al compactar; pregunta solo ante un bloqueo real que requiera al usuario.

Mantén diagnóstico, reparación, instalación y verificación en un mismo trabajador cuando dependan del mismo entorno y formen un ciclo secuencial. Usa una lista de comprobación sin crear un agente y una auditoría para cada paso. Conserva la auditoría final del trabajo sustancial.
