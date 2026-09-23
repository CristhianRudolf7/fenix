---
name: python-development
description: Implement Python behavior with correct environments, resource lifetime, exceptions and async execution.
metadata:
  version: 1.1.0
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
5. Use the existing formatter/type checker/test runner. Check package behavior from its actual consumer when changing imports or packaging.

## Verification

Exercise real inputs and failure cleanup; rerun relevant tests and the configured type checks.

## Limits

Do not mandate a new formatter, async framework or arbitrary coverage percentage. Never unpickle untrusted data.

## Español

Usa el intérprete del proyecto, conserva interfaces y comprueba errores y limpieza; los tipos no validan datos externos.

Prepara las pruebas en el recurso que realmente utilizan; dos conexiones independientes a una base en memoria no comparten tablas por defecto.
