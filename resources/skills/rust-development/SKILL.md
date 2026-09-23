---
name: rust-development
description: Implement Rust changes involving ownership, async cancellation, traits or unsafe boundaries.
metadata:
  version: 1.0.0
  category: engineering
  tags: rust, development
  aliases: rust, préstamos, ownership, lifetimes, tokio
  required_tools: file_read,file_write,command_run
  requires: Project Rust toolchain and Cargo workspace
  effects: mixed
---
# Rust Development

## Scope

Implement Rust changes involving ownership, async cancellation, traits or unsafe boundaries.

## Inputs

Crate, requested behavior, supported toolchain and failing example if any. Prerequisite: Project Rust toolchain and Cargo workspace.

## Workflow

1. Read workspace features, target configuration and the ownership path across callers. Prefer borrowing or explicit ownership transfer to adding clones or global locks to silence errors.
2. Keep blocking work off async executors. Examine cancellation at await points, resource cleanup and mutex guards held across awaits; spawned work needs an owner and shutdown path.
3. Use Result for recoverable failures and preserve error context. For unsafe/FFI code, document the actual lifetime, alignment, aliasing and thread-safety invariants.
4. Change the narrow crate boundary and test affected features/build modes; use the project formatter and Clippy configuration.

## Verification

Reproduce the requested behavior; for concurrency or unsafe changes, exercise cancellation/race or sanitizer/Miri checks when supported.

## Limits

Do not change the MSRV, runtime or error policy just to simplify implementation. A successful debug build does not establish release-mode safety.

## Español

Revisa propiedad, cancelación y límites unsafe; prueba las variantes relevantes sin cambiar el toolchain por comodidad.
