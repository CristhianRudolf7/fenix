# Time and synchronization / Tiempo y sincronización

Use an IANA zone such as `America/Lima`, not an ambiguous abbreviation. A wall time during a DST transition can map to no instant or two instants. `python3 <skill-directory>/scripts/local_time_candidates.py 2026-11-01T01:30 America/New_York` returns both UTC candidates; it does not choose one. Exit 0 means unique, 2 means ambiguous/nonexistent, and 1 means invalid input. Python needs system IANA data or `tzdata`; consult [zoneinfo](https://docs.python.org/3/library/zoneinfo.html).

For repeated events, preserve local wall-time intent and zone rather than adding 24 hours in UTC. For ICS, keep UID, TZID, recurrence exceptions and RECURRENCE-ID. An all-day DTEND is exclusive. A floating local time is not automatically UTC. The helper does not expand RRULE or validate an entire calendar.

Read a local CalDAV cache as a snapshot with freshness information. Synchronizing with vdirsyncer can push pending local writes as well as pull updates; do not call sync merely to inspect availability without checking its scope. Preserve ETags/sync tokens and handle conflicts against the actual changed revision. A global remote-wins rule can discard local edits. Re-read the exact event after an authorized write, keeping notifications and changes to one occurrence distinct from the series.

ES: detecta horas repetidas o inexistentes y conserva identidad y excepciones de eventos. Una sincronización puede publicar cambios pendientes; no equivale a una lectura inocua del calendario.
