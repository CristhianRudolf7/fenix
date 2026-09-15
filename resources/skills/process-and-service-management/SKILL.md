---
name: process-and-service-management
description: Start, inspect, test and stop background processes and persistent HTTP services. Procesos servicios puertos.
version: 1.0.0
required_tools: command_run,command_status,command_output,command_cancel
---

# Process And Service Management

## Procedure

Use persistent=true explicitly for a service that must remain available after the turn. Run the service in the foreground of the supervised shell; do not daemonize, call setsid or exit the supervisor. Ordinary transient shell descendants are cleaned up when their shell ends; command ending successfully is not proof a server remains alive. Inspect the registered process and use ready_port only as a TCP availability probe. Then send a real protocol request to the required endpoint and verify its response. Save the process ID, endpoint and observations. Never infer ownership or application health from an open port. Stop only services that should end or are demonstrably unusable.

## Evidence and scope

Use task-local observed outputs as evidence. This is a manually curated procedure, not a validated benchmark solution. Consult current project instructions. Tool availability and owner permissions take precedence; report missing capabilities.

## Español

Declara persistent=true para servicios durables. Comprueba proceso, puerto y respuesta real; un puerto abierto no prueba salud funcional.
