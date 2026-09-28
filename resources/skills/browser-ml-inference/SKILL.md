---
name: browser-ml-inference
description: Implement client-side or JavaScript model inference with Transformers.js, ONNX, WebGPU or WASM and verified device fallback. Inferencia de modelos en el navegador.
metadata:
  version: 1.0.0
  category: data
  tags: browser, inference, transformersjs, onnx, webgpu, wasm
  aliases: Transformers.js, WebGPU, WASM, ONNX, modelo en navegador, inferencia JavaScript, client-side ML
  required_tools: file_read,file_write,file_edit,command_run,tool_search,web_read
  requires: Supported JavaScript runtime, compatible model artifacts and a browser for client verification
  effects: local-write
---
# Browser ML Inference

## Scope

Run supported models through a JavaScript inference runtime. Browser deployment differs from a Python GPU server: exported operators, download size, browser capabilities and worker lifecycle matter. Use model-selection for candidate comparison.

## Inputs

Task and model revision, Transformers.js/runtime version, ONNX artifacts and dtype, target browsers, device constraints, data-handling expectations and offline requirements.

## Workflow

1. Check the installed runtime's current model/task/device support. A Python Transformers model is not automatically an executable Transformers.js export. Verify tokenizer/processor files and actual ONNX operator compatibility.
2. Choose device and dtype from tested capabilities. Detect WebGPU availability and handle adapter or allocation failure; provide a supported WASM/CPU path when the application requires it, explaining different performance and output tolerance.
3. Keep initialization and inference off the UI thread where appropriate. Reuse loaded pipelines, bound request concurrency, dispose replaced sessions and suppress stale results after cancellation or component unmount.
4. Show model download progress and actionable errors. Respect CSP, model origin and cache constraints; cross-origin isolation requirements depend on the backend. Do not advertise offline operation until a cold reload with the network disabled succeeds.
5. Test representative and malformed inputs in the actual browser/runtime on each claimed backend. Compare preprocessing and output with a reference, using task-appropriate numerical tolerance rather than expecting byte-identical GPU output.

## References

Read [JavaScript inference checks](references/javascript-runtime.md) for capability discovery, worker messaging and cache boundaries.

## Verification

Record runtime, model revision, artifacts, dtype and backend actually used. Check load failures, input responsiveness, disposal and device fallback. A successful bundle build does not prove model inference.

## Limits

Model files and runtimes remain external dependencies. Local inference still downloads model assets unless provisioned otherwise. Do not claim that text/audio remains local until network behavior and provider fallbacks have been checked.
