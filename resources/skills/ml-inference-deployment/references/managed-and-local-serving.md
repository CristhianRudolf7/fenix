# Managed and local serving / Inferencia administrada y local

For llama.cpp/GGUF, select an exact compatible artifact, tokenizer/chat template and runtime version. Verify model load and a real request, then measure context/concurrency limits. Conversion and quantization need a comparison against the source model on representative inputs; file creation alone proves neither compatibility nor preserved quality.

For SageMaker or another managed endpoint, resolve the actual account, region, existing role, quotas and supported serving container. Discover current image URIs from authoritative provider documentation and pin the resulting identity. Match generative models, embeddings and rerankers to the container's actual request/response contract; do not infer capability solely from the model name or vendor preference.

Choose real-time, asynchronous, batch or scale-to-zero behavior from workload and latency requirements. Define cold-start and retry semantics, output persistence, autoscaling and observability. Reuse a suitable authorized execution role before proposing IAM changes. Distinguish an inaccessible quota preflight from an actual resource denial. Record created resource IDs and ownership to support precise teardown without touching unrelated resources.

An `InService` endpoint still needs real inference, error handling and a check of the intended consumer. Deployment permission does not imply permission to broaden IAM, enable input capture or create unlimited capacity. Keep paid resources and rollback within the established task constraints. Recheck current prices and platform limits when needed instead of preserving dated examples in this skill.

ES: usa artefactos y contenedores compatibles, cuenta/región/rol reales y capacidad autorizada. Comprueba solicitudes reales, arranque en frío y persistencia. Conserva identificadores para restaurar o retirar solo los recursos de la tarea.
