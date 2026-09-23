---
name: threat-modeling
description: Map assets, trust boundaries and misuse scenarios into prioritized security controls and open questions.
metadata:
  version: 1.0.0
  category: security
  tags: threat, modeling
  aliases: modelo amenazas, riesgo, límites confianza, STRIDE
  required_tools: file_read,files_search,file_write
  requires: System description or repository architecture
  effects: local-write
---
# Threat Modeling

## Scope

Use for defensive design analysis before or alongside implementation. No dedicated modeling application is required.

## Inputs

System scope, actors, data flows, sensitive assets and existing controls.

## Workflow

1. Draw a compact inventory of processes, stores, external services and identity boundaries. Mark unknown deployment assumptions instead of treating them as facts.
2. For each boundary, ask who can impersonate an actor, alter data, gain unintended access or exhaust a resource. Include cross-user and background-agent paths.
3. Turn risks into concrete scenarios with preconditions, affected assets and existing mitigations. Avoid a generic list disconnected from this system.
4. Rank scenarios by plausible impact and exposure. Propose a control and a way to verify it, with owner/status if known.
5. Record accepted decisions separately from recommendations and link unresolved questions to the design components they affect.

## Verification

Every prioritized risk should connect to a real flow and a testable control. Revisit changed assumptions after architecture updates.

## Limits

Do not label recommendations as implemented controls or turn a threat model into live penetration testing.

## Español

Relaciona amenazas con activos y flujos reales. Prioriza controles verificables y conserva explícitos los supuestos y riesgos pendientes.
