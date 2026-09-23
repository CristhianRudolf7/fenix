# Spaces, adapters and ZeroGPU / Spaces, adaptadores y ZeroGPU

Check current Space SDK/runtime support and the account's actual hardware entitlement. Pin compatible application dependencies and model revisions; do not hardcode a historical GPU generation, quota or price. README metadata must agree with the deployed SDK and application entrypoint. Choose Gradio, Docker or Static according to the real workload; hosted build behavior can differ from a local environment.

For LoRA, inspect base-model metadata, architecture, pipeline task, target layers, adapter strength and trigger tokens. Match image, edit, control or video inputs to that pipeline. Keep mutable adapter state safe under concurrency. Validate an actual request and inspect output bytes before publishing a success claim.

For ZeroGPU, consult [current platform documentation](https://huggingface.co/docs/hub/spaces-zerogpu). It currently uses Gradio and `spaces.GPU` to scope GPU execution. Model placement at startup and real CUDA computation are different: follow the platform's supported module-level loading pattern, and run computation inside the decorated function. Declared duration affects admission/quota; measure representative runtime. Worker process boundaries require serializable inputs/outputs, isolated per-user state and care with mutable models. `torch.compile` support must not be assumed.

An ordinary local CPU/GPU run cannot verify ZeroGPU scheduling, quotas or worker behavior. Test those in the authorized target Space and state when no hosted test was performed. Diagnose build logs separately from inference failures. Temporary storage is not automatically durable; record where user outputs and checkpoints persist.

ES: verifica compatibilidad de modelo y adaptador, aislamiento entre usuarios y metadatos del Space. Consulta versiones y límites actuales. Una prueba local no certifica la ejecución de ZeroGPU ni concede recursos de la cuenta.
