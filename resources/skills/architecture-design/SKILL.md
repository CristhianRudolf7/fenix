---
name: architecture-design
description: Choose system boundaries and design tradeoffs from concrete workloads and failure modes.
metadata:
  version: 1.1.0
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

For each consequential boundary, identify who owns failures: trigger, partial state/effects, retry or compensation, user-visible error, detection signal and the check that exercises it. Include tenant/identity changes, cancellation and duplicate requests where relevant. Use a small failure table or state diagram when it clarifies ownership; do not require artifacts for unchanged trivial paths.

Include how a new binary, package, container or schema reaches its consumers: supported platforms, installation/update path and compatibility during rollout. A working local module does not establish distribution. Keep optional infrastructure separate from what the requested outcome actually needs.

## Verification

Walk representative scenarios end to end and tie each component to a requirement.

## Limits

A diagram is not an implemented system; avoid adding brokers, graphs or caches without a demonstrated role.
