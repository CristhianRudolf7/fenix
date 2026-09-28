---
name: process-and-service-management
description: Start, inspect, test and stop background processes and persistent HTTP services. Procesos servicios
  puertos.
metadata:
  version: 1.4.0
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

## Focused execution checks

A requirement to keep serving includes the lifetime after the agent turn. Use the runtime persistent service mechanism and confirm a real protocol response, not only a temporary successful probe inside a shell that will terminate. Preserve its process receipt and do not cancel it during final cleanup.

## Execute the requested outcome

Distinguish configuring a running service from writing a setup script. If the task requests a working service in the authorized environment, install required dependencies within scope, apply the configuration, start it and exercise the requested client flow before completion. A script plus README or bash -n is not that outcome. Inspect uid, executable locations (including /usr/sbin), PID 1 and service capabilities first; systemctl being installed does not prove systemd is running. Use the existing supervised-service mechanism when there is no service manager. Ask about genuinely unspecified external hosts, but do not invent a remote destination just because the current directory is empty. An explicit sandbox assignment establishes the execution target; follow it without asking for production hostname details.

Before starting a dependent service, wait for its required downloads/installations to finish and verify readiness. After editing running server code, restart the owned service intentionally unless verified hot reload applies; old processes do not load changed source automatically. Then use a bounded readiness wait and a real request to the changed endpoint. Reuse the registered process ID and visible tool schemas. A TCP connection or a stale response from the previous instance is not validation of the edit.

## Verification

Inspect the supervised process and exercise the real protocol endpoint.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.
