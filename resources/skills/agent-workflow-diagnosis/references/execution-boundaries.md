# Locating an agent failure

Use when a trace does not reveal whether the defect belongs to context assembly, tool execution or presentation. Work from the same task/request identity and compare available boundaries:

| Boundary | Evidence | Discriminating question |
| --- | --- | --- |
| Request to assembled context | Owner instructions, selected skill version, scoped memory references | Was an obligation lost or stale context admitted? |
| Context to provider request | Actual model/settings and serialized request if safely available | Did the application send the intended inputs? |
| Proposed tool call to execution | Validated arguments, policy decision, execution receipt | Was the call executed, rejected or merely described? |
| Execution to tool observation | Exit status, full saved output, truncation metadata | Did the model receive the relevant failure or artifact? |
| Model response to delivered response | Stored message, stream events and rendered result | Did parsing, retries or UI rendering alter the answer? |
| Current result to persistence | Stored version, scope, correction/withdrawal history | Was an old or inferred artifact reused as current fact? |

Inspect only boundaries needed to distinguish the hypotheses. Missing visibility stays a limitation. A source-code pattern or a second model's agreement is not a reproduced failure. Keep raw traces access-controlled; use references and redacted excerpts in reports.

For context pressure, inventory what this request actually loaded: system text, tool schemas, selected skill bodies/references, memory, history and results. Installed skills that were never loaded are not all prompt overhead. Separate static byte/character counts, tokenizer estimates and provider-reported usage; do not assume a fixed number of tokens per tool or a universal context window. Cached billing tokens still occupy context. Inspect duplicate content by source and scope before proposing removal.

If authorized to test recovery, preserve the original failure, vary one suspected layer and keep the same input/model/settings where possible. A direct-provider comparison may change privacy, cost and tools, so it is not a default probe. Prefer a captured synthetic fixture when sufficient. Record attempted fixes and stop repeating the same failed check without new evidence. Return the smallest supported runtime, tool or instruction change; do not label every failure a prompting problem.

## Español

Sigue una misma solicitud desde el contexto hasta la respuesta mostrada y la persistencia. Distingue llamada propuesta, ejecución y resultado observado. Mide contenido realmente cargado, no todo el catálogo instalado; una estimación de tokens no equivale a consumo medido. Conserva fallos y cambia una hipótesis por vez dentro del alcance autorizado.
