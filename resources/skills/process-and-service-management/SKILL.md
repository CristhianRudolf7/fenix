---
name: process-and-service-management
description: Start, inspect, test and stop background processes and persistent HTTP services. Procesos servicios
  puertos.
metadata:
  version: 1.1.0
  category: operations
  tags: process, and, service, management
  aliases: proceso, servicio, puerto, segundo plano
  required_tools: command_run,command_status,command_output,command_cancel
  requires: Available task-authorized Fénix tools
  effects: mixed
---
# Process And Service Management

## Scope

Start, inspect, test and stop background processes and persistent HTTP services. Procesos servicios puertos.

## Inputs

Owned process/service, desired lifetime, endpoint and requested lifecycle action.

## Workflow

Use persistent=true explicitly for a service that must remain available after the turn. Run the service in the foreground of the supervised shell; do not daemonize, call setsid or exit the supervisor. Ordinary transient shell descendants are cleaned up when their shell ends; command ending successfully is not proof a server remains alive. Inspect the registered process and use ready_port only as a TCP availability probe. Then send a real protocol request to the required endpoint and verify its response. Save the process ID, endpoint and observations. Never infer ownership or application health from an open port. Stop only services that should end or are demonstrably unusable.

## Verification

Inspect the supervised process and exercise the real protocol endpoint.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.

## Español


Declara persistent=true para servicios durables. Comprueba proceso, puerto y respuesta real; un puerto abierto no prueba salud funcional.
