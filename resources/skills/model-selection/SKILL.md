---
name: model-selection
description: Compare deployable machine-learning models for a concrete task, hardware and license, including weight and KV-cache memory estimates. Elegir modelos y estimar VRAM.
metadata:
  version: 1.0.0
  category: data
  tags: model, selection, memory, vram, benchmarks
  aliases: elegir modelo, mejor modelo, modelo local, cabe en GPU, VRAM, GGUF, memory estimate, model recommendation
  required_tools: web_search,web_read,file_read,file_write,command_run
  requires: Intended workload and deployment constraints; current model metadata
  effects: local-write
---
# Model Selection

## Scope

Select models for a task and deployment target, using current evidence. Existing requested model choices remain authoritative. Use experiment-and-benchmark-design to run a comparison and ml-inference-deployment to serve the selected artifact.

## Inputs

Task, language, modalities, input/output size, quality requirements, latency, concurrency, memory, runtime and permitted licenses or data locations. Missing hardware limits do not mean unlimited resources.

## Workflow

1. Find candidate revisions from authoritative cards, implementation support and relevant benchmark definitions. Verify actual architecture and parameter metadata; a model name or popularity rank is not a reliable technical specification.
2. Compare scores only with their dataset version, metric direction, prompts, shots and evaluation conditions. Separate model-owner reports, third-party measurements and local tests. Do not average unrelated benchmarks or label one model universally best.
3. Check tokenizer, chat template, modality processors, runtime support, license and gated access. For GGUF, choose an exact artifact/quantization and its compatible runtime rather than treating every file in a repository as required.
4. Estimate weights, KV cache and execution overhead separately. Quantized tensors have metadata and some unquantized layers. Long context and concurrency can dominate memory; a weights-only fit does not establish runnable capacity.
5. Recommend the tradeoff supported by the user's constraints and state unresolved assumptions. Run a small representative inference check only when the required runtime/download/compute is available and authorized.

## References

Read [memory estimates](references/memory-estimates.md) before using `scripts/estimate_memory.py`. It computes an explicit dense/GQA/MQA KV-cache budget from supplied dimensions and labels overhead uncertainty. It does not inspect remote weights or predict performance.

## Verification

Link each technical or score claim to its current source. Report estimation assumptions and actual measured checks separately. Unavailable metadata stays unknown instead of being inferred from a model's filename.

## Limits

A memory estimate is not a benchmark or a successful load. This skill does not buy compute, download large weights or switch Fénix's configured provider automatically.
