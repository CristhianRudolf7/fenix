---
name: llm-cost-optimization
description: Diagnose LLM API spending and reduce cost per successful task with measured usage, bounded retries, caching and quality-preserving routing. Optimizar costes de inferencia LLM.
metadata:
  version: 1.0.0
  category: engineering
  tags: llm, inference, cost, usage, budget
  aliases: gasto LLM, reducir coste tokens, factura API modelos, LLM cost optimization, inference spending, cost per successful task, prompt caching
  required_tools: file_read,file_write,task_inspect,web_search,web_read,command_run
  requires: Authorized usage records, current provider pricing and representative task outcomes
  effects: local-write
---
# LLM Cost Optimization

## Scope

Investigate model API spend or design an application's inference budget. Use model-selection for hardware/model suitability and cloud-cost-review for infrastructure bills. Keep explicitly selected models and privacy constraints authoritative.

## Inputs

Date range, account/project, currency, exact provider/model versions, task outcomes, usage dimensions, pricing date and approved experimentation budget. Missing usage is unknown, never zero.

## Workflow

1. Reconcile request IDs with retries, workers, background calls and task IDs. Avoid summing provider totals again with their child records. Separate input, output, cache writes/reads, reasoning and other billable units according to the provider's current definitions; some fields overlap rather than add.
2. Distinguish invoice amounts, usage-derived estimates and preflight bounds. Record pricing source/date, currency, tiers, batch discounts and applicable rounding. Keep fixed subscriptions and infrastructure separate. Do not use model names or prices embedded in an old example as current facts.
3. Attribute waste to observed causes: repeated retrieval, oversized context, failed attempts, duplicate jobs, unbounded output or unnecessary model calls. Count failed attempts in total spend. Report total cost and cost per successful task with the success denominator; with zero successes, the ratio is undefined.
4. Try the smallest supported change. Prefer deterministic processing for deterministic work, narrower retrieval and bounded outputs before adding routing machinery. Test cache isolation and invalidation. Provider prompt caching differs from storing and reusing a model's answer; neither implies fewer tokens entered the context.
5. If model routing is appropriate, evaluate representative and difficult tasks against the existing baseline. Input length alone is not task difficulty. Preserve minimum quality and tail-latency constraints, account for fallback calls, and retain the requested model override. Propose changes without silently switching Fénix's own provider.
6. For application budgets, reserve an upper bound before dispatch, including permitted retries and concurrent calls, then reconcile actual usage and release unused reservations. A check after a completed request reports overspend but cannot prevent it. Require the runtime/accounting store to enforce atomic reservations; skill instructions cannot provide a hard spending cap.

## Verification

Recompute totals from unique records and explain uncovered usage. Compare the same tasks under the same measurement conditions, retaining errors and timeouts. A lower bill is not an improvement if tasks fail, quality falls or required work is omitted.

## Limits

Do not buy credits, change account billing, export private traces or run paid comparisons without applicable authorization. Price estimates and small development samples do not prove future savings.
