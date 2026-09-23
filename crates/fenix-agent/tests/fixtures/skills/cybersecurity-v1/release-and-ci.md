---
name: release-and-ci
description: Prepare releases and CI changes with reproducible checks, accurate versions and a verifiable artifact.
metadata:
  version: 1.0.0
  category: engineering
  tags: release, and, ci
  aliases: release, versión, CI, pipeline, publicar paquete
  required_tools: file_read,file_edit,command_run
  requires: Repository release conventions and configured build/test tools
  effects: mixed
---
# Release And Ci

## Scope

Use for CI configuration or release preparation. Use package-build-and-install for package consumer checks and Git skill for history operations.

## Inputs

Target version, change scope, repository policy, artifact and requested publication action.

## Workflow

1. Inspect existing workflows, lockfiles, version sources and release conventions. Identify which checks protect the affected behavior.
2. Run the relevant checks locally or through the authorized CI path. Diagnose infrastructure failures separately from product failures; do not make a failing test disappear by dropping it.
3. Keep build inputs pinned where the project expects reproducibility. Avoid exposing credentials in logs, forked pull requests or artifacts.
4. Update version references and user-facing changes consistently. Build the actual release artifact and smoke-test its entrypoint or install path.
5. Publish, tag or deploy only if requested or already authorized. On remote failure, inspect release state before repeating to prevent duplicate artifacts.

## Verification

Link the commit, workflow run and artifact when available. Distinguish prepared, built, uploaded and deployed states; CI success is not proof of deployment.

## Limits

Do not invent release notes from uncommitted assumptions or rewrite remote tags without authorization.

## Español

Prepara versiones y artefactos con comprobaciones reproducibles. Distingue lo preparado de lo publicado y verifica el estado remoto antes de repetir acciones.
