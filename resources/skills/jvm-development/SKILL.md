---
name: jvm-development
description: Implement Java or Kotlin changes across transactions, nullability, coroutines and JVM service boundaries.
metadata:
  version: 1.0.0
  category: engineering
  tags: jvm, development
  aliases: java, kotlin, jvm, spring, gradle, maven
  required_tools: file_read,file_write,command_run
  requires: Existing JVM, build wrapper and framework versions
  effects: mixed
---
# Jvm Development

## Scope

Implement Java or Kotlin changes across transactions, nullability, coroutines and JVM service boundaries.

## Inputs

Module, API contract, persistence behavior and supported runtime. Prerequisite: Existing JVM, build wrapper and framework versions.

## Workflow

1. Use the repository Gradle/Maven wrapper and configured JDK. Inspect dependency injection, transaction entrypoints and persistence mappings.
2. Preserve nullability at Java/Kotlin boundaries. Do not swallow coroutine cancellation; distinguish thread blocking from suspending work.
3. Check transaction scope, lazy loading and query counts through the real caller. A method annotation may not apply through internal/self invocation.
4. Write a focused change and run module tests; use an integration check when database/proxy/framework behavior determines correctness.

## Verification

Exercise serialization, rollback and cancellation as applicable, under the configured runtime.

## Limits

Do not force a JDK/framework upgrade or a new architecture because an upstream guide uses it.
