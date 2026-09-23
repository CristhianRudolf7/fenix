---
name: calendar-scheduling
description: Find availability and create, reschedule or cancel calendar events with correct timezones and recurrence
  scope.
metadata:
  version: 1.3.0
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

1. Resolve relative dates against the current date and user timezone; Use the configured user/account timezone; do not infer it from the deployment location. Ask when a date or AM/PM ambiguity changes the intended appointment.
2. Query availability in the relevant calendars. Treat missing access as unknown, not free. Consider travel/buffer constraints only when supplied by the user.
3. Propose feasible slots or use the already selected slot. Distinguish all-day dates from timestamped events, and one recurrence instance from an entire series.
4. Before creation or rescheduling, reread nearby events and check for an existing matching event. Specify attendee notification behavior according to the user request.
5. Read back the event ID, local and timezone-qualified time, attendees and recurrence after the mutation.

## Native Google Calendar operations

Discover `connector_call`, select the account and describe `calendar_list`; supply the intended `calendar_id`, RFC3339 `time_min`/`time_max` and bounded pagination. Omitting a time range does not mean today's agenda. Read `/users/me/settings/timezone` through `request` with `service=calendar` when using the account timezone. Inspect `workspace_schema` before unfamiliar event/recurrence changes, preserving notification settings and returned IDs. Generic POST reads such as free/busy currently require write-enabled connections; do not bypass that gate. Permission to inspect availability is still not permission to send invitations.

## Verification

For DST ambiguity, ICS or CalDAV synchronization, read [time and sync boundaries](references/time-and-sync.md). `scripts/local_time_candidates.py` identifies nonexistent or repeated local times without selecting a meeting or contacting a calendar.

Confirm start is before end, intended day in each relevant timezone and no duplicated event. Explicitly distinguish a suggested slot from a scheduled invitation.

## Limits

A calendar invitation sends information to others. Reuse existing user authorization but do not infer it from a request to inspect availability.

## Español

Resuelve fechas con zona horaria y distingue una ocurrencia de toda la serie. Revisa disponibilidad, duplicados y notificaciones antes de confirmar la cita.
