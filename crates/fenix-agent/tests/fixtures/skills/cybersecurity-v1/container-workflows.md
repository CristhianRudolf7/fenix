---
name: container-workflows
description: Build and diagnose containers with reproducible images, correct mounts and graceful process lifecycle.
metadata:
  version: 1.0.0
  category: operations
  tags: container, workflows
  aliases: docker, contenedores, imagen, compose
  required_tools: file_read,file_write,command_run
  requires: Available container runtime within the authorized environment
  effects: mixed
---
# Container Workflows

## Scope

Build and diagnose containers with reproducible images, correct mounts and graceful process lifecycle.

## Inputs

Image/build context, runtime command, mounts and intended service. Prerequisite: Available container runtime within the authorized environment.

## Workflow

1. Inspect build context and ignore rules; separate build dependencies from runtime contents and avoid copying credentials into image layers.
2. Pin relevant inputs using the project policy. Check target architecture, user permissions, working directory and writable paths.
3. Run the main process with correct signal handling; distinguish image health, process liveness and functional readiness.
4. Diagnose logs, mounts, network and exit reasons before rebuilding. Preserve data volumes when replacing disposable containers.

## Verification

Build the actual image and test startup, shutdown and the service endpoint using intended mounts.

## Limits

Do not mount the host root/socket or run privileged merely to bypass a failing permission check.

## Español

Verifica contexto, capas, permisos y señales; conserva volúmenes y prueba el servicio real.
