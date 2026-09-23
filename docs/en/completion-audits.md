# Selective completion auditing and evidence

Fénix distinguishes a worker finishing execution from an independently verified
result. This extends the existing versioned planner; it does not replace initial
plan review or require another model call after each tool action.

## Admission and execution

- Small, clear tasks keep the direct execution path.
- A milestone that supplies another milestone waits in `verifying` after successful
  execution. Its dependent milestones cannot dispatch until an auditor accepts it.
- In a multi-milestone plan, a worker with at least 12 substantive tool results also
  receives a completion audit. Tool discovery, skill loading, command status polling
  and finishing are excluded from this deterministic complexity heuristic.
- Plans with three or more milestones, dependencies, or substantial observed work
  receive a final integrated audit. Set `plan_manage.audit_required=true` for a large
  single-stage plan. A single-stage plan receives one final audit, not two identical
  milestone/final reviews. Ordinary unplanned turns are not automatically audited:
  the agent is instructed to route long work through the planner.

The engine creates a fresh sibling auditor on the existing worker pool using the
master's configured model. The master and independent branches stay available.
Auditors cannot delegate, receive supervisor side-channel instructions, write files,
execute shell commands, call connectors, or change the plan. Read tools are filtered
both from discovery and at execution. Artifact content remains untrusted data.

The auditor retrieves the original owner request, current objective and constraints,
version history, relevant milestone outcomes, confirmed answers and evidence in pages.
Worker reports are claims; the auditor must inspect actual deliverables. Final review
checks the integrated result against the request and explicit amendments, including
requirements the planner might have missed. It must not invent new acceptance criteria.

## Evidence ledger

SQLite migration 10 stores `completion_audits` and `plan_evidence`. Ledger records
have independent dimensions:

| Dimension | Values |
| --- | --- |
| Kind | `task`, `requirement`, `artifact`, `fact` |
| State | `pending`, `verified`, `doubtful`, `refuted` |

Each entry retains a statement, source, author task, optional milestone and question,
plan version, time, audit reference and observation call references. Original owner
requests, plan objectives and milestone outcomes are seeded automatically. The plan
brief supplies sourced candidate requirements, observations, assumptions and questions.
Executor-declared artifacts are captured as pending claims, never automatic proof.

`plan_evidence list` retrieves bounded pages. `record` appends a unique `note:` entry
using `expected_version`; executors can submit only pending/doubtful candidates in
their own scope. It cannot overwrite the owner request or self-certify completion.
Facts can be refuted by an auditor with evidence; this cannot remove a requirement or
dismiss an unresolved owner question. Sources describe provenance, not authorization.

When a candidate has `question`, Fénix saves it before calling the existing asynchronous
text clarification mechanism. Other tasks and the conversation remain available.
The answer persists in the question journal/task contract. An answer confirms intent;
it does not certify an artifact. Auditors consult earlier answers before asking again.

## Acceptance, revisions and failures

`subagent_finish.completion_audit` reports `verified`, `incomplete` or `needs_user`
with per-entry checks and observation call IDs. A pass must cover every assigned
entry and cite successful observations made by that auditor. Invented IDs, executor
observations, failed reads, schema-loading receipts and unanswered questions cannot
qualify as proof. Factual candidates explicitly refuted with evidence are resolved;
requirements and task/artifact outcomes must be verified.

`file_read` includes the SHA-256 of the complete source text even for a paginated
read. Before accepting a pass, the runtime compares cited text files with those
hashes; changed files require fresh reads. The report retains the full observations
and any document hashes/design revisions returned by their native read tools. This
is a point-in-time assessment, not a filesystem lock or permanent correctness guarantee.
The model still judges whether the observed contents substantiate a requirement.

An incomplete audit blocks its milestone/dependants or the final plan, and sends the
findings to the original channel. It does not automatically replay effects or start
a reviewer debate. Repair requires a concrete plan revision. Version checks fence
late audit reports; cancellation and replanning stop affected auditors. New versions
retain historical evidence but reset current proof. Global objective/constraint changes
invalidate prior milestones; unchanged valid work otherwise remains available.

Audit and ledger data survive context compaction and process restart. Restart marks
unfinished audits interrupted rather than replaying actions. The task UI exposes
verification status and ledger counts; plan details include reports and provenance.

This is a read-only inspection auditor. Tests needing execution remain executor work;
missing evidence or an inaccessible external outcome must be reported as incomplete.
No claim is made that these changes increase a benchmark score without new evaluation.
