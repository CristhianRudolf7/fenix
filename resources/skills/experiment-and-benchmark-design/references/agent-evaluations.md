# Agent evaluation boundaries

Use when comparing agent configurations or measuring skill adherence. Freeze task requirements, starting artifacts, tool permissions and outcome rubrics before exposing the task to a candidate. Keep grader inputs and expected answers out of the acting agent's context where the evaluation requires independence.

Judge the requested artifact and actual effects. A grep match, tool-call sequence or self-rating can help diagnose a run but cannot alone establish correctness. Different valid solutions may use different tool sequences. Test the grader with a known correct artifact and plausible invalid artifacts, including an empty output, a fabricated success message and a weakened test. A grader that accepts these needs repair before comparing agents.

Record whether evidence came from deterministic checks, a calibrated human assessment or a model judge. Model judgments remain fallible; use blinded condition labels and retain disagreements/unknowns. Report task success and instruction adherence separately. Repeated samples are not independent tasks, and best-of-N success is different from first-attempt success; include the cost of all attempts.

A directory or Git worktree separates working files, not OS/network authority. Run generated code, scenario setup and project test commands inside the configured isolation boundary. An allowlist containing a package manager does not make its installation scripts safe. Use synthetic accounts and bounded fixtures; do not inherit real credentials or turn a benchmark into external messages or purchases.

Keep setup errors, timeouts and unavailable dependencies visible. If setup did not produce the intended state, mark that trial invalid/incomplete with its reason rather than scoring an easier task. Use the existing [paired outcome contract](paired-outcomes.md) for aggregation and preserve its missing-trial semantics.

## Español

Puntúa artefactos y efectos, no solo secuencias de herramientas o autoevaluaciones. Comprueba que el evaluador rechace resultados plausibles pero incorrectos. Una carpeta o worktree no sustituye el aislamiento; registra errores de preparación y todos los intentos, incluidos fallos y tiempos agotados.
