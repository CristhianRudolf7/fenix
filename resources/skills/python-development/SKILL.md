---
name: python-development
description: Implement Python behavior with correct environments, resource lifetime, exceptions and async execution.
metadata:
  version: 1.5.0
  category: engineering
  tags: python, development
  aliases: python, pytest, asyncio, entorno virtual
  required_tools: file_read,file_write,command_run
  requires: Existing project interpreter and declared dependencies
  effects: mixed
---
# Python Development

## Scope

Implement Python behavior with correct environments, resource lifetime, exceptions and async execution.

## Inputs

Entry point, Python version, packaging conventions and intended behavior. Prerequisite: Existing project interpreter and declared dependencies.

## Workflow

1. Read pyproject/lockfiles and use the project interpreter, not an unrelated global Python. Preserve import paths and public interfaces.
2. Distinguish blocking I/O from coroutine work; offload only where needed and close files, sessions and generators on cancellation.
3. Catch exceptions where recovery is possible; retain causes and avoid broad success-shaped fallbacks. Validate untrusted values at boundaries rather than trusting annotations.
4. Keep fixtures and operations on the same resource: separate in-memory database connections do not share initialized tables by default. Check transaction ownership and cleanup on the connection actually under test.
5. For ports or differential tests, derive input record widths, padding, numeric representation and encoding from the source. Use separate copies of identical initial data. Verify the actual Python entry point: changing cwd does not redirect paths based on `__file__`. Run valid inputs first; label malformed-input experiments separately. Diagnose test setup before treating a mismatch as an implementation bug.
6. Use the existing formatter/type checker/test runner. Check package behavior from its actual consumer when changing imports or packaging.

## Focused execution checks

For compatibility/reimplementation work, compare a successful operation from identical independent initial data, not just unchanged outputs from rejected input. When Python is missing, inspect available versions and install the required interpreter within existing authorization; do not substitute syntax guesses for execution or silently change a required version. Rerun affected execution paths after the last patch.

For mixed Python/Cython packages, inspect declared extension sources and search relevant .pyx/.pxd files as well as .py before declaring a compatibility migration complete. Test behavior of the required compiled modules, not only imports. Reuse existing examples/tests and preserve pinned dependency versions. After a patch mismatch, inspect the named region and add unique context; do not retry identical hunks or repeatedly reinstall unchanged sources.

## Verification

Exercise real inputs and failure cleanup; rerun relevant tests and the configured type checks. When cancellation or concurrency is part of the requested behavior, test active and queued work, asynchronous cleanup and propagation at the actual boundary (for example SIGINT only when keyboard interruption matters). Synchronize tests with events rather than assuming a task started after a sleep. Select the smallest cases that expose the risk; do not require these tests for unrelated Python edits.

## Limits

Do not mandate a new formatter, async framework or arbitrary coverage percentage. Never unpickle untrusted data.
