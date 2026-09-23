# Agent trust-boundary checks

Map external content, retrieval results, tool metadata, conversation memory and model output to the operations they can influence. Identify the identity that owns each item and where permissions are enforced. Retrieved instructions remain data even when they resemble system messages or cite a supposed previous authorization.

For an authorized local evaluation, use inert canary strings and a recording stub for effects. Check whether untrusted content can change the target account, recipient, command, file path or tool choice; whether tool results can add durable instructions; and whether tenant filtering applies both when storing and retrieving memory. Score actual attempted/completed effects and leaked canaries, not merely whether the final prose sounds compliant. Include legitimate uses of the same content and tools so a blanket refusal is not mistaken for success.

Review the execution path for schema validation, per-user credentials, allowlists, cancellation and output handling. A router's outer request can contain multiple nested operations; validate each operation and target. If remote MCP execution bypasses a local SDK hook, that hook does not protect the remote path. Logs should retain nonsecret operation IDs and redacted evidence.

Inspect tool description/schema changes as review triggers. A hash mismatch is not proof of malicious intent, and a keyword scanner can miss an attack or flag benign instructions. Listing tools over stdio starts the configured process; treat it as execution and inspect the source/environment first. A remote scanner can also transmit descriptions or contact endpoints.

Report demonstrated paths, unmet preconditions and untested surfaces separately. Prefer a focused code or configuration correction with a negative regression case. Do not install a proxy, disable tools globally or export transcripts unless that is within the requested work.

## Español

Traza contenido no confiable hasta memoria, argumentos y efectos. Usa canarios inertes y herramientas simuladas para comprobar identidad, destinos y aislamiento, incluyendo casos legítimos. Las descripciones y anotaciones no aplican permisos. Enumerar un servidor MCP puede ejecutarlo; un escaneo limpio tampoco certifica seguridad.
