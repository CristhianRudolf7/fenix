# Adaptive plan review

[Español](../es/plan-reviews.md)

Fénix reviews complex plans before dispatch and supports selective review when new
evidence reveals a problem. Reviewers use the master's configured provider, model and
reasoning setting. The chat remains available during review and clarification.

## Intent and admission

The master retrieves confirmed decisions, inspects evidence and asks unresolved
questions before committing to scope. `plan_manage.brief` separates `requirements`
(including sources), `observations`, `assumptions` and `open_questions`. Assumptions
are proposals, not owner requirements. “Improve login” does not implicitly request
social login or two-factor authentication. Group related questions, reuse answers,
plan outcomes first and detail near-term milestones as evidence becomes available.
Continue only independent authorized work while waiting.

Initial review is automatic for three or more milestones, any dependency, assumptions,
open questions or an explicit `review_reason`. These are initial engineering rules,
not a learned complexity detector. The master should provide a reason for shorter
but consequential or ambiguous plans. Simple clear plans avoid extra inference.

`plan_manage review` requires `plan_id`, `expected_version` and a concrete
`review_reason`. Optional `review_nodes` gates those nodes and transitive dependants.
Blocked/failed milestones trigger review once per worker execution. No inference is
used for polling and failed effects are not automatically replayed. Gates prevent
new dispatch; existing actions are not retroactively stopped. Use replan/cancel to
replace active work. Unrelated ready branches continue.

## Reviewers and evidence

One principal checks alignment, unsupported scope, dependencies, feasibility and
completion criteria. It may request up to two specialists with distinct questions,
milestone scopes and suggested skills. The scheduler creates siblings under the
original master task. Specialists cannot request more agents. All retain the shared
skill metadata catalog and load enabled bodies on demand.

`plan_review_read` supplies the original request, brief, bounded full milestone pages
and answered questions. Independent reviewers do not receive peer conclusions.
Scoped milestone journals are accessible with `task_inspect`. Reviewers may read files,
retrieve sources and memory, load skills and ask questions. The runtime denies commands,
file writes, memory mutations, plan changes and delegation, even for fabricated calls.
Checks requiring commands are proposed to the master. Reviewing a plan is not proof
that its execution succeeded.

`subagent_finish.review` includes `verdict` (`pass`, `changes_requested`, `needs_user`),
`findings` and optional principal-only `specialists`. Findings require ID, category,
severity, description, evidence and optional node IDs. Categories are `intent`, `scope`,
`dependency`, `feasibility` and `evidence`; severity is `blocking` or `advisory`. A pass
cannot contain blocking findings; other verdicts require one. Invalid or plain-text
reports do not release a gate.

## Resolution and limits

All independent reports are collected. If all pass, dispatch is released. Any blocking
finding creates one background master resolution turn. Two passes cannot outvote an
objection. Factual disagreements require sources; unknown preferences require an owner
answer. This resolution turn is restricted to reads, questions and its assigned plan.

`plan_manage resolve_review` records each blocking `role/finding_id` with disposition
`addressed`, `dismissed` or `needs_user`, rationale and evidence. An owner answer or
factual rebuttal may settle the existing plan. `revise` instead records resolutions and
an updated brief, preserving valid completed effects. The engine checks resolution
structure and coverage, not the truth of model-authored evidence.

There is one initial review and one master response. A revision receives one final
check focused on changed milestones/dependencies; changed global objectives or
constraints require a global check. No additional specialists or automatic corrections
are allowed in that final check. Remaining objections block the affected scope and
return to the original channel. A failure after correction may be reviewed but cannot
restart another automatic correction cycle. Owner edits may start a fresh cycle.

Review jobs share four concurrent child inference slots and 32 live children. Questions
release capacity while awaiting answers. The three transient-provider attempts retain
completed tool results and are distinct from semantic review rounds.

## Persistence and visibility

Migration 5 adds version-pinned `plan_reviews` and `plan_review_workers`. Version checks
surround inference, tools and report application. Edits cancel obsolete reviews and
questions; late reports cannot release a new version. Restart preserves audit data,
marks unfinished reviews interrupted and does not resume effects.

Tasks shows review state and scope. Plan details include findings, resolutions and
provider-reported usage for reviewers and the resolution turn. Elapsed time includes
waiting. Missing usage is identified; failed API calls without usage are not counted
as known consumption. This does not estimate prices or measure task quality. Events
and questions use the existing durable pub/sub and channel outbox, preserving one
conversation in web and Telegram.

Tests with a mock provider cover dispatch gates, independent branches, forbidden
effects, questions with a responsive chat, disagreement, bounded corrections, stale
results, persistence and usage. Ambiguity detection remains a model responsibility.
Real-model evaluations are still needed to measure quality, latency, token efficiency
and optimal trigger thresholds; no benchmark improvement is claimed.
