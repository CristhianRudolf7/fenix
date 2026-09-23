---
name: calendar-scheduling
description: Find availability and create, reschedule or cancel calendar events with correct timezones and recurrence
  scope.
metadata:
  version: 1.2.0
  category: office
  tags: calendar, scheduling
  aliases: calendario, agenda, reunión, disponibilidad, agendar cita, CalDAV, daylight saving, hora ambigua, horario de verano, schedule meeting, time zones, programar reunión
  required_tools: tool_search,command_run,file_read,file_write
  requires: Authorized calendar connector or supplied availability
  effects: external-write
---
# Calendar Scheduling

## Scope

Use for calendar events involving availability and attendees. Use memory-and-reminders for a personal notification in Fénix.

## Inputs

Calendar/account, date or window, duration, timezone, attendees and recurrence scope.

## Workflow

1. Resolve relative dates against the current date and user timezone; Fénix defaults to America/Lima unless configured otherwise. Ask when a date or AM/PM ambiguity changes the intended appointment.
2. Query availability in the relevant calendars. Treat missing access as unknown, not free. Consider travel/buffer constraints only when supplied by the user.
3. Propose feasible slots or use the already selected slot. Distinguish all-day dates from timestamped events, and one recurrence instance from an entire series.
4. Before creation or rescheduling, reread nearby events and check for an existing matching event. Specify attendee notification behavior according to the user request.
5. Read back the event ID, local and timezone-qualified time, attendees and recurrence after the mutation.

## Verification

For DST ambiguity, ICS or CalDAV synchronization, read [time and sync boundaries](references/time-and-sync.md). `scripts/local_time_candidates.py` identifies nonexistent or repeated local times without selecting a meeting or contacting a calendar.

Confirm start is before end, intended day in each relevant timezone and no duplicated event. Explicitly distinguish a suggested slot from a scheduled invitation.

## Limits

A calendar invitation sends information to others. Reuse existing user authorization but do not infer it from a request to inspect availability.

## Español

Resuelve fechas con zona horaria y distingue una ocurrencia de toda la serie. Revisa disponibilidad, duplicados y notificaciones antes de confirmar la cita.
