---
name: architecture-design
description: Choose system boundaries and design tradeoffs from concrete workloads and failure modes.
metadata:
  version: 1.0.0
  category: engineering
  tags: architecture, design
  aliases: arquitectura, diseño sistema, monolito, microservicios, ADR
  required_tools: file_read,file_write,command_run
  requires: System requirements and existing architecture
  effects: mixed
---
# Architecture Design

## Scope

Choose system boundaries and design tradeoffs from concrete workloads and failure modes.

## Inputs

Workload, data ownership, reliability needs, constraints and decision to make. Prerequisite: System requirements and existing architecture.

## Workflow

1. Map user flows, stores, external dependencies and ownership. Separate measured constraints from speculative scale.
2. Compare the smallest viable design with alternatives using consistency, latency, operational burden and migration cost. Do not choose microservices solely by fashion.
3. Trace normal operation, dependency failure, duplicate delivery and recovery through each boundary. State what can be eventually consistent and what cannot.
4. Record the selected decision, rejected alternatives, assumptions and triggers for revisiting it.

## Verification

Walk representative scenarios end to end and tie each component to a requirement.

## Limits

A diagram is not an implemented system; avoid adding brokers, graphs or caches without a demonstrated role.

## Español

Selecciona límites y componentes según cargas y fallos concretos, con alternativas y supuestos explícitos.
