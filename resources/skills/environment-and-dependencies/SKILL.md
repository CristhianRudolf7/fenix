---
name: environment-and-dependencies
description: Prepare development environments and diagnose dependency, build and installation failures. Entornos
  y dependencias.
metadata:
  version: 1.3.0
  category: engineering
  tags: environment, and, dependencies
  aliases: entorno, dependencias, instalar, compilación
  required_tools: command_run,command_status,command_input,file_read
  requires: Available task-authorized Fénix tools
  effects: mixed
---
# Environment And Dependencies

## Scope

Prepare development environments and diagnose dependency, build and installation failures. Entornos y dependencias.

## Inputs

Project toolchain, lockfile, target environment and exact setup failure.

## Workflow

Inspect the existing lockfile and interpreter/toolchain before installing. Prefer the project-supported isolated environment and pinned dependencies. Read the first actionable error; do not repeat the same installation unchanged. Separate network failure, missing executable, version incompatibility and build failure. A running installation with no recent output may still be working: wait for its process event or bounded status. Preserve successful setup and do not cancel progressing work just to inspect files. Verify imports or the actual consumer command inside the intended environment.

### Local diagnostic recovery

Separate a failure in the diagnostic tool or container from a failure in the target program. Read the exact error, including the end of the output. If it identifies an environment constraint, inspect that constraint before rerunning. Apply only an authorized, reversible, process-local correction; do not change host-wide settings or invent application behavior. Then rerun the original diagnostic and report whether it actually observed the program. A tool that could not start is not evidence that the code is correct. Ask the owner about genuinely unresolved requirements or missing permission, not facts that the available files or diagnostics already establish.

### Unattended installation

Inspect what is already installed and use only needed dependencies. For authorized unattended Debian setup, `DEBIAN_FRONTEND=noninteractive` is process-local and appropriate when package defaults satisfy the task; `-y` alone does not answer configuration dialogs. Preserve established timezone/configuration and do not invent owner preferences or accept licenses implicitly. If output explicitly asks for input, inspect the prompt and answer through command_input using known task settings. Empty incremental output does not erase the earlier prompt. Do not cancel an active package manager merely to inspect it. If an interruption already occurred, let it release its locks and follow its diagnostic to repair pending configuration before retrying; never delete lock files or run competing package managers. Distinguish a missing sbin PATH entry from a missing executable.

## Verification

Use the actual consumer command inside the intended environment after correction.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.
