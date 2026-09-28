---
name: incident-response
description: Triage an operational or security incident, preserve evidence and document recovery within authorized
  scope.
metadata:
  version: 1.2.0
  category: security
  tags: incident, response
  aliases: incidente, caída, respuesta, recuperación servicio, postmortem, forensic timeline, PCAP, línea temporal forense, evidencia forense, cloud incident, credential containment, incidente cloud
  required_tools: task_inspect,file_read,command_run,command_status
  requires: Access to relevant logs/state and an authorized operational scope
  effects: local-write
---
# Incident Response

## Scope

Use for ongoing incidents or post-incident analysis. Ordinary isolated local failures belong to local-troubleshooting.

## Inputs

Symptom, affected service/users, observation time, available evidence and allowed mitigation actions.

## Workflow

1. Establish impact and timeline from observed facts. Preserve relevant logs, task IDs and versions before restarting or deleting anything.
2. Compare recent changes, resource pressure and dependency status. Separate the immediate trigger from contributing conditions and unknown causes.
3. Choose the smallest authorized containment or recovery action with a rollback path. Do not destroy evidence or rotate every credential merely because one error occurred.
4. Validate recovery through the user-visible operation, not only a healthy process. Watch for recurrence and inspect partial effects before repeating failed actions.
5. Write a factual incident record with timeline, impact, evidence, mitigation and owned follow-ups. Communicate externally only when authorized.

For supplied host, memory or network evidence, load [forensic evidence and timelines](references/forensic-evidence.md). Preserve original timestamps, acquisition details and hashes; correlate event time, collection time and clock uncertainty. A detection hit is a lead, and missing telemetry cannot establish absence of compromise. Use detection-engineering when the deliverable is a tested detection rule.

For cloud identity incidents or traffic evidence, load [cloud and network evidence](references/cloud-and-network-evidence.md). Preserve control-plane changes, temporary sessions and collection gaps; correlate network observations before concluding compromise.

## Verification

Correlate timestamps and distinguish detection, onset and recovery. Report remaining degradation and unresolved hypotheses.

## Limits

Do not infer compromise from an outage alone or blame a person without evidence. This skill does not create an always-on monitoring service.
