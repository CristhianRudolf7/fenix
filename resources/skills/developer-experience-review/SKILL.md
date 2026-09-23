---
name: developer-experience-review
description: Evaluate onboarding and usability of a developer-facing API, CLI, SDK or documentation, from first setup to a verified useful result. Auditar experiencia del desarrollador.
metadata:
  version: 1.0.0
  category: engineering
  tags: developer, experience, onboarding, usability
  aliases: DX audit, devex, developer onboarding, first successful API call, getting started friction, experiencia desarrollador, primera integración, auditar SDK, facilidad de instalación
  required_tools: file_read,files_search,command_run,tool_search
  requires: Target interface and documentation; an isolated test environment for executable checks
  effects: mixed
---
# Developer Experience Review

## Scope

Evaluate whether an intended developer can discover, install, use and recover from errors in a product. Distinguish a proposed journey from a live trial. Use api-contract-design to design schemas; technical-documentation to write guides; package-build-and-install for a build failure. This audit reports findings; implementation follows only when included in the user's request.

## Inputs

Target version, intended developer, supported environment, starting knowledge and first useful outcome. Recover these from the brief and docs; ask only when ambiguity changes the experiment. A successful install is not necessarily a useful result.

## Workflow

1. Trace the documented path from discovery through prerequisites, authentication, first result and recovery. Name the actual commands/pages and required accounts. In plan-only work label predicted friction as a hypothesis, not an observed failure.
2. For a live trial, use a clean temporary project and existing authorized tools. Inspect install scripts and command side effects before execution. Keep personal configuration, production accounts and shared services out of the experiment. Discover browser tools only for relevant web steps.
3. Follow the public instructions without silently supplying undocumented expertise. Record starting conditions, each blocking step, the observed error and any workaround. Measure elapsed time only for an actual timed run; separate download, execution and waiting time. An experienced agent's trial is not a novice usability study.
4. Exercise a representative invalid input and recovery: missing configuration, expired test credentials or an unsupported option. Check that the message identifies the problem and a valid next action without leaking secrets. For machine consumers also check structured output, stable exit codes and noninteractive behavior.
5. Compare examples, CLI help, API/SDK names and installed behavior at the same version. Check upgrades only when in scope, preserving the old environment. Report missing coverage when an account, OS, device or external dependency is unavailable.
6. Prioritize concrete blockers before convenience improvements. Give each finding its reproduction, evidence, user consequence and smallest useful correction. If repair is authorized, rerun the same starting path afterward; otherwise leave source and configuration unchanged.

## Verification

Deliver a journey table with expected result, actual observation and evidence per step. Keep observed, inferred and untested separate. Compare a target time with a measured time only when their outcome and prerequisites match; avoid universal minute targets or uncalibrated scores.

## Limits

Do not sign up, buy a plan, publish packages or change live credentials merely to complete the audit. A skill provides no connector or browser by itself. Preserve existing authorization; do not add a confirmation ceremony for ordinary permitted checks.

## Español

Audita el recorrido real hasta el primer resultado útil: requisitos, instalación, autenticación, ejemplos y recuperación de errores. Separa pruebas ejecutadas de hipótesis sobre un plan. Registra bloqueos y tiempos observados sin inventar puntuaciones ni suponer que un agente experto representa a un principiante. Corrige solo dentro del encargo y verifica el mismo recorrido después.
