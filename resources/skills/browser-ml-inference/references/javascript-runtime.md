# JavaScript inference / Inferencia en JavaScript

Inspect the project's actual Transformers.js major version before choosing imports and options. Use compatible ONNX exports and tokenizer/processor files from a pinned model revision. Backend/device/dtype support varies by model, operator and browser; verify it against [the current runtime documentation](https://huggingface.co/docs/transformers.js/en/index).

A worker message protocol should include a request ID so late responses cannot overwrite a newer result. Track initialization separately from inference, report download progress, limit simultaneous runs and release superseded model sessions. Test teardown during initialization and device loss or allocation failure. A WASM fallback can require a different dtype or artifact set; do not merely change a label while leaving an unsupported graph loaded.

For offline/local-only claims, inspect asset origins, caching and outbound traffic. Verify with a fresh page load, then a reload after disabling network; an already-loaded model in memory is insufficient. Do not route private prompts to a hosted API on fallback unless that behavior is requested and disclosed. Keep precision differences in the quality comparison and use representative task outputs rather than an arbitrary tensor equality check.

ES: comprueba operadores y archivos reales, identifica peticiones para descartar resultados antiguos y libera sesiones. Prueba fallos del backend, carga sin conexión y tráfico antes de afirmar que los datos permanecen locales.
