---
name: database-schema-design
description: Design relational schemas with explicit identities, constraints, tenancy and query-driven indexes.
metadata:
  version: 1.0.0
  category: engineering
  tags: database, schema, design
  aliases: esquema base datos, tablas, claves, postgres, sqlite
  required_tools: file_read,file_write,command_run
  requires: Target database version and expected query patterns
  effects: mixed
---
# Database Schema Design

## Scope

Design relational schemas with explicit identities, constraints, tenancy and query-driven indexes.

## Inputs

Entities, invariants, ownership, growth and consistency requirements. Prerequisite: Target database version and expected query patterns.

## Workflow

1. Model stable keys and relationships; enforce essential uniqueness and references in the database where supported.
2. Specify null semantics, deletion behavior, currency/time representation and tenant keys. Check whether uniqueness must include tenant scope.
3. Choose indexes from real predicates and ordering; verify engine-specific behavior such as foreign-key indexing and nullable uniqueness.
4. Review transaction boundaries and concurrent writes. Keep a migration path for existing rows and avoid premature denormalization.

## Verification

Try invalid/duplicate/cross-tenant records and representative queries against the target engine.

## Limits

Do not apply PostgreSQL assumptions to SQLite or assume ORM validation protects concurrent writers.
