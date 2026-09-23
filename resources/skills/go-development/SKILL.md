---
name: go-development
description: Implement Go services and tools with bounded goroutine lifetimes, context propagation and explicit
  errors.
metadata:
  version: 1.0.0
  category: engineering
  tags: go, development
  aliases: go, golang, goroutines, canales
  required_tools: file_read,file_write,command_run
  requires: Project Go version and modules
  effects: mixed
---
# Go Development

## Scope

Implement Go services and tools with bounded goroutine lifetimes, context propagation and explicit errors.

## Inputs

Package, interface, lifecycle and concurrency expectations. Prerequisite: Project Go version and modules.

## Workflow

1. Read go.mod, existing interfaces and callers. Prefer small behavior-driven interfaces at the consumer boundary.
2. Propagate context cancellation through I/O and worker loops. Define who closes channels and drains errors; avoid a goroutine that can outlive its task indefinitely.
3. Wrap errors with useful context and preserve errors.Is/As behavior. Close response bodies and other resources on every relevant path.
4. Use gofmt, targeted tests and go vet; run the race detector for shared-state changes when supported.

## Verification

Exercise shutdown, error paths and repeated invocations; a normal output alone does not prove absence of leaked goroutines.

## Limits

Do not add channels for sequential work or ignore cancellation because a worker is in the background.

## Español

Define quién termina cada goroutine y cierra canales. Propaga cancelación y verifica errores, carreras y cierre.
