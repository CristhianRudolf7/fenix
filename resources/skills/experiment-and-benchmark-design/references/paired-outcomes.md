# Paired agent and skill outcomes

Use for a baseline/candidate comparison based on completed task artifacts and recorded effects. The local `scripts/compare_outcomes.py` summarizes supplied records; it does not run a model, judge artifacts, verify reported usage or install a winning skill.

Freeze tasks, artifact-based rubric, repetitions and gate before editing. Keep development, selection and final test tasks disjoint. Repeatedly tuning against selection cases makes them development evidence; reserve a fresh final set. Avoid exposing expected outputs or grader internals to the acting agent. A controlled comparison holds model, settings, tools, permissions and starting state constant; record any deliberate difference and attribute the result accordingly.

For repeated trials, pair explicit run IDs within each task. Start each trial from isolated state, avoid carrying baseline solutions into candidate context, and preserve the original failed attempt when retrying. Prefer executable outcome checks; calibrate subjective graders with examples and blind condition labels. An instruction count, keyword match, compliant heading or confident language is not an outcome score.

## Local input contract

One JSON document contains:

```json
{
  "schema_version": 1,
  "conditions": {
    "baseline": {"revision": "before", "execution_contract": "same-model-tools-state-v1"},
    "candidate": {"revision": "after", "execution_contract": "same-model-tools-state-v1"}
  },
  "tasks": [{"id": "repair-timeout", "split": "selection", "critical": true}],
  "run_ids": ["r1"],
  "gate": {"min_delta": 0, "max_task_regression": 0},
  "outcomes": [
    {"condition": "baseline", "task_id": "repair-timeout", "run_id": "r1", "status": "completed", "score": 0, "evidence": "baseline/test-result.json"},
    {"condition": "candidate", "task_id": "repair-timeout", "run_id": "r1", "status": "completed", "score": 1, "evidence": "candidate/test-result.json"}
  ]
}
```

IDs are nonempty strings. Splits are `development`, `selection` or `test`. Status is `completed`, `timeout` or `error`; unfinished trials have score zero. Scores are finite numbers in `[0,1]`. Every supplied outcome has an evidence reference. Optional `tokens`, `seconds` and `cost` are nonnegative measured values; omit unknown values. The execution-contract label is declared provenance, not a verified sandbox or a settings check. Store the actual model/settings/environment with the experiment artifacts and use artifact hashes as revisions where available.

Run `python3 scripts/compare_outcomes.py results.json`. The helper rejects duplicate keys, duplicate identities, foreign task/run IDs and incompatible declared execution contracts. Missing trials remain in the planned denominator as zero and block the selection gate. It averages repetitions per task, reports per-split and per-task deltas, distinguishes improvements/regressions and reports usage coverage. Repeated seeds are not treated as independent tasks.

The gate requires complete selection records, mean gain strictly greater than `min_delta`, no critical-trial regression and no other task-mean regression larger than `max_task_regression`. Each repetition of a critical task is checked separately so a better run cannot hide a worse one. Test-set scores are reported separately and are never used to select an edit. Exit codes: `0` meets this declared gate, `2` does not meet it or is incomplete, `1` malformed input. No p-value, confidence interval or general superiority is inferred. An A/A comparison should have zero delta and no improvements; it does not pass a strictly positive improvement gate.

## Español

Compara tareas y repeticiones con IDs explícitos, estado aislado y criterios fijados antes. Conserva fallos, tiempos agotados y métricas desconocidas. El auxiliar calcula resultados declarados y bloquea datos incompletos; no ejecuta el agente ni demuestra significancia estadística. El conjunto de prueba final permanece separado de la selección de cambios.
