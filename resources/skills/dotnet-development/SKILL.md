---
name: dotnet-development
description: Implement C# and .NET changes with correct dependency lifetimes, async I/O and persistence boundaries.
metadata:
  version: 1.0.0
  category: engineering
  tags: dotnet, development
  aliases: csharp, C#, dotnet, aspnet, entity framework
  required_tools: file_read,file_write,command_run
  requires: Project .NET SDK, solution and NuGet lock policy
  effects: mixed
---
# Dotnet Development

## Scope

Implement C# and .NET changes with correct dependency lifetimes, async I/O and persistence boundaries.

## Inputs

Target framework, affected service and observable outcome. Prerequisite: Project .NET SDK, solution and NuGet lock policy.

## Workflow

1. Inspect solution/project files, nullable settings and dependency-injection lifetimes. Avoid capturing a scoped service inside a singleton.
2. Propagate CancellationToken through async calls. Do not block on Task.Result in request paths or start unowned fire-and-forget work.
3. Keep DTO validation and authorization at boundaries; inspect EF query translation and transaction behavior rather than only in-memory results.
4. Run project build/tests and exercise the affected HTTP or consumer path with realistic service registrations.

## Verification

Check disposal, cancellation, null cases and database behavior relevant to the change.

## Limits

Preserve supported framework versions; compilation does not validate dependency lifetime or SQL behavior.

## Español

Comprueba ciclos de vida de servicios, cancelación y consultas reales; conserva la versión .NET del proyecto.
