---
name: ml-inference-deployment
description: Serve local or remote models with resource sizing, tokenizer compatibility and latency-quality checks.
metadata:
  version: 1.1.0
  category: data
  tags: ml, inference, deployment
  aliases: servir modelo, inferencia, vllm, llama.cpp, cuantización, SageMaker, endpoint, serving container
  required_tools: file_read,file_write,command_run
  requires: Compatible model artifact, serving runtime and authorized deployment target
  effects: mixed
---
# Ml Inference Deployment

## Scope

Serve local or remote models with resource sizing, tokenizer compatibility and latency-quality checks.

## Inputs

Model revision, request format, concurrency and latency/quality requirements. Prerequisite: Compatible model artifact, serving runtime and authorized deployment target.

## Workflow

1. Check model license, architecture, tokenizer/chat template and runtime support. Estimate weights, KV cache and concurrency memory separately.
2. Compare quantization/batching choices on representative requests; nominal model size alone does not predict quality or throughput.
3. Validate streaming, stop conditions, structured outputs and cancellation. Apply request/account isolation and avoid logging prompts by default.
4. Deploy the versioned artifact and verify readiness plus real inference; preserve a compatible rollback target.

Read [managed and local serving](references/managed-and-local-serving.md) for SageMaker deployment contracts, account/role discovery, container compatibility, scale-to-zero or llama.cpp/GGUF. Use model-selection for hardware-aware candidate choice, browser-ml-inference for a JavaScript/ONNX client, and ml-demo-apps for a Gradio/Spaces interface.

## Verification

Measure time to first output, total latency, throughput and error rate at stated concurrency, alongside quality.

## Limits

Do not call a loaded model healthy before inference succeeds or assume quantization preserves every capability.

## Español

Dimensiona pesos y caché, valida plantillas y streaming y mide calidad junto con latencia y concurrencia.
