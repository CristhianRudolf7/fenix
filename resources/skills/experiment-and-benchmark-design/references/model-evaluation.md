# Model evaluation execution / Ejecución de evaluaciones

Choose an evaluation framework such as Inspect or LightEval only when its task definition and backend match the requested comparison. Record framework/task/model revisions, inference backend, prompt/chat template, few-shot settings, sample limits, generation settings and scoring. Inspect the current installed CLI/schema; task identifiers and backend flags can change.

Run a bounded smoke test to detect data, tokenizer or runtime errors before spending on the full evaluation. Keep smoke/development results separate from the locked evaluation set. A provider-backed run may execute inference remotely even if the evaluation script runs locally; account for data transmission and billed usage accordingly.

Use the same effective task and scoring contract across backends or disclose the difference. Changing from vLLM to Transformers, altering precision or enabling remote repository code can change behavior. Review required custom code before enabling it; do not treat `trust_remote_code` as a routine fix for every load failure.

Preserve per-example outputs, missing scores, failures and retries. Recompute aggregates with denominators and compare paired tasks where appropriate. Writing a model card or uploading results is a separate publication step. For remote jobs, use huggingface-hub's available CLI/SDK workflow; no missing upstream skill name or developer-local checkout is assumed.

ES: registra el contrato efectivo de evaluación, diferencia prueba pequeña de evaluación reservada y conserva fallos. Ejecutar el evaluador localmente no garantiza inferencia local. Publicar resultados requiere su alcance propio.
