---
name: release-and-ci
description: Prepare releases and CI changes with reproducible checks, accurate versions and a verifiable artifact.
metadata:
  version: 1.2.0
  category: engineering
  tags: release, and, ci
  aliases: release, versión, CI, pipeline, publicar paquete, GitHub Actions security, workflow permissions, seguridad CI
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

When reviewing workflow security, load [CI trust boundaries](references/ci-trust-boundaries.md). Trace untrusted code, artifacts and caches into privileged jobs; verify permissions and provenance at the release boundary.

Bind check evidence to the actual artifact inputs, including dirty files, lockfiles, build configuration and dependency manifests. Version-only changes can still affect packaging: inspect their effect rather than exempting every manifest edit. After integration, rebase or an affecting final edit, rerun the relevant checks on the resulting state. Never sweep unrelated working changes into a release because they are present.

When the request includes deployment verification or canary observation, load [release observation](references/release-observation.md). A PR, a successful deployment job and healthy production traffic are different observations; report the strongest state actually established.

## Verification

Link the commit, workflow run and artifact when available. Distinguish prepared, built, uploaded and deployed states; CI success is not proof of deployment.

## Limits

Do not invent release notes from uncommitted assumptions or rewrite remote tags without authorization.
