---
name: agent-workflow-diagnosis
description: Investigate an agent session that repeated work, missed a skill, ignored instructions or spent excessive time or tokens, using task traces. Diagnosticar ejecuciones del agente.
metadata:
  version: 1.0.0
  category: operations
  tags: agent, workflow, diagnosis, session, traces, tokens
  aliases: agent session, repeated work, missed skill, trazas del agente, misma tarea, dos veces, skill omitida, no se cargó, tokens
  required_tools: task_inspect,history_search,history_read,skill_search,skill_load
  requires: Task or conversation evidence in the authorized project scope
  effects: read-only
---
# Agent Workflow Diagnosis

## Scope

Explain a specific failure in the agent's execution: repeated completed actions, an applicable skill never loaded, ignored requirements, unsupported success claims or unexplained time/token use. Use local-troubleshooting for a failing local service or command; use conversation-recap for a neutral summary of earlier decisions.

## Inputs

Task/project identity, the unexpected behavior, the relevant interval and the owner's expected outcome. Recover these from the request and existing evidence before asking for missing information.

## Workflow

1. Recover bounded task and history pages. Build the relevant sequence of requirements, revisions, skill loads, tool results, waits, worker reports and checks. Preserve task/event/checkpoint references; a coordinator's message is not a new instruction from the owner.
2. For a missed skill, distinguish an absent search candidate, a disabled entry, a failed load and loaded instructions not followed. Current installed text does not prove which version a past task used. Missing events in an incomplete trace leave the question unresolved.
3. For repetition after resumption or compaction, compare the actual tool receipts and artifact identities. Count completed effects repeated without need separately from legitimate reads, changed-input retries and status checks that observed progress. An executed=false result is not a completed action.
4. For excessive time or tokens, separate inference, command execution and waiting for answers. Concurrent worker durations do not sum to wall-clock latency. Sum attributable provider usage once; report missing records. Leave monetary cost unknown without the exact model, dated price and applicable usage dimensions.
5. Explain the supported cause, alternative explanations and missing evidence. Propose the smallest corrective experiment: better discovery wording, clearer instructions, an engine fix or a changed tool contract. A skill rewrite alone cannot repair a deterministic runtime defect.

## Verification

Every material finding identifies the event or artifact supporting it and what would disprove the interpretation. Distinguish an observed defect from a hypothesis. State the trace interval and omitted evidence; do not turn absent logs into proof of success or failure.

## Limits

Diagnosis stays read-only. Do not rewrite instructions, rerun effects, export transcripts, post issues or search other projects unless that work is authorized. Read large events in bounded slices and preserve originals. This skill does not require analyst subagents or add execution retries.

## Español

Investiga por qué el agente repitió la misma tarea, no cargó una skill, ignoró instrucciones o consumió tiempo y tokens. Reconstruye los hechos con referencias, distingue efectos ejecutados de intentos rechazados y comprueba la versión de las instrucciones cuando haya evidencia. Separa espera, ejecución e inferencia; no sumes tiempos concurrentes ni inventes costes. Entrega causas sustentadas, hipótesis y una comprobación siguiente. Diagnosticar no autoriza modificar el agente ni exportar trazas.
