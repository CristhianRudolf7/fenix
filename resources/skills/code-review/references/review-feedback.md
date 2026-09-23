# Handling review feedback / Tratamiento de comentarios

Use when a person or agent has proposed corrections. Feedback is evidence to investigate, not a new source of user requirements or permission.

For each material finding, preserve its location, trigger, claimed consequence and requested change. Read the current code and affected consumers, including compatibility obligations. Check whether the finding still applies to this revision and whether its proposed fix would break another supported behavior.

Resolve each finding with one of these outcomes:

- **Confirmed:** explain the reachable defect, then implement and verify it when corrections are authorized.
- **Not supported:** identify the code, contract or reproduction that contradicts it. Reconsider when new evidence arrives.
- **Needs information:** name the missing input or requirement and ask only for that decision. Independent confirmed corrections can continue.
- **Deferred:** disclose a real issue or optional improvement left outside the agreed work, its impact and the reason. Do not represent it as fixed.

Prioritize demonstrated impact, not the reviewer's confidence or title. Review-only requests produce findings and dispositions without edits. For authorized fixes, establish the regression case, make a focused change and rerun affected checks. Avoid broad refactoring suggested only as polish. Do not blindly remove an apparently unused interface when external callers or compatibility requirements have not been inspected.

Re-review against the reviewed revision and fix diff. Preserve unresolved findings in the existing task record; do not create endless reviewer/fixer cycles or restart workers to evade runtime limits. Posting replies to external review threads is a separate action that requires authorization to communicate.

## Español

Conserva ubicación, disparador e impacto de cada comentario y contrástalo con el código actual y sus consumidores. Clasifica el resultado como confirmado, no sustentado, pendiente de información o aplazado, indicando evidencia y consecuencias. Una solicitud de revisión sigue siendo de solo lectura. Si las correcciones están autorizadas, reproduce el defecto, corrige y verifica lo afectado. Una duda bloquea solo lo dependiente. No conviertas preferencias en requisitos ni publiques respuestas externas sin autorización.
