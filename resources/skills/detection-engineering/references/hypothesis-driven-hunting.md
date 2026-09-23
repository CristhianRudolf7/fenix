# Hypothesis-driven hunting

Start with a behavior, preconditions, affected population and a bounded time window. Write what evidence would support or weaken the hypothesis and which benign activity could resemble it. Confirm relevant sources/fields are collected for that population before interpreting a query result. A framework ID indexes a behavior; it does not prove visibility or deployment coverage.

Keep original queries and results with source/version, timezone, interval, pagination, samples and collection gaps. Test joins, event deduplication, grouping and temporal windows against known examples. Establish whether the query operates on event time or ingestion time. For rare activity, report counts and denominator rather than a universal volume threshold.

Correlate independent anchors such as process parent/child, user/session, DNS and endpoint events. Record contradictory evidence and alternative explanations. Threat-intelligence indicators need provenance, age and applicability; do not equate a shared IP match with an actor attribution. When results are negative, report no supporting evidence in the observed scope rather than absence of compromise.

Promote a hunt query to a detection rule only after positive/benign tests and an alert-volume review. Query creation, scheduled execution, rule enablement, alert delivery and automated containment are separate changes. Confirm the real delivery path if deployment was requested; no live replay or automatic blocking is necessary for offline rule testing.

## Español

Formula hipótesis refutables, población y ventana antes de consultar. Conserva cobertura y denominadores, correlaciona evidencias y considera alternativas benignas. Sin resultados no significa sin compromiso. Convertir una consulta en alerta requiere probar comportamiento y entrega.
